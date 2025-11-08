use crate::parser::ASTNode;
use crate::context::{Context, GlobalContext}; 

pub fn ast_to_json(nodes: Vec<ASTNode>) -> GlobalContext {
    GlobalContext::new()            
}
