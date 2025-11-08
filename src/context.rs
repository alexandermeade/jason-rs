use crate::lexer;
use crate::parser;
use crate::token;
use regex::Regex;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error;
use std::fs;
use std::rc::Rc;

pub struct GlobalContext {
    contexts: HashMap<String, Context>,
}

impl GlobalContext {
    pub fn new() -> Self {
        GlobalContext { contexts: HashMap::new() }
    }
}

pub struct Context {
    variables: HashMap<String, String>,
}

impl Context {
    
}

