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

pub struct Template {
    arguments: HashMap<String, String>,
}

impl Template { 
    pub fn new(keys: Vec<String>, values: Vec<String>) -> Self {
        let arguments = keys.into_iter()
            .zip(values.into_iter())
            .collect::<HashMap<_, _>>();

        Self { arguments }
    }

    pub fn resolve(args: Vec<String>) {

    } 
}
