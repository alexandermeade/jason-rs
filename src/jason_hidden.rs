use std::{fs, error};
use crate::{context::Context, lexer, parser};
use crate::lua_instance::LuaInstance;
use std::rc::Rc;
use std::cell::RefCell;


pub fn jason_context_from_src(src: &str, lua: Rc<RefCell<LuaInstance>>) -> Result<Context, Box<dyn error::Error>> {
    let toks = lexer::Lexer::start(src.to_string());

    let nodes = parser::Parser::start(toks);
        
    let mut context = Context::new("direct source".to_string(), lua)?;

    for (_i, node) in nodes.iter().enumerate() {
        context.to_json(&node);
    }
    
    Ok(context) 
}



pub fn compile_jason_from_file(file_path: &str, lua:Rc<RefCell<LuaInstance>>) -> Result<serde_json::Value, Box<dyn error::Error>> { 
    match fs::metadata(file_path) {
        Ok(_) => {},
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(format!("Path does not exist. {}", file_path).into());
            } else {

                return Err(e.into());
            }
        }
    }

    let context = jason_context_from_file(file_path.into(), lua).unwrap();
    Ok(context.out)
}

pub fn compile_jason_from_src(src: &str, lua:Rc<RefCell<LuaInstance>>) -> Result<serde_json::Value, Box<dyn error::Error>> { 
    let context = jason_context_from_src(src, lua).unwrap();
    Ok(context.out)
}




pub fn jason_context_from_file(file_path: String, lua: Rc<RefCell<LuaInstance>>) -> Result<Context, Box<dyn error::Error>> {
    match fs::metadata(file_path.clone()) {
        Ok(_) => {},
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(format!("Path does not exist. {}", file_path).into());
            } else {

                return Err(e.into());
            }
        }
    }

    let src = fs::read_to_string(file_path.clone()).unwrap();    
    let toks = lexer::Lexer::start(src);

    let nodes = parser::Parser::start(toks);
        
    let mut context = Context::new(file_path, lua)?;

    for (_i, node) in nodes.iter().enumerate() {
        context.to_json(&node);
    }
    
    Ok(context) 
}


