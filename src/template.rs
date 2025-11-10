use crate::astnode::ASTNode;
use crate::context::Context;
use crate::token;
use crate::token::Token;
use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct Template {
    name: String,
    arguments: Vec<String>,
    block: token::Args,
}

impl Template { 
    pub fn new(name: String, arguments: Vec<String>, block: token::Args) -> Self {
        Self { name, arguments, block }
    }

    pub fn arguments_count(&self) -> usize {
        self.arguments.len()
    }

    pub fn resolve(&self, context: &mut Context, arguments: token::Args) -> Option<serde_json::Value> {
        // Build map of parameter name -> argument value
        let args: HashMap<String, serde_json::Value> = self
            .arguments
            .clone()
            .into_iter()
            .zip(
                arguments
                    .into_iter()
                    .flatten()
                    .map(|tok| context.to_json(&ASTNode::new(tok))),
            )
            .filter_map(|(k, v)| v.map(|val| (k, val)))
            .collect();

        // Save original variable state for keys we're overwriting
        let mut old_values = HashMap::new();
        for key in args.keys() {
            if let Some(val) = context.variables.get(key).cloned() {
                old_values.insert(key.clone(), val);
            }
            context.add_var(key.clone(), args[key].clone());
        }

        // Evaluate block
        let resolved_block = context.to_json(&ASTNode::new(
            Token::new(token::TokenType::Block(self.block.clone()), "block".to_string(), 1, 1)
        ));

        // Restore old values
        for key in args.keys() {
            if let Some(old_val) = old_values.remove(key) {
                context.add_var(key.clone(), old_val);
            } else {
                context.remove_var(key.clone());
            }
        }

        resolved_block
    }
}
