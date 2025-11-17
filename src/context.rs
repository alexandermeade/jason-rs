use crate::{
    astnode::ASTNode, 
    lua_instance::LuaInstance, 
    template::Template, 
    token::{ArgsToNode, TokenType}
};
use std::collections::HashMap;
use mlua::Table;
use serde_json::{Value, Number, Map};
use std::rc::Rc;
use std::cell::RefCell;
use crate::jason_hidden;

#[allow(unused_imports)]
use crate::jason_errors;

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
    pub lua_instance: Rc<RefCell<LuaInstance>>,
    pub lua_env: Table,
    pub lua_fn_cache: HashMap<String, mlua::RegistryKey>, // cache lua functions
}

impl Context {    
    pub fn new(path: String, lua_instance: Rc<RefCell<LuaInstance>>) -> Result<Self, Box<dyn std::error::Error>> {
        let lua_env = {
            let lua_borrow = lua_instance.borrow();
            let lua_ref = &lua_borrow.lua_instance;
            let env = lua_ref.create_table()?;
            
            // Set up metatable to inherit from base_env (which inherits from globals)
            let mt = lua_ref.create_table()?;
            mt.set("__index", lua_borrow.base_env.clone())?;
            env.set_metatable(Some(mt));
            
            env
        };
                        
        Ok(Context {
            variables: HashMap::new(),
            templates: HashMap::new(),
            out: Value::Null,
            source_path: path,
            lua_instance,
            lua_env,
            lua_fn_cache: HashMap::new(),
        })
    }

    fn eval_mult(&mut self, node: &ASTNode) -> Option<serde_json::Value>{
        //left sided experations I.E. expression * n
        if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
            if let TokenType::NumberLiteral(num) = right_node.token.token_type.clone() {
                let count = num.parse().expect("failed to parse num");
                let mut result: Vec<serde_json::Value> = Vec::with_capacity(count);
                for _ in 0..count {
                    let value = self.to_json(left_node);
                    match value {
                        Some(v) => result.push(v),
                        None => panic!("failed here {:#?}",node),
                    };
                }
                return Some(Value::Array(result)) 
            }
            //right sided operations I.E.   n * expression
            if let TokenType::NumberLiteral(num) = left_node.token.token_type.clone() {
                let count = num.parse().expect("failed to parse num");
                let mut result: Vec<serde_json::Value> = Vec::with_capacity(count);
                for _ in 0..count {
                    let value = self.to_json(right_node).unwrap();
                    result.push(value);                        
                }
                
                return Some(Value::Array(result)) 
            }

        }
        panic!("Repeat failed {:#?}", node);
    }

    pub fn eval_equal(&mut self, node: &ASTNode) -> Option<serde_json::Value> {
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
    }

    pub fn eval_from(&mut self, node: &ASTNode) -> Option<serde_json::Value> {
        if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
            let mut import_path:String = "".to_string();
            let mut lua_import_path: String = "".to_string();
            match right_node.token.token_type.clone() {
                TokenType::StringLiteral(path) => {
                    import_path = path;
                },
                TokenType::ID => {
                    lua_import_path = right_node.token.plain();
                },
                _ => panic!("[ERROR] from statement must have a string path\n ... from \"<Path>\"")
            }
            match left_node.token.token_type.clone() {
                TokenType::Import(args) =>  { 
                    if import_path == "" {
                        panic!("to import templates/variable you must import from a string path I.E. import(...) from \"path/to/file\"");
                    }
                    let context = jason_hidden::jason_context_from_file(import_path.clone(), self.lua_instance.clone()).unwrap();
                    let args:Vec<String> = args.to_nodes().into_iter().map(|node| node.token.plain()).collect();
                    if args.contains(&"*".to_string()) {
                        let exports = context.export_all();
                        self.absorb_exports(exports);
                        return None;
                    } 

                    let exports = context.export(args);
                    self.absorb_exports(exports);                           
                    return None;
                }

                TokenType::Use(args) => {
                    if lua_import_path == "" {
                        panic!("to import lua functions you must derive from a plain component I.E. use(...) from std\n note how std std is plain text and not in qoutes");
                    }

                    let args:Vec<String> = args.to_nodes().into_iter().map(|node| node.token.plain()).collect();                                     
                    for arg in &args {
                        let _ = self.import_from_base(arg);
                    }
                    if args.contains(&"*".to_string()) {
                        return None;
                    } 
                    return None; 
                },
                _ => panic!("[ERROR] from statement must have a string path\n ... from \"<Path>\"")
            }
        }
        return None;
    }

    pub fn eval_lua_fn(&mut self, node:&ASTNode) -> Option<serde_json::Value> {
        if let TokenType::LuaFnCall(args) = &node.token.token_type {
            // Convert to JSON first
            let json_values: Vec<serde_json::Value> = args.to_nodes().iter()
                .map(|node| self.to_json(node).unwrap())
                .collect();

            // Now borrow lua
            let lua = self.lua_instance.borrow();

            // Convert to Lua values
            let lua_args: Vec<mlua::Value> = json_values.iter()
                .map(|value| lua.json_to_lua_value(value.clone()).unwrap())
                .collect();     
            let fn_name = node.token.plain();
            
            //retrieve function from cache or load functino into cache
            let func = if let Some(key) = self.lua_fn_cache.get(&fn_name) {
                //get function from cache
                lua.lua_instance.registry_value::<mlua::Function>(key).unwrap()
            } else {
                // load function from lua directly  
                let func: mlua::Function = lua.lua_instance
                    .load(&fn_name)
                    .set_environment(self.lua_env.clone())
                    .eval()
                    .expect(&format!("failed to load function {}", fn_name));
                
                // Store in registry for reuse
                let key = lua.lua_instance.create_registry_value(func.clone()).unwrap();
                drop(lua); // Drop borrow before mutable access
                self.lua_fn_cache.insert(fn_name.clone(), key);
                func
            };


            /*
            // Load and evaluate to get the function directly
            let func: mlua::Function = lua.lua_instance
                .load(node.token.plain())
                .set_environment(self.lua_env.clone())
                .eval()
                .expect(&format!("failed to load function {}", node.token.plain()));
            */
            // Call the function with arguments
            let result = 
                func.call::<mlua::MultiValue>(mlua::MultiValue::from_vec(lua_args)).unwrap();

            // Get the first value from the result
            let json_value: serde_json::Value = if let Some(first) = result.into_iter().next() {
                LuaInstance::lua_value_to_json(&first)
            } else {
                serde_json::Value::Null
            };
            return Some(json_value);
        }
        panic!("at eval_lua_fn token is not of luafncall")
    }
    
    pub fn eval_plus(&mut self, node:&ASTNode) -> Option<serde_json::Value> {
        let left = self.to_json(node.left.as_ref()?)?;
        let right = self.to_json(node.right.as_ref()?)?;

        match (left, right) {
            //[] + [] => [1,2,3] + [4,5] = [1,2,3,4,5]
            (Value::Array(mut a), Value::Array(b)) => {
                a.extend(b);
                Some(Value::Array(a))
            },
            //"..." + "..." => "hello" + " world" = "hello world"
            (Value::String(a), Value::String(b)) => {
                Some(Value::String(a + &b))
            }
            // {...} + {...} => {name: "Alex"} + {age:20} = {name:"Alex", age: 20} 
            (Value::Object(mut a), Value::Object(b)) => {
                a.extend(b);
                Some(Value::Object(a))
            }

            // Add other cases if needed:
            // (Value::Number(a), Value::Number(b)) => ...
            
            other => panic!("invalid + operation for values {:?}", other),
        }
    }

    pub fn to_json(&mut self, node: &ASTNode) -> Option<serde_json::Value> {
        match &node.token.token_type {
            TokenType::Mult => self.eval_mult(node),
            TokenType::Plus => self.eval_plus(node),
            TokenType::ID => {
                if !self.variables.contains_key(&node.token.plain()) {
                    panic!("the variable {} does not exist in file {}", node.token.plain(), self.source_path);
                }
                Some(self.variables.get(&node.token.plain()).unwrap().clone())
            },
            TokenType::BoolLiteral(value) => {
                Some(serde_json::Value::Bool(value.clone()))
            },
            TokenType::Block(_) => Some(self.block_to_json(node)),
            TokenType::NumberLiteral(num) => {

                Some(serde_json::Value::Number(Number::from_f64(num.parse::<f64>().unwrap().into()).expect(&format!("broke here: {:#?}", node))))
            },
            TokenType::Equals => self.eval_equal(node),
            TokenType::StringLiteral(s) => Some(serde_json::Value::String(s.to_string())), 
            TokenType::List(args)=> Some(
                Value::Array(args.to_nodes()
                    .into_iter()
                    .map(|n| self.to_json(&n).unwrap())
                    .collect::<Vec<serde_json::Value>>())
            ),
            TokenType::From => self.eval_from(node),
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
                
                    self.templates.insert(node.token.plain(), Template::new(args, block.clone()));
                    return None;
                }

                self.templates.insert(node.token.plain(), Template::new(Vec::new(), block.clone()));
                return None;
            },
            TokenType::LuaFnCall(_) => self.eval_lua_fn(node),
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
    
    pub fn import_from_base(&mut self, key: &str) -> mlua::Result<()> {
        let lua_instance = self.lua_instance.borrow();
        let val: mlua::Value = lua_instance.base_env.get(key)?;
        self.lua_env.set(key, val.clone())?; // clone the function
                        



        let globals = lua_instance.lua_instance.globals();

        for pair in globals.clone().pairs::<mlua::Value, mlua::Value>() {
            let (k, v) = pair?;
            self.lua_env.set(k, v)?;
        }

        /*
        for pair in lua_instance.base_env.clone().pairs::<mlua::Value, mlua::Value>() {
            let (k, v) = pair.unwrap();
            //println!("\tbase_env contains: {:?} = {:?}", k, v);
        }

        
        for pair in lua_instance.base_env.clone().pairs::<mlua::Value, mlua::Value>() {
            let (k, v) = pair.unwrap();
            //println!("\tcontext_env contains: {:?} = {:?}", k, v);
        }
        */
        Ok(())
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

    pub fn export(&self, args: Vec<String>) -> Vec<ExportType> {
        let mut exported_values:Vec<ExportType> = Vec::new(); 

        for arg in &args {
            
            if arg == "$" {
                for (name, value) in self.variables.clone() {
                    exported_values.push(ExportType::Variable(name, value));
                }
                continue;
            }

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

    pub fn export_all(&self) -> Vec<ExportType> {
        let mut exported_values:Vec<ExportType> = Vec::new(); 

        for (name, value) in self.variables.clone() {
            exported_values.push(ExportType::Variable(name, value));
        }

        for (name, value) in self.templates.clone() {
                exported_values.push(ExportType::Template(name, value));
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


