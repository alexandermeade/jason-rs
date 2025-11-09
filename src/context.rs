use crate::lexer;
use crate::parser;
use crate::astnode::ASTNode;
use crate::template::Template;
use crate::token;
use crate::token::TokenType;
use regex::Regex;
use serde_json::Value;
use serde_json::json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error;
use std::fs;
use std::rc::Rc;

pub struct Context {
    templates: Vec<Template>,
}

impl Context {
    
    pub fn evaluate(&mut self, nodes: Vec<ASTNode>) {
        for node in nodes {
            //valid top level expressions
            match node.token.token_type {
                TokenType::TemplateDef(args, block_args) => {
                                          
                },
                TokenType::Equals => {

                },
                TokenType::AS => {

                },
                TokenType::Out => {
                    
                },
                _ => {}
            }
        }
    }

    pub fn new() -> Self {
        Context { templates: Vec::new() }
    }
}


