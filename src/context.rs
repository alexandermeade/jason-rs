use crate::{jason, astnode::ASTNode, template::Template, token::{TokenType, ArgsToNode}};
use std::collections::HashMap;
use serde_json::{Value, Number, Map};

#[derive(Debug)]
pub enum ExportType {
    Template(String, Template),
    Variable(String, serde_json::Value)
}

#[derive(Debug)]
pub struct Context {
    pub variables: HashMap<String, serde_json::Value>,
    pub templates: HashMap<String, Template>,
    pub out: serde_json::Value,
    pub source_path: String,
}

impl Context {    
    pub fn to_json(&mut self, node: &ASTNode) -> Option<serde_json::Value> {
        match &node.token.token_type {
            TokenType::ID => {
                if !self.variables.contains_key(&node.token.plain()) {
                    panic!("the variable {} does not exist in file {}", node.token.plain(), self.source_path);
                }
                Some(self.variables.get(&node.token.plain()).unwrap().clone())
            },
            TokenType::Block(_) => Some(self.block_to_json(node)),
            TokenType::NumberLiteral(num) => Some(serde_json::Value::Number(Number::from_f64(num.parse::<f64>().unwrap().into()).unwrap())),
            TokenType::Equals => {
                if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
                    let left = &left_node;
                    let right = &right_node;

                    if left.token.token_type != TokenType::ID {
                        panic!(
                            "[ERROR] {} = {}, variable name must be a valid identifier!",
                            left.token.plain(),
                            right.token.plain()
                        );
                    }
                    let right_value = self.to_json(right).unwrap();

                    self.variables.insert(left.token.plain(), right_value);
                    
                    
                }
                None
            },
            TokenType::StringLiteral(s) => Some(serde_json::Value::String(s.to_string())), 
            TokenType::List(args)=> Some(
                Value::Array(args.to_nodes()
                    .into_iter()
                    .map(|n| self.to_json(&n).unwrap())
                    .collect::<Vec<serde_json::Value>>())
            ),
            /*
            TokenType::Dot => {
                if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
                    let left = &left_node;
                    let right = &right_node;

                    if left.token.token_type != TokenType::ID || right.token.token_type != TokenType::ID{
                        panic!(
                            "[ERROR] {}.{}, accessor and identifier must be a valid identifier!",
                            left.token.plain(),
                            right.token.plain()
                        );
                    }
                    let right_value = self.to_json(right).unwrap();
                    self.variables.insert(left.token.plain(), right_value); 
                    return None
                }
                panic!("dot operator must have two identifiers with it. Context.Identifier");
            },
            */
            TokenType::From => {
                if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
                    if let TokenType::Import(args) = left_node.token.token_type.clone() {
                        if let TokenType::StringLiteral(import_path) = right_node.token.token_type.clone() {
                            let context = jason::jason_context_from_file(import_path.clone()).unwrap();
                            let args:Vec<String> = args.to_nodes().into_iter().map(|node| node.token.plain()).collect();
                            
                            let exports = context.export(args);
                            self.absorb_exports(exports);                            
                            return None;          
                        }
                        panic!("[ERROR] from statement must have a string path\n ... from \"<Path>\"") 
                    }
                }
                return None;
            },
            TokenType::Out => {
                if let Some(right_node) = node.right.as_ref() {
                    self.out = self.to_json(right_node).unwrap();
                    return None;
                }
                panic!("out statement must have valid jason expression.\n example: out \"Hello!\"");
            },
            TokenType::TemplateDef(args, block) => {
                let args = args.to_nodes();

                if args.len() > 0 {
                    let args:Vec<String> = args.into_iter().map(|node| node.token.plain()).collect();
                
                    self.templates.insert(node.token.plain(), Template::new(node.token.plain(), args, block.clone()));
                    return None;
                }

                self.templates.insert(node.token.plain(), Template::new(node.token.plain(), Vec::new(), block.clone()));
                return None;
            },
            TokenType::FnCall(args) => {
                
                if !self.templates.contains_key(&node.token.plain()) {
                    panic!("the template {} does not exist in file {}", node.token.plain(), self.source_path);
                }

                let template = self.templates.get(&node.token.plain()).unwrap().clone();
                template.resolve(self, args.to_vec())
            },
            token => {
                panic!("Unknown token: {:?}", token)    
            }
        }
    }

    fn block_to_json(&mut self, node: &ASTNode) -> serde_json::Value {
        if let TokenType::Block(args) = &node.token.token_type {
            let nodes = args.to_nodes();

            let mut map = Map::new(); // this will become our JSON object

            for node in nodes {
                if node.token.token_type == TokenType::Colon {
                    let key_node = node.left.as_ref().expect("Missing key");
                    let value_node = node.right.as_ref().expect("Missing value");

                    if key_node.token.token_type != TokenType::ID {
                        panic!("Key must be an ID");
                    }

                    let key = key_node.token.plain();
                    let value = self.to_json(&*value_node); // recursive call

                    map.insert(key, value.unwrap());
                    continue;
                }
                panic!("values must adheere to <key : value> fields in blocks")
            }

            Value::Object(map)
        } else {
            panic!("block_to_json called on non-block token");
        }
    }

    pub fn add_var(&mut self, key: String, value: serde_json::Value) {
        self.variables.insert(key, value);
    }

    pub fn remove_var(&mut self, key: String) {
        self.variables.remove(&key);
    }

    pub fn new(path: String) -> Self {
        Context { variables: HashMap::new(), templates: HashMap::new(), out: Value::Null, source_path: path }
    }

    pub fn export(&self, args: Vec<String>) -> Vec<ExportType> {
        let mut exported_values:Vec<ExportType> = Vec::new(); 

        for arg in &args {
            if self.variables.contains_key(arg) {
                let variable = self.variables.get(arg).unwrap().clone();
                exported_values.push(ExportType::Variable(arg.clone(), variable));
                continue;
            }

            if self.templates.contains_key(arg) {
                let template = self.templates.get(arg).unwrap().clone();
                exported_values.push(ExportType::Template(arg.clone(), template));
                continue;
            }
            panic!("{} is not exported from file {}", arg, self.source_path);
        }

        exported_values
    }

    pub fn absorb_exports(&mut self, exports: Vec<ExportType>) {
        for exp in exports {
            match exp {
                ExportType::Template(name, template) => {
                    self.templates.insert(name, template);
                }

                ExportType::Variable(name, variable) => {
                    self.variables.insert(name, variable);
                }
            }
        }
    }
}


