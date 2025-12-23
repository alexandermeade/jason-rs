use crate::astnode::ASTNode;
use crate::context::Context;
use crate::jason_errors::JasonErrorKind;
use crate::jason_types::JasonType;
use crate::token;
use crate::token::Token;
use crate::token::ArgsToNode;
use crate::jason_errors;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Template {
    name: String,
    arguments: Vec<String>,
    block: token::Args,
    typing: Option<(Vec<JasonType>, JasonType)>
}

impl Template { 
    pub fn new(name: String, arguments: Vec<String>, block: token::Args, typing: Option<(Vec<JasonType>, JasonType)>) -> Self {
        Self { name, arguments, block, typing }
    }
    
    pub fn resolve(&self, context: &mut Context, arguments: token::Args) -> jason_errors::JasonResult<Option<serde_json::Value>> {
        // Parse arguments into proper AST nodes first
        let parsed_args = arguments.to_nodes()?;
        
        // Build map of parameter name -> argument value
        let args: Vec<(String, serde_json::Value)> = self
            .arguments
            .iter()
            .cloned()
            .zip(
                parsed_args
                    .iter()
                    .map(|node| context.to_json(node))
                    .collect::<jason_errors::JasonResult<Vec<_>>>()?
            )
            .filter_map(|(k, v)| v.map(|val| (k, val)))
            .collect();
            
        // Save original variable state for keys we're overwriting
        let mut old_values:HashMap<String, (serde_json::Value, JasonType)> = HashMap::new();

        let (param_types, result_type) = if let Some((param_types, result_type)) = &self.typing {
            (param_types, result_type)
        } else {
            (&vec![JasonType::Any; args.len()], &JasonType::Any)
        };

        for (i, (key, value)) in args.iter().enumerate() {
            if let Some(val) = context.variables.get(key).cloned() {
                let typed_val = context.variable_types.get(key).unwrap_or(&JasonType::Any);
                old_values.insert(key.clone(), (val, typed_val.clone()));
            }

            let typed_param = param_types.get(i).unwrap().clone();

            if !typed_param.matches(value) {
                let infered_type = context.infer_type_from(value)?;
                return Err(context.err(
                    JasonErrorKind::TypeError(key.clone()),
                    format!(
                        "expected type {} for {} found {} in template {}",
                        typed_param, key, infered_type, self.name
                    ),
                ));
            }

            context.add_var(key.clone(), value.clone(), typed_param);
        }
                
        // Evaluate block
        let resolved_block = context.to_json(&ASTNode::new(
            Token::new(token::TokenType::Block(self.block.clone()), "block".to_string(), 1, 1)
        ))?.ok_or_else(|| 
            context.err(JasonErrorKind::ValueError, format!("failed to evaluate block"))
        )?;

        if !result_type.matches(&resolved_block) {
            let block_type = context.infer_type_from(&resolved_block)?;
            return Err(
                context.err(JasonErrorKind::Custom, format!("Template {} resulted in {} expected {}", self.name, result_type, block_type))
            )
        }
        
        // Restore old values
        for (key, _) in args {
            if let Some((old_val, type_val)) = old_values.remove(&key) {
                context.add_var(key.clone(), old_val, type_val);
            } else {
                context.remove_var(key.clone());
            }
        }
        
        Ok(Some(resolved_block))
    }
}
