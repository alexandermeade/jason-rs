use std::collections::{BTreeMap, HashSet};
use std::collections::hash_map::HashMap;
use serde_json::{Value};

use crate::token::ArgsToNode;
use crate::astnode::ASTNode;
use crate::jason_errors::{JasonError, JasonErrorKind, JasonResult};
use crate::context::Context;
use crate::token::TokenType;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum JasonType {
    String,
    Number,
    Bool,
    Null,
    Any,
    Union(Vec<Box<JasonType>>),
    List(Box<JasonType>),
    Object(BTreeMap<String, JasonType>),
}

impl Context {
    
    pub fn to_type(&mut self, node: &ASTNode) -> JasonResult<JasonType> {
        match &node.token.token_type {
            TokenType::ID           => Ok(
                self.types.get(&node.token.plain())
                .ok_or_else(|| 
                    self.err(
                        JasonErrorKind::UndefinedVariable(node.token.plain()), 
                        format!("The type {} is not defined\n hint: to define the type do  {} :: T", node.token.plain(), node.token.plain())
                    )
                )?.clone()
            ),
            TokenType::StringType   => Ok(JasonType::String),
            TokenType::NumberType   => Ok(JasonType::Number),
            TokenType::BoolType     => Ok(JasonType::Bool),
            TokenType::AnyType      => Ok(JasonType::Any),
            TokenType::NullType     => Ok(JasonType::Null),
            TokenType::Bar          => {
                let left = node.left.as_ref().ok_or_else(||
                    self.err(
                        JasonErrorKind::MissingValue,
                        format!("missing left side of bar expression")
                    )
                )?;

                let right = node.right.as_ref().ok_or_else(||
                    self.err(
                        JasonErrorKind::MissingValue,
                        format!("missing right side of bar expression")
                    )
                )?;

                
                let left_type:JasonType = self.to_type(&left)?;
                let right_type:JasonType = self.to_type(&right)?;
                
                Ok(match (left_type, right_type) {
                    (typ, JasonType::Union(mut args)) |
                    (JasonType::Union(mut args), typ)  => {
                        args.push(Box::new(typ));
                        JasonType::Union(args) 
                    },
                    (typ1, typ2) => {
                        JasonType::Union(vec![Box::new(typ1), Box::new(typ2)])
                    }, 
                })
            },
            TokenType::Block(args)  => {
                let nodes = args.to_nodes()?;
                let mut map:HashMap<String, JasonType> = HashMap::new(); // this will become our typed Object
                for node in nodes {
                    if node.token.token_type == TokenType::Colon {
                        let key_node = node.left.as_ref().ok_or_else(||
                            JasonError::new(JasonErrorKind::MissingKey, self.source_path.clone(), self.local_root.clone(), "Missing key"))?;
                        let value_node = node.right.as_ref().ok_or_else(||
                            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(), self.local_root.clone(), "Missing Type"))?;
                        if key_node.token.token_type != TokenType::ID {
                            return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(),
                                self.local_root.clone(), "Key must be an ID"));
                        }
                        let key = key_node.token.plain();
                        let value = self.to_type(&*value_node)?; // recursive call
                        map.insert(key, value);
                        continue;
                    }
                    return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                        "values must adheere to <key : value> fields in blocks"));
                }
                return Ok(JasonType::Object(map.into_iter().collect()))
            },

            TokenType::List(values) => {
                if values.is_empty() {
                    return Ok(JasonType::List(Box::new(JasonType::Any)));
                }
                //collect inner types as a union and then propogate that as the type
                let mut inner_types = vec![];
                for node in values.to_nodes()? {
                    let t = self.to_type(&node)?;
                    inner_types.push(Box::new(t));
                }

                let inner_type = if inner_types.len() == 1 {
                    inner_types.remove(0)
                } else {
                    Box::new(JasonType::Union(inner_types))
                };

                Ok(JasonType::List(inner_type))
            }
            _ => Err(
                JasonError::new(
                    JasonErrorKind::ValueError, 
                    self.source_path.clone(), 
                    self.local_root.clone(), 
                    format!("unexpected token {:?} when evaluating type", node.token.token_type)   
                )
            ) 
        }
    }

    pub fn infer_type_from(&mut self, value: &serde_json::Value) -> JasonResult<JasonType> { 
        match value { 
            Value::String(_) => return Ok(JasonType::String),
            Value::Number(_) => return Ok(JasonType::Number),
            Value::Null => return Ok(JasonType::Null),
            Value::Bool(_) => return Ok(JasonType::Bool),

            Value::Array(values) => {
                let mut infered_types: Vec<JasonType> = values
                    .iter()
                    .map(|e| self.infer_type_from(e))
                    .collect::<JasonResult<HashSet<JasonType>>>()?
                    .into_iter()
                    .collect(); 
                
                match infered_types.len() {
                    0 => Ok(JasonType::List(Box::new(JasonType::Any))),
                    1 => Ok(JasonType::List(Box::new(infered_types.remove(0)))),
                    _ => Ok(JasonType::List(
                            Box::new(
                                JasonType::Union(
                                        infered_types
                                        .into_iter()
                                        .collect::<HashSet<JasonType>>()
                                        .into_iter()
                                        .map(Box::new)
                                        .collect()
                                    )
                                )
                            )
                        )
                }
            },
            Value::Object(mapping) => {
                Ok(
                    JasonType::Object(
                        mapping
                        .iter()
                        .map(|(k, v)| {
                            Ok((
                                k.clone(),
                                self.infer_type_from(v)?
                            ))
                        })
                      .collect::<JasonResult<_>>()?
                    )
                )
            },
        }
    }


}

impl JasonType {
    pub fn matches(&self, value: &Value) -> bool {
        match self {
            JasonType::Any => true,

            JasonType::String => value.is_string(),
            JasonType::Number => value.is_number(),
            JasonType::Bool => value.is_boolean(),
            JasonType::Null => value.is_null(),

            JasonType::List(inner) => {
                if let Value::Array(arr) = value {
                    arr.iter().all(|v| inner.matches(v))
                } else {
                    false
                }
            }

            JasonType::Union(types) => {
                types.iter().any(|t| t.matches(value))
            }

            JasonType::Object(map) => {
                if let Value::Object(obj) = value {
                    map.iter().all(|(key, jval_type)| {
                        obj.get(key).map_or(false, |v| jval_type.matches(v))
                    })
                } else {
                    false
                }
            }
        }
    }
}

use std::fmt;

impl fmt::Display for JasonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JasonType::String => write!(f, "String"),
            JasonType::Number => write!(f, "Number"),
            JasonType::Bool   => write!(f, "Bool"),
            JasonType::Null   => write!(f, "Null"),
            JasonType::Any    => write!(f, "Any"),

            JasonType::List(inner) => {
                write!(f, "[{}]", inner)
            }

            JasonType::Union(types) => {
                let mut first = true;
                for t in types {
                    if !first {
                        write!(f, " | ")?;
                    }
                    write!(f, "{t}")?;
                    first = false;
                }
                Ok(())
            }

            JasonType::Object(map) => {
                write!(f, "{{")?;
                let mut first = true;

                for (key, value) in map {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                    first = false;
                }

                write!(f, "}}")
            }
        }
    }
}



