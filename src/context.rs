use crate::lexer;
use crate::parser;
use crate::astnode::ASTNode;
use crate::template::Template;
use crate::token;
use crate::token::TokenType;
use regex::Regex;
use serde_json::json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error;
use std::fs;
use std::rc::Rc;
use serde_json::{Value, Number};

use crate::token::ArgsToNode;

use serde_json::Map;

#[derive(Debug)]
pub struct Context {
    pub variables: HashMap<String, serde_json::Value>,
    pub templates: HashMap<String, Template>,
}

impl Context {
    
    pub fn to_json(&mut self, node: &ASTNode) -> Option<serde_json::Value> {
        match &node.token.token_type {
            TokenType::ID => { 
                Some(self.variables.get(&node.token.plain()).unwrap().clone())
            },
            TokenType::Block(_) => Some(self.block_to_json(node)),
            TokenType::NumberLiteral(num) => Some(serde_json::Value::Number(Number::from_f64(num.parse::<f64>().unwrap().into()).unwrap())),
            TokenType::StringLiteral(s) => Some(serde_json::Value::String(s.to_string())), 
            TokenType::TemplateDef(args, block) => {
                let args:Vec<String> = args.into_iter().map(|v| v.get(0).unwrap().plain()).collect();
                self.templates.insert(node.token.plain(), Template::new(node.token.plain(), args, block.clone()));
                None
            },
            TokenType::FnCall(args) => {
                let template = self.templates.get(&node.token.plain()).unwrap().clone();
                template.resolve(self, args.to_vec())
            },
            tok => {
                //println!("[ERROR] to_json not implemented for tokentype {:?}", tok);
                Some(serde_json::Value::String(tok.name()))
            }
        }
    }

    fn block_to_json(&mut self, node: &ASTNode) -> serde_json::Value {
        if let TokenType::Block(args) = &node.token.token_type {
            let nodes = args.to_nodes();

            let mut map = Map::new(); // this will become our JSON object

            for node in nodes {
                if node.token.token_type == TokenType::Colon {
                    let key_node = node.left.as_ref().expect("Missing key");
                    let value_node = node.right.as_ref().expect("Missing value");

                    if key_node.token.token_type != TokenType::ID {
                        panic!("Key must be an ID");
                    }

                    let key = key_node.token.plain();
                    let value = self.to_json(&*value_node); // recursive call

                    map.insert(key, value.unwrap());
                    continue;
                }
                panic!("values must adheere to <key : value> fields in blocks")
            }

            Value::Object(map)
        } else {
            panic!("block_to_json called on non-block token");
        }
    }

    pub fn add_var(&mut self, key: String, value: serde_json::Value) {
        self.variables.insert(key, value);
    }

    pub fn remove_var(&mut self, key: String) {
        self.variables.remove(&key);
    }

    pub fn new() -> Self {
        Context { variables: HashMap::new(), templates: HashMap::new() }
    }
}


