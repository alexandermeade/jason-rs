use crate::{astnode::ASTNode, context::Context};

pub fn ast_to_context(nodes: Vec<ASTNode>) -> Context {
    Context::new("dummy path".into())
}


