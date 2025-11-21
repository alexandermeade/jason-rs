use std::fs;
use crate::jason_errors::{JasonError, JasonErrorKind};
use crate::{context::Context, lexer, parser};
use crate::lua_instance::LuaInstance;
use std::rc::Rc;
use std::cell::RefCell;
use crate::jason::CompilerResult;

pub fn jason_context_from_src(src: &str, lua: Rc<RefCell<LuaInstance>>) -> CompilerResult<Context> {
    let file_path: Rc<String> = Rc::new("direct source".to_string());
    let toks = lexer::Lexer::start(file_path.clone(), src.to_string())?;
    let nodes = parser::Parser::start(file_path.clone(), toks)?;
    let mut context = match Context::new(file_path.clone(), lua) {
        Ok(ctx) => ctx,
        Err(_) => return Err(JasonError::new(JasonErrorKind::ContextError, file_path, None, "failed to build context")),
    };
    let mut errors: Vec<JasonError> = Vec::new();
    for node in nodes.iter() {
        context.set_local_root(node);
        if let Err(e) = context.to_json(&node) {
            errors.push(e);
        }
        context.clear_local_root();
    }
    if !errors.is_empty() {
        return Err(JasonError::new(JasonErrorKind::Bundle(errors), context.source_path.clone(), None, "summary of errors"));
    }
    Ok(context)
}

pub fn jason_context_from_file(file_path: String, lua: Rc<RefCell<LuaInstance>>) -> CompilerResult<Context> {
    let file_path: Rc<String> = Rc::new(file_path);
    
    // Check file existence
    match fs::metadata(&**file_path) {
        Ok(_) => {},
        Err(e) => {
            return Err(if e.kind() == std::io::ErrorKind::NotFound {
                JasonError::new(JasonErrorKind::ImportError, file_path, None, format!("Path does not exist"))
            } else {
                JasonError::new(JasonErrorKind::Custom, file_path, None, format!("Unknown error: {:?}", e))
            });
        }
    }
    
    // Read file
    let src = match fs::read_to_string(&**file_path) {
        Ok(s) => s,
        Err(_) => {
            return Err(JasonError::new(JasonErrorKind::FileError, file_path, None, "Failed to read file"))
        }
    };
    
    // Parse
    let toks = lexer::Lexer::start(file_path.clone(), src)?;
    let nodes = parser::Parser::start(file_path.clone(), toks)?;
    
    // Create context - now takes Rc<String> directly
    let mut context = match Context::new(file_path.clone(), lua) {
        Ok(ctx) => ctx,
        Err(_) => return Err(JasonError::new(JasonErrorKind::ContextError, file_path, None, "failed to build context")),
    };
    
    let mut errors: Vec<JasonError> = Vec::new();
    for node in nodes.iter() {
        context.set_local_root(node);
        if let Err(e) = context.to_json(&node) {
            errors.push(e);
        }
        context.clear_local_root();
    }
    if !errors.is_empty() {
        return Err(JasonError::new(JasonErrorKind::Bundle(errors), context.source_path.clone(), None, "summary of errors"));
    }
    Ok(context)
}

pub fn compile_jason_from_file(file_path: &str, lua: Rc<RefCell<LuaInstance>>) -> CompilerResult<serde_json::Value> {
    let context = jason_context_from_file(file_path.to_string(), lua)?;
    Ok(context.out)
}

pub fn compile_jason_from_src(src: &str, lua: Rc<RefCell<LuaInstance>>) -> CompilerResult<serde_json::Value> {
    let context = jason_context_from_src(src, lua)?;
    Ok(context.out)
}
