use std::collections::{BTreeMap, HashSet};
use std::collections::hash_map::HashMap;
use serde_json::{Number, Value};
use crate::astnode::ASTNode;
use crate::jason_errors::{JasonError, JasonErrorKind, JasonResult};
use crate::context::Context;
use crate::token::TokenType;

//(a, b),[a, b), (a, b], [a, b]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum InfOrNum {
    Infinity,
    Num(Number)
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct JasonInterval {
    min: InfOrNum,
    min_included: bool,
    max: InfOrNum,
    max_included: bool,
}

impl JasonInterval {
    pub fn new(min: InfOrNum, min_included: bool, max: InfOrNum, max_included: bool) -> Self {
        let min_inf = min == InfOrNum::Infinity;
        let max_inf = max == InfOrNum::Infinity;

        Self {
            min,
            min_included: if min_inf {false} else {min_included},
            max,
            max_included: if max_inf {false} else {max_included},
        }
    }

    pub fn from_type(context: &Context, token_type: &TokenType, num: Number) -> JasonResult<Self> {
        match &token_type {
            TokenType::GreaterThan => Ok(JasonInterval::new(InfOrNum::Num(num), false, InfOrNum::Infinity, false)),
            TokenType::GreaterThanEqualTo => Ok(JasonInterval::new(InfOrNum::Num(num), true, InfOrNum::Infinity, false)),
            TokenType::LessThan => Ok(JasonInterval::new(InfOrNum::Infinity, false, InfOrNum::Num(num), false)),
            TokenType::LessThanEqualTo => Ok(JasonInterval::new(InfOrNum::Infinity, false, InfOrNum::Num(num), true)),
            _ => Err(context.err(JasonErrorKind::TypeError(">".to_string()), format!("can't build type of interval from non interval type {:?}", token_type)))
        }
    }
    
    pub fn min<'a>(&'a self, context: &Context, interval: &'a Self) -> JasonResult<&'a Self> {
        // Returns the interval with the LARGER minimum (more restrictive)
        if self.min == InfOrNum::Infinity {
            return Ok(interval); // interval's min is larger (finite > -infinity)
        }
        if interval.min == InfOrNum::Infinity {
            return Ok(self); // self's min is larger
        }
        match (&self.min, &interval.min) {
            (InfOrNum::Num(n1), InfOrNum::Num(n2)) => {
                let n1 = n1.as_f64();
                let n2 = n2.as_f64();
                if let (Some(n1), Some(n2)) = (&n1, &n2) {
                    if n1 > n2 { // CHANGED: > instead of 
                        return Ok(self); 
                    }
                    
                    if n1 < n2 { // CHANGED: < instead of >
                        return Ok(interval); 
                    }
                    if n1 == n2 {
                        return match (&self.min_included, &interval.min_included) {
                            (true, true) => Ok(self),
                            (true, false) => Ok(interval), // (a, ...) is more restrictive than [a, ...)
                            (false, true) => Ok(self),
                            (false, false) => Ok(self),
                        };
                    }
                    return Ok(interval);
                }
                return Err(context.err(JasonErrorKind::ValueError, format!("failed to build min comparison")));
            },
            _ => unreachable!() // Already handled infinity cases above
        }
    }

    pub fn max<'a>(&'a self, context: &Context, interval: &'a Self) -> JasonResult<&'a Self> {
        // Returns the interval with the SMALLER maximum (more restrictive)
        if self.max == InfOrNum::Infinity {
            return Ok(interval); // interval's max is smaller
        }
        if interval.max == InfOrNum::Infinity {
            return Ok(self); // self's max is smaller
        }
        match (&self.max, &interval.max) {
            (InfOrNum::Num(n1), InfOrNum::Num(n2)) => {
                if let (Some(n1), Some(n2)) = (&n1.as_f64(), &n2.as_f64()) {
                    if n1 < n2 { // CHANGED: < instead of >
                        return Ok(self); 
                    }
                    if n1 > n2 { // CHANGED: > instead of 
                        return Ok(interval); 
                    }
                    if n1 == n2 {
                        return match (&self.max_included, &interval.max_included) {
                            (true, true) => Ok(self),
                            (true, false) => Ok(interval), // (..., a) is more restrictive than (..., a]
                            (false, true) => Ok(self),
                            (false, false) => Ok(self),
                        };
                    }
                    return Ok(interval);
                }
                return Err(context.err(JasonErrorKind::ValueError, format!("failed to build max comparison")));
            },
            _ => unreachable!()
        }
    }

    pub fn combine(&self, context: &Context, int2: &Self) -> JasonResult<Self> {
        let tighter_min_interval = self.min(context, &int2)?;
        let tighter_max_interval = self.max(context, &int2)?;
        
        let result_min = &tighter_min_interval.min;
        let result_min_included = tighter_min_interval.min_included;
        
        let result_max = &tighter_max_interval.max;
        let result_max_included = tighter_max_interval.max_included;
        
        match (result_min, result_max) {
            (InfOrNum::Num(min_num), InfOrNum::Num(max_num)) => {
                let min_val = min_num.as_f64().ok_or_else(|| 
                    context.err(JasonErrorKind::ValueError, "Failed to convert min to f64".to_string())
                )?;
                let max_val = max_num.as_f64().ok_or_else(|| 
                    context.err(JasonErrorKind::ValueError, "Failed to convert max to f64".to_string())
                )?;
                
                if min_val > max_val {
                    return Err(context.err(
                        JasonErrorKind::ValueError, 
                        format!("Empty interval: min ({}) > max ({})", min_val, max_val)
                    ));
                }
                
                if min_val == max_val && (!result_min_included || !result_max_included) {
                    return Err(context.err(
                        JasonErrorKind::ValueError, 
                        format!("Empty interval: bounds meet at {} but not both inclusive", min_val)
                    ));
                }
            },
            _ => {}
        }
        
        Ok(JasonInterval::new(
            result_min.clone(), 
            result_min_included, 
            result_max.clone(), 
            result_max_included
        ))
    }
    pub fn contains(&self, n: f64) -> bool {
        let min_check = match &self.min {
            InfOrNum::Num(num) => {
                let min = match num.as_f64() {
                    Some(v) => v,
                    None => return false,
                };

                if self.min_included {
                    n >= min
                } else {
                    n > min
                }
            }
            InfOrNum::Infinity => true,
        };

        let max_check = match &self.max {
            InfOrNum::Num(num) => {
                let max = match num.as_f64() {
                    Some(v) => v,
                    None => return false,
                };

                if self.max_included {
                    n <= max
                } else {
                    n < max
                }
            }
            InfOrNum::Infinity => true,
        };

        min_check && max_check
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum JasonType {
    String,
    Number,
    Int,
    Float,
    Bool,
    Null,
    Any,
    NumberLiteral(Number),
    StringLiteral(String),
    Interval(JasonInterval),
    Union(Vec<Box<JasonType>>),
    List(Box<JasonType>),
    Object(BTreeMap<String, JasonType>),
    Variance(Box<JasonType>),
}

impl JasonType {
    fn merge(o1: JasonType, o2: JasonType) -> JasonResult<JasonType> {
        
        let mut result:BTreeMap<String, JasonType> = BTreeMap::new();
        if let (JasonType::Object(obj1), JasonType::Object(obj2)) = (&o1, &o2) {
            for (k, v) in obj1 {
                // if obj1 doesn't contain a key in obj2 add key to obj 1
                if !obj2.contains_key(k) {
                    result.insert(k.clone(), v.clone());
                }     

                if obj2.contains_key(k) {
                    result.insert(k.clone(), Self::merge(v.clone(), obj2.get(k).unwrap().clone())?);
                }
            }

            for (k, v) in obj2 {
                if !obj1.contains_key(k) {
                    result.insert(k.clone(), v.clone());
                }
            }
        } else {
            return Ok(JasonType::Union(vec![Box::new(o1), Box::new(o2)]));
        }
            
        Ok(JasonType::Object(result))
    }

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

            TokenType::FloatLiteral(n) => Ok(JasonType::NumberLiteral(Number::from_f64(*n).ok_or_else(||
                self.err(JasonErrorKind::ConversionError, format!("failed to covert {} into NumberLiteralType", *n))
            )?)),
            TokenType::IntLiteral(n) => Ok(JasonType::NumberLiteral(Number::from(*n))),
            TokenType::StringLiteral(s) => Ok(JasonType::StringLiteral(s.clone())),
            TokenType::StringType   => Ok(JasonType::String),
            TokenType::IntType      => Ok(JasonType::Int),
            TokenType::FloatType    => Ok(JasonType::Float),
            TokenType::NumberType   => Ok(JasonType::Number),
            TokenType::BoolType     => Ok(JasonType::Bool),
            TokenType::AnyType      => Ok(JasonType::Any),
            TokenType::NullType     => Ok(JasonType::Null),
            TokenType::GreaterThan        |
            TokenType::LessThan           |
            TokenType::GreaterThanEqualTo |
            TokenType::LessThanEqualTo  => { 
                let right = node.right.as_ref().ok_or_else(||
                    self.err(
                        JasonErrorKind::MissingValue,
                        format!("missing right side of bar expression")
                    )
                )?;
                
                match &right.token.token_type {
                    TokenType::FloatLiteral(f)  => {
                        let num = Number::from_f64(*f).ok_or_else(|| 
                            self.err(JasonErrorKind::ConversionError, format!("failed to convert {} to Number", f))
                        )?;
                        Ok(JasonType::Interval(JasonInterval::from_type(self, &node.token.token_type, num)?))
                    },
                    TokenType::IntLiteral(n)  => {
                        let num = Number::from(*n);
                        Ok(JasonType::Interval(JasonInterval::from_type(self, &node.token.token_type, num)?))
                    },
                    _ => {
                        return Err(self.err(JasonErrorKind::TypeError(right.plain_sum.clone()), format!("{} must be of type Int or Float", right.plain_sum)))
                    }
                }
            },

            TokenType::While        => {
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

                let left_type = self.to_type(left)?;
                let right_type = self.to_type(right)?;

                match (&left_type, &right_type) {
                    (JasonType::Interval(int1), JasonType::Interval(int2)) => {
                        let new_int = int1.combine(&self, int2)?;

                        Ok(JasonType::Interval(new_int))
                    },
                    _ => {
                        //println!("{:?}, {:?}", left_type, right_type);
                        Err(self.err(JasonErrorKind::TypeError(self.local_root.clone().unwrap().plain_sum.clone()), format!("can only apply while statements to Interval types. I.E. Interval1 while Interval2")))
                    }
                }
            },
            TokenType::With         => {
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

                let left_type = self.to_type(left)?;
                let right_type = self.to_type(right)?;

                fn with_helper(obj: BTreeMap<String, JasonType>, replacement_type: JasonType) -> BTreeMap<String, JasonType> {
                    let mut result: BTreeMap<String, JasonType> = BTreeMap::new();

                    for (k, v) in obj {
                        if let JasonType::Object(inner_obj) = v {
                            let inner_type = with_helper(inner_obj.clone(), replacement_type.clone());
                            result.insert(k.clone(), JasonType::Object(inner_type));
                            continue;
                        }

                        result.insert(k.clone(), replacement_type.clone()); 
                    }

                    return result;
                }

                match (&left_type, &right_type) {
                    (JasonType::Object(o1), v) => {
                        Ok(JasonType::Object(with_helper(o1.clone(), v.clone())))
                    },
                    _ => {
                        Err(
                            JasonError::new(
                                JasonErrorKind::TypeError("*ALL*".to_string()),
                                self.source_path.clone(),
                                self.local_root.clone(), 
                                format!("you can 'with' types of type Object I.E. {{key: value, ...}} with T")
                            )
                        )
                    }
                }
            },
            TokenType::VarianceOperator     => {
                //backtick is post fix so you just need left
                let left = node.left.as_ref().ok_or_else(||
                    self.err(
                        JasonErrorKind::MissingValue,
                        format!("missing left side of bar expression")
                    )
                )?;
                 
                Ok(JasonType::Variance(Box::new(self.to_type(&left)?)))
            }, 
            TokenType::Merge         => {
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

                let left_type = self.to_type(left)?;
                let right_type = self.to_type(right)?;


                match (&left_type, &right_type) {
                    (JasonType::Object(_), JasonType::Object(_)) => {
                        let result = JasonType::merge(left_type, right_type)?;
                        Ok(result)
                    },
                    (_,  JasonType::Object(_)) => {
                        Err(
                            JasonError::new(
                                JasonErrorKind::TypeError(left.to_code()),
                                self.source_path.clone(),
                                self.local_root.clone(), 
                                format!("you can only merge types of type Object I.E. {{key: value, ...}}")
                            )
                        )
                    },
                    (JasonType::Object(_), _) => {
                        Err(
                            JasonError::new(
                                JasonErrorKind::TypeError(right.to_code()),
                                self.source_path.clone(),
                                self.local_root.clone(), 
                                format!("you can only merge types of type Object I.E. {{key: value, ...}}")
                            )
                        )
                    },
                    _ => {
                        Err(
                            JasonError::new(
                                JasonErrorKind::TypeError("*ALL*".to_string()),
                                self.source_path.clone(),
                                self.local_root.clone(), 
                                format!("you can only concat types of type Object I.E. {{key: value, ...}}")
                            )
                        )
                    }
                }

            },



            TokenType::Plus         => {
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
                
                match (self.to_type(left)?, self.to_type(right)?) {
                    (JasonType::Object(o1), JasonType::Object(o2)) => {
                        let mut new_object = o1.clone();
                        new_object.extend(o2);
                        Ok(JasonType::Object(new_object))
                    },
                    (_,  JasonType::Object(_)) => {
                        Err(
                            JasonError::new(
                                JasonErrorKind::TypeError(left.to_code()),
                                self.source_path.clone(),
                                self.local_root.clone(), 
                                format!("you can only concat types of type Object I.E. {{key: value, ...}}")
                            )
                        )
                    },
                    (JasonType::Object(_), _) => {
                        Err(
                            JasonError::new(
                                JasonErrorKind::TypeError(right.to_code()),
                                self.source_path.clone(),
                                self.local_root.clone(), 
                                format!("you can only concat types of type Object I.E. {{key: value, ...}}")
                            )
                        )
                    },
                    _ => {
                        Err(
                            JasonError::new(
                                JasonErrorKind::TypeError("*ALL*".to_string()),
                                self.source_path.clone(),
                                self.local_root.clone(), 
                                format!("you can only concat types of type Object I.E. {{key: value, ...}}")
                            )
                        )
                    }
                }

            },

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
                let nodes = args;
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
                for node in values {
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
            Value::Number(n) => {
                if n.is_i64() {
                    return Ok(JasonType::Int)
                }
                return Ok(JasonType::Float)
            }
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
            JasonType::Int => value.is_i64(),
            JasonType::Float => value.is_f64(),
            JasonType::Bool => value.is_boolean(),
            JasonType::Null => value.is_null(),

            JasonType::Variance(var_obj) => {
                if let (Value::Object(obj), JasonType::Object(vobj) ) = (value, &**var_obj) {
                    // Check all expected keys exist and match
                    for (key, jval_type) in vobj {
                        match obj.get(key) {
                            Some(v) => if jval_type.matches(v) {return true},    
                            None => {}                        
                        }
                    }
                    return false
                } 
                false 
            },

            JasonType::NumberLiteral(n) => { 
                if !value.is_number() {
                    return false;
                }

                if let Value::Number(num) = value { 
                    return n.eq(num);    
                }

                return false;
            }, 

            JasonType::StringLiteral(s) => {
                if !value.is_string() {
                    return false;
                }

                if let Value::String(value_string) = value {
                    return s == value_string;
                }
                return false;
            },

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
                    // Check all expected keys exist and match
                    for (key, jval_type) in map {
                        match obj.get(key) {
                            Some(v) if jval_type.matches(v) => {},
                            Some(_) => return false, // type mismatch
                            None => return false,    // missing key
                        }
                    }

                    for key in obj.keys() {
                        if !map.contains_key(key) {
                            return false;
                        }
                    }

                    true
                } else {
                    false
                }
            }

            JasonType::Interval(interval) => {
                if value.is_number() {
                    if let Some(n) = value.as_f64() {
                        return interval.contains(n);
                    }
                }
                false
            }
        }
    }

    pub fn diff_objects(expected: &BTreeMap<String, JasonType>, found: &BTreeMap<String, JasonType>) -> String {
        let mut result = String::new();
        
        let mut missing_keys: Vec<&String> = expected.keys()
            .filter(|k| !found.contains_key(*k))
            .collect();
        missing_keys.sort();
        
        let mut extra_keys: Vec<&String> = found.keys()
            .filter(|k| !expected.contains_key(*k))
            .collect();
        extra_keys.sort();
        
        let mut different_types: Vec<(&String, &JasonType, &JasonType)> = expected.iter()
            .filter_map(|(k, v)| {
                found.get(k).and_then(|found_v| {
                    if v != found_v {
                        Some((k, v, found_v))
                    } else {
                        None
                    }
                })
            })
            .collect();
        different_types.sort_by_key(|(k, _, _)| *k);
        
        if !missing_keys.is_empty() {
            result.push_str("\n  Missing fields:\n");
            for key in missing_keys {
                result.push_str(&format!("    - {}: {}\n", key, expected.get(key).unwrap()));
            }
        }
        
        if !extra_keys.is_empty() {
            result.push_str("\n  Extra fields:\n");
            for key in extra_keys {
                result.push_str(&format!("    + {}: {}\n", key, found.get(key).unwrap()));
            }
        }
        
        if !different_types.is_empty() {
            result.push_str("\n  Type mismatches:\n");
            for (key, expected_type, found_type) in different_types {
                result.push_str(&format!("    ~ {}: expected {}, found {}\n", key, expected_type, found_type));
            }
        }
        
        if result.is_empty() {
            result.push_str("  (no differences)");
        }
        
        result
    }

}

use std::fmt;

impl fmt::Display for InfOrNum {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfOrNum::Infinity => write!(f, "Infinity"),
            InfOrNum::Num(num) => {
                if num.is_i64() {
                    let num_value = num.as_i64().unwrap();
                    return write!(f, "{}", num_value.to_string())
                } 
                if num.is_f64() {
                    let num_value = num.as_f64().unwrap();
                    return write!(f, "{}", num_value.to_string())
                }
                return write!(f, "")
            },
        }
    }
}

impl fmt::Display for JasonInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{},{}{}", 
            if self.min_included {"["} else {"("},
            format!("{}", if self.min == InfOrNum::Infinity {"-Infinity".to_string()} else {format!("{}", self.min)}),
            format!("{}", self.max),

            if self.max_included {"]"} else {")"}
        )

    }
}

impl fmt::Display for JasonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JasonType::String => write!(f, "String"),
            JasonType::Number => write!(f, "Number"),
            JasonType::Int => write!(f, "Int"),
            JasonType::Float => write!(f, "Float"),
            JasonType::Bool   => write!(f, "Bool"),
            JasonType::Null   => write!(f, "Null"),
            JasonType::Any    => write!(f, "Any"),
            JasonType::StringLiteral(s) => write!(f, "\"{}\"", s),
            JasonType::NumberLiteral(n) => write!(f, "\"{}\"", n),
            JasonType::Interval(interval) => write!(f, "{}", interval), 
            JasonType::List(inner) => {
                write!(f, "[{}]", inner)
            }

            JasonType::Variance(value) => write!(f, "{}'", value),
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



