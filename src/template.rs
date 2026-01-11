use crate::astnode::ASTNode;
use crate::context::Context;
use crate::jason_errors::JasonError;
use crate::jason_errors::JasonErrorKind;
use crate::jason_errors::JasonResult;
use crate::jason_types::JasonType;
use crate::token;
use crate::token::Token;
use crate::jason_errors;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Template {
    name: String,
    arguments: Vec<String>,
    block: token::Args,
    typing: Option<(Vec<JasonType>, JasonType)>
}

impl Template { 
    pub fn new(context: &Context, name: String, arguments: Vec<String>, block: token::Args, typing: Option<(Vec<JasonType>, JasonType)>) -> JasonResult<Self> {
        
        for node in &block {
            Self::check_self_reference(&context, &node, &node, &name)?;
        }

        Ok(Self { name, arguments, block, typing })
    }
    
    pub fn resolve(&self, context: &mut Context, arguments: &token::Args) -> jason_errors::JasonResult<Option<serde_json::Value>> {
        // Parse arguments into proper AST nodes first 
        let parsed_args = arguments;
        
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
        let block_node = &ASTNode::new(
            Token::new(token::TokenType::Block(self.block.clone()), "block".to_string(), 1, 1)
        );
        let resolved_block = context.to_json(block_node)?.ok_or_else(|| 
            context.err(JasonErrorKind::ValueError, format!("failed to evaluate block"))
        )?;

        if !result_type.matches(&resolved_block) {
            let block_type = context.infer_type_from(&resolved_block)?;
             
            match (result_type, &block_type) {
                (JasonType::Object(o1), JasonType::Object(o2)) => {  
                    let diff = JasonType::diff_objects(&o1, &o2);
                    return Err(
                        context.err(JasonErrorKind::TypeError(block_node.token.plain()), format!("Template {} resulted in {} expected {}{}", self.name, block_type, result_type, diff))
                    )                    
                },
                _ => {}
            }

           
            return Err(
                context.err(JasonErrorKind::TypeError(block_node.token.plain()), format!("Template {} resulted in {} expected {}", self.name, block_type, result_type))
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


    pub fn check_self_reference(context: &Context, top_level: &ASTNode, node: &ASTNode, name: &str) -> JasonResult<()> {
        if let token::TokenType::FnCall(_) = &node.token.token_type {
            if node.token.plain() == name {
                return Err(
                    JasonError::new(
                        JasonErrorKind::TemplateRescursion(name.to_string()), 
                        context.source_path.clone(), 
                        Some(Rc::new(top_level.clone())), 
                        format!("Self reference found in Template") 
                    )
                ); 
            }
        }
        if let Some(left) = &node.left {
            Self::check_self_reference(context, top_level, left, name)?; 
        }
        if let Some(right) = &node.right {
            Self::check_self_reference(context, top_level, right, name)?;
        }
        Ok(())
    }

}
