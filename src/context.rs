use crate::{
    astnode::ASTNode, jason_errors::{JasonError, JasonResult}, jason_types::JasonType, lua_instance::LuaInstance, template::Template, token::{ArgsToNode, Token, TokenType}
};

use std::{collections::HashMap};
use mlua::Table;
use rand::Rng;
use serde_json::{Value, Number, Map};
use std::rc::Rc;
use std::cell::RefCell;
use crate::jason_hidden;
#[allow(unused_imports)]
use crate::jason_errors;
use crate::jason_errors::JasonErrorKind;

#[derive(Debug)]
pub enum ExportType {
    Template(String, Template),
    Variable(String, serde_json::Value)
}
#[derive(Debug)]
pub struct Context {
    pub variables: HashMap<String, serde_json::Value>,
    pub templates: HashMap<String, Template>,
    pub types: HashMap<String, JasonType>,
    pub variable_types: HashMap<String, JasonType>,
    pub template_types: HashMap<String, (Vec<JasonType>, JasonType)>,
    pub out: serde_json::Value,
    pub source_path: Rc<String>,
    pub lua_instance: Rc<RefCell<LuaInstance>>,
    pub lua_env: Table,
    pub lua_fn_cache: HashMap<String, mlua::RegistryKey>, // cache lua functions
    pub local_root:Option<Rc<ASTNode>>
}

impl Context {    
    pub fn new(path: Rc<String>, lua_instance: Rc<RefCell<LuaInstance>>) -> JasonResult<Self> {
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
            types: HashMap::new(),
            variable_types: HashMap::new(),
            template_types: HashMap::new(),
            out: Value::Null,
            source_path: path,
            lua_instance,
            lua_env,
            lua_fn_cache: HashMap::new(),
            local_root: None
        })
    }
    
    pub fn set_local_root(&mut self, root: &ASTNode) {
        self.local_root = Some(Rc::new(root.clone()));
    }

    pub fn clear_local_root(&mut self) {
        self.local_root = None;
    }
    //repeat and evaluate
    fn repeat_value(&mut self, count: String, repeated_node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        let count = count.parse::<f64>().map_err(|_| 
            JasonError::new(JasonErrorKind::ConversionError, self.source_path.clone(), self.local_root.clone(), format!("failed to parse num {}", count)))?;
        let bound:usize = count as usize;
        let mut result: Vec<Value> = Vec::with_capacity(bound);
        for _ in 0..bound{
            let value = self.to_json(repeated_node)?.ok_or_else(||
                JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(),"value is None"))?;
            result.push(value);
        }
        return Ok(Some(Value::Array(result)));
    }
    //repeat and don't evaluate
    fn dumb_repeat_value(&mut self, count: String, repeated_node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        let count = count.parse::<f64>().map_err(|_| 
            JasonError::new(JasonErrorKind::ConversionError, self.source_path.clone(), self.local_root.clone(), format!("failed to parse num {}", count)))?;
        let bound:usize = count as usize;
        let mut result: Vec<Value> = Vec::with_capacity(bound);

        let value = self.to_json(repeated_node)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(),"value is None"))?;
        for _ in 0..bound{
            result.push(value.clone());
        }
        return Ok(Some(Value::Array(result)));
    }

    fn eval_mult(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        // Repeat operations: expression * n or n * expression
        if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
            match (&left_node.token.token_type, &right_node.token.token_type) {
                (_, TokenType::ID) => {
                    let eval_variable = self.to_json(right_node)?.unwrap();                    
                    return self.repeat_value(eval_variable.to_string(), &left_node);
                }
                (TokenType::ID, _) => {
                    let eval_variable = self.to_json(left_node)?.unwrap();                    
                    return self.repeat_value(eval_variable.to_string(), &right_node);
                }

                // Left-sided: expression * n
                (_, TokenType::NumberLiteral(num)) => {
                    return self.repeat_value(num.clone(), &left_node);
                }
                // Right-sided: n * expression
                (TokenType::NumberLiteral(num), _) => {
                    return self.repeat_value(num.clone(), &right_node);
                }
                _ => return Err(JasonError::new(JasonErrorKind::InvalidOperation, self.source_path.clone(), self.local_root.clone(), 
                    "invalid operation"))
            }
        }
        Err(JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(), self.local_root.clone(),
            format!("repeat statement failed")))
    }

    fn eval_repeat(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        // Repeat operations: expression * n or n * expression
        if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
            match (&left_node.token.token_type, &right_node.token.token_type) {
                (_, TokenType::ID) => {
                    let eval_variable = self.to_json(right_node)?.unwrap();                    
                    return self.dumb_repeat_value(eval_variable.to_string(), &left_node);
                }
                (TokenType::ID, _) => {
                    let eval_variable = self.to_json(left_node)?.unwrap();                    
                    return self.dumb_repeat_value(eval_variable.to_string(), &right_node);
                }

                // Left-sided: expression * n
                (_, TokenType::NumberLiteral(num)) => {
                    return self.dumb_repeat_value(num.clone(), &left_node);
                }
                // Right-sided: n * expression
                (TokenType::NumberLiteral(num), _) => {
                    return self.dumb_repeat_value(num.clone(), &right_node);
                }
                _ => return Err(JasonError::new(JasonErrorKind::InvalidOperation, self.source_path.clone(), self.local_root.clone(), 
                    "invalid operation"))
            }
        }
        Err(JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(), self.local_root.clone(),
            format!("repeat statement failed")))
    }

    pub fn eval_equal(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
            let left = &left_node;
            let right = &right_node;
            if left.token.token_type != TokenType::ID {
                return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                    format!("[ERROR] {} = {}, variable name must be a valid identifier!",
                        left.token.plain(),
                        right.token.plain())));
            }

            let right_value = self.to_json(right)?.ok_or_else(||
                JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), None,"right value is None"))?;
            let var_name = left.token.plain();

            if self.variable_types.contains_key(&var_name) {
                let infered_type = self.infer_type_from(&right_value)?;
                let typed_var = self.variable_types.get(&var_name).unwrap();
                if !typed_var.matches(&right_value) {
                    return Err(
                        self.err(
                            JasonErrorKind::TypeError(var_name),
                            format!("type mismatches\n expected {}, found {}", typed_var, infered_type)
                        )
                    )
                }
            }

            self.variables.insert(var_name, right_value);             
        }
        Ok(None)
    }

    pub fn eval_narwhal(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
            let left = &left_node;
            let right = &right_node;
            if left.token.token_type != TokenType::ID {
                return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                    format!("[ERROR] {} = {}, variable name must be a valid identifier!",
                        left.token.plain(),
                        right.token.plain())));
            }

            let right_value = self.to_json(right)?.ok_or_else(||
                JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), None,"right value is None"))?;
            let var_name = left.token.plain();

            if self.variable_types.contains_key(&var_name) {
                return Err(self.err(
                    JasonErrorKind::TypeError(var_name.clone()),
                    format!("cannot reassign type of {}, existing type is, {}", var_name, self.variable_types.get(&var_name).unwrap())
                ))
            }
            let infered_type = self.infer_type_from(&right_value)?;
            self.variable_types.insert(var_name.clone(), infered_type);
            println!("{:?}", self.variable_types);
            self.variables.insert(var_name, right_value);             
        }
        Ok(None)
    }
    
    pub fn eval_from(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
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
                _ => return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                    "[ERROR] from statement must have a string path\n ... from \"<Path>\"")),
            }
            match left_node.token.token_type.clone() {
                TokenType::Import(args) =>  { 
                    if import_path == "" {
                        return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                            "to import templates/variable you must import from a string path I.E. import(...) from \"path/to/file\""));
                    }
                    let context = match jason_hidden::jason_context_from_file(import_path.clone(), self.lua_instance.clone()) {
                        Ok(v) => Ok(v),
                        Err(mut err) => {
                            err.file = self.source_path.clone();
                            err.node = self.local_root.clone();
                            Err(err)
                        },
                    }?;
                    let args:Vec<String> = args.to_nodes()?.into_iter().map(|node| node.token.plain()).collect();
                    if args.contains(&"*".to_string()) {
                        let exports = context.export_all();
                        self.absorb_exports(exports);
                        return Ok(None);
                    } 
                    let exports = context.export(args);
                    self.absorb_exports(exports);                           
                    return Ok(None);
                }
                TokenType::Use(args) => {
                    if lua_import_path == "" {
                        return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), None,
                            "to import lua functions you must derive from a plain component I.E. use(...) from std\n note how std std is plain text and not in qoutes"));
                    }
                    let args:Vec<String> = args.to_nodes()?.into_iter().map(|node| node.token.plain()).collect();                                     
                    for arg in &args {
                        let _ = self.import_from_base(arg);
                    }
                    if args.contains(&"*".to_string()) {
                        return Ok(None);
                    } 
                    return Ok(None); 
                },
                _ => return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(),None,
                    "[ERROR] from statement must have a string path\n ... from \"<Path>\"")),
            }
        }
        return Ok(None);
    }
    
    pub fn eval_lua_fn(&mut self, node:&ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let TokenType::LuaFnCall(args) = &node.token.token_type {
            // Convert to JSON first
            let json_values: Vec<Value> = args.to_nodes()?
                .iter()
                .map(|node| {
                    self.to_json(node)?
                        .ok_or_else(|| JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), None, "Argument is Empty"))
                })
                .collect::<JasonResult<Vec<Value>>>()?;
            // Now borrow lua
            let lua = self.lua_instance.borrow();
            // Convert to Lua values
            let lua_args: Vec<mlua::Value> = json_values.iter()
                .map(|value| lua.json_to_lua_value(value.clone()))
                .collect::<Result<Vec<_>, _>>()?;     
            let fn_tok = node.token.clone();
            
            //retrieve function from cache or load functino into cache
            let func = if let Some(key) = self.lua_fn_cache.get(&fn_tok.plain()) {
                //get function from cache
                lua.lua_instance.registry_value::<mlua::Function>(key)?
            } else {
                // load function from lua directly  
                let func: mlua::Function = lua.lua_instance
                    .load(&fn_tok.plain())
                    .set_environment(self.lua_env.clone())
                    .eval()
                    .map_err(|e| JasonError::new(JasonErrorKind::LuaFnError(fn_tok.pretty()), self.source_path.clone(), self.local_root.clone(), format!("failed to find function {}: {}", fn_tok.plain(), e)))?;
                
                // Store in registry for reuse
                let key = lua.lua_instance.create_registry_value(func.clone())?;
                drop(lua); // Drop borrow before mutable access
                self.lua_fn_cache.insert(fn_tok.plain().clone(), key);
                func
            };
            
            // Call the function with arguments
            let result = 
                func.call::<mlua::MultiValue>(mlua::MultiValue::from_vec(lua_args)).map_err(|e|
                    JasonError::new(
                        JasonErrorKind::LuaFnError(fn_tok.pretty()), self.source_path.clone(), self.local_root.clone(), format!("failed to evaluate function {}: {}\n", fn_tok.plain(), e)
                        )
                )?;
                    // Get the first value from the result
            let json_value: serde_json::Value = if let Some(first) = result.into_iter().next() {
                LuaInstance::lua_value_to_json(&first)
            } else {
                serde_json::Value::Null
            };
            return Ok(Some(json_value));
        }
        Err(JasonError::new(JasonErrorKind::TypeError(node.token.pretty()), self.source_path.clone(), None,
            "at eval_lua_fn token is not of luafncall"))
    }
    
    pub fn eval_plus(&mut self, node:&ASTNode) -> JasonResult<Option<serde_json::Value>> {
        let left = self.to_json(node.left.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue,self.source_path.clone(), self.local_root.clone(), format!("left side of the expression is missing")))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(),self.local_root.clone(), "left value is None"))?;
        let right = self.to_json(node.right.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(),self.local_root.clone(), "right node missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError,self.source_path.clone(), self.local_root.clone(), "right value is None"))?;
        
        match (left, right) {
            //[] + [] => [1,2,3] + [4,5] = [1,2,3,4,5]
            (Value::Array(mut a), Value::Array(b)) => {
                a.extend(b);
                Ok(Some(Value::Array(a)))
            },
            //"..." + "..." => "hello" + " world" = "hello world"
            (Value::String(a), Value::String(b)) => {
                Ok(Some(Value::String(a + &b)))
            }
            // {...} + {...} => {name: "Alex"} + {age:20} = {name:"Alex", age: 20} 
            (Value::Object(mut a), Value::Object(b)) => {
                a.extend(b);
                Ok(Some(Value::Object(a)))
            }
            _ => Err(JasonError::new(JasonErrorKind::InvalidOperation, self.source_path.clone(), self.local_root.clone(),
                format!("invalid + operation for values ", ))),
        }
    }

    pub fn eval_at(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
       let left = self.to_json(node.left.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue,self.source_path.clone(), self.local_root.clone(), format!("left side of the expression is missing")))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(),self.local_root.clone(), "left value is None"))?;
        let right = self.to_json(node.right.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(),self.local_root.clone(), "right node missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError,self.source_path.clone(), self.local_root.clone(), "right value is None"))?;

        match (left, right) {
            // [a, b, c, ...] at 0 -> a
            (Value::Array(a), Value::Number(n)) => {
                let index = n.as_f64().ok_or_else(|| JasonError::new(
                    JasonErrorKind::ConversionError,
                    self.source_path.clone(),
                    self.local_root.clone(),
                    format!("unable to convert number {} to index", n)
                ))? as usize;

                a.get(index)
                    .cloned() 
                    .ok_or_else(|| JasonError::new(
                        JasonErrorKind::IndexError,
                        self.source_path.clone(),
                        self.local_root.clone(),
                        format!("invalid convert number {} at list with len {}", index, a.len())
                    ))
                    .map(Some)
            },
             // [a, b, c, ..., z] at 0..n -> [a, b, c, ... up to n]
            // {name: "alex", age: 20} at "name" -> "alex" 
            (Value::String(s), Value::Number(n)) => {
                let index = n.as_f64().ok_or_else(|| JasonError::new(
                    JasonErrorKind::ConversionError,
                    self.source_path.clone(),
                    self.local_root.clone(),
                    format!("unable to parse number {} to index", n)
                ))? as usize;

                s.chars().nth(index).ok_or_else(|| JasonError::new(
                        JasonErrorKind::IndexError, 
                        self.source_path.clone(), 
                        self.local_root.clone(),  
                        format!("invalid convert number {} at list with len {}", index, s.len())
                )).map(|c| Some(Value::String(c.to_string()))) 
            }

            // {name: "alex", age: 20} at ["name", "age"] -> ["alex", 20] 
            (Value::Object(a), Value::String(key)) => {
                a.get(&key).ok_or_else(|| {
                    JasonError::new(
                        JasonErrorKind::IndexError, 
                        self.source_path.clone(), 
                        self.local_root.clone(),  
                        format!("key doesn't exit {}", key)
                    )
                }).map(|v| Some(v.clone()))
            }
            // [{name: "alex", age: 20}, {name: "jason", age: 38} at each ["name"] -> ["alex", "jason"] 
            _ => Err(JasonError::new(JasonErrorKind::InvalidOperation, self.source_path.clone(), self.local_root.clone(),
                format!("invalid + operation for values ", ))),
        }        
    }

    pub fn eval_pick(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
       let left = self.to_json(node.left.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue,self.source_path.clone(), self.local_root.clone(), format!("left side of the expression is missing")))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(),self.local_root.clone(), "left value is None"))?;
        let right = self.to_json(node.right.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(),self.local_root.clone(), "right node missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError,self.source_path.clone(), self.local_root.clone(), "right value is None"))?;
      
        //rust thinks this is unused
        #[allow(unused_assignments)]
        let mut count:usize = 0;

        if let Value::Number(n) = right {
            count = n.as_f64().ok_or_else(|| JasonError::new(
                    JasonErrorKind::ConversionError,
                    self.source_path.clone(),
                    self.local_root.clone(),
                    format!("unable to convert number {} to index", n)
                ))? as usize;
        } else {
            return Err(JasonError::new(
                JasonErrorKind::ValueError, 
                self.source_path.clone(), 
                self.local_root.clone(), 
                format!("value must be of type number")
            ))
        }

        match left {
            // [a, b, c, ...] at 0 -> a
            Value::Array(a) => {
                
                if a.is_empty() {
                    return Ok(Some(serde_json::Value::Array(Vec::new())));
                }

                let mut result:Vec<serde_json::Value> = Vec::with_capacity(count);
                
                for _ in 0..count {

                    let index = rand::rng().random_range(0..a.len());

                    let value = a.get(index).ok_or_else(|| JasonError::new(
                            JasonErrorKind::IndexError, 
                            self.source_path.clone(), 
                            self.local_root.clone(), 
                            format!("unable to index array while picking with values {}", count)
                        )
                    )?.clone();
                    result.push(value); 
                }

                return Ok(Some(serde_json::Value::Array(result)))
            },
            _ => Err(JasonError::new(JasonErrorKind::InvalidOperation, self.source_path.clone(), self.local_root.clone(),
                format!("invalid + operation for values ", ))),
        }        
    }

    pub fn eval_upick(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
       let left = self.to_json(node.left.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue,self.source_path.clone(), self.local_root.clone(), format!("left side of the expression is missing")))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(),self.local_root.clone(), "left value is None"))?;
        let right = self.to_json(node.right.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(),self.local_root.clone(), "right node missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError,self.source_path.clone(), self.local_root.clone(), "right value is None"))?;
      
        //rust thinks this is unused
        #[allow(unused_assignments)]
        let mut count:usize = 0;

        if let Value::Number(n) = right {
            count = n.as_f64().ok_or_else(|| JasonError::new(
                    JasonErrorKind::ConversionError,
                    self.source_path.clone(),
                    self.local_root.clone(),
                    format!("unable to convert number {} to index", n)
                ))? as usize;
        } else {
            return Err(JasonError::new(
                JasonErrorKind::ValueError, 
                self.source_path.clone(), 
                self.local_root.clone(), 
                format!("value must be of type number")
            ))
        }

        match left {
            // [a, b, c, ...] at 0 -> a
            Value::Array(a) => { 
                if a.is_empty() {
                    return Ok(Some(serde_json::Value::Array(Vec::new())));
                }

                if count > a.len() {
                    return Err(
                        JasonError::new(
                            JasonErrorKind::InvalidOperation, 
                            self.source_path.clone(), 
                            self.local_root.clone(), 
                            format!("unable to use upick operaton when count {} is larger than the count of elements {}", count, a.len())
                        )
                    )
                }

                let mut result:Vec<serde_json::Value> = Vec::with_capacity(count);
                let mut possible_indexs = (0..a.len()).collect::<Vec<usize>>(); 

                for _ in 0..count {

                    let index = rand::rng().random_range(0..possible_indexs.len());
                    
                    let picked_index = possible_indexs.remove(index);
                    let value = a.get(picked_index).ok_or_else(|| JasonError::new(
                            JasonErrorKind::IndexError, 
                            self.source_path.clone(), 
                            self.local_root.clone(), 
                            format!("unable to index array while picking with values {}", count)
                        )
                    )?.clone();

                    result.push(value); 
                }

                return Ok(Some(serde_json::Value::Array(result)))
            },
            _ => Err(JasonError::new(JasonErrorKind::InvalidOperation, self.source_path.clone(), self.local_root.clone(),
                format!("invalid + operation for values ", ))),
        }        
    }

    pub fn eval_map(&mut self, node:&ASTNode) -> JasonResult<Option<serde_json::Value>>{
        let left = self.to_json(node.left.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue,self.source_path.clone(), self.local_root.clone(), format!("left side of the expression is missing")))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(),self.local_root.clone(), "left value is None"))?;
        let right = node.right.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(),self.local_root.clone(), "right node missing"))?;
        

        let args = match &node.token.token_type {
            TokenType::Map(args) => args,
            _ => return Err(
                 JasonError::new(
                    JasonErrorKind::ValueError, 
                    self.source_path.clone(), 
                    self.local_root.clone(), 
                    format!("left side of the operand must be of type List")
                ))
        };

        let values = match left {
            Value::Array(args) => args,
            _ => return Err(
                JasonError::new(
                    JasonErrorKind::ValueError, 
                    self.source_path.clone(), 
                    self.local_root.clone(), 
                    format!("left side of the operand must be of type List")
                ))
        };
        let mut flat_args: Vec<Token> = args.iter().flat_map(|v| v.iter().cloned()).collect();

        if flat_args.is_empty() {
            return Err(JasonError::new(
                JasonErrorKind::ValueError,
                self.source_path.clone(),
                self.local_root.clone(),
                "map must have at least one argument",
            ));
        }

        let argument = flat_args.remove(0).plain(); // first token
        let index_argument = if !flat_args.is_empty() {
            flat_args.remove(0).plain() // second token, if exists
        } else {
            "".to_string()
        };
        let has_index_argument = !index_argument.is_empty();
        
        let mut results:Vec<Value> = Vec::with_capacity(values.len());

        for (i, value) in values.into_iter().enumerate() {
            self.variables.insert(argument.clone(), value);
            
            if has_index_argument {
                self.variables.insert(index_argument.clone(), Value::Number(i.into()));
            }

            results.push(
                self.to_json(right)?
                    .ok_or_else(|| 
                        JasonError::new(
                            JasonErrorKind::ValueError,
                            self.source_path.clone(),
                            self.local_root.clone(),
                            format!("variable {} in mapping already exists", argument),
                        )
                    )?
            );

            if has_index_argument {
                self.variables.remove(&index_argument);
            }

            self.variables.remove(&argument);
        }
        
        Ok(Some(Value::Array(results)))
    }
    
    fn eval_double_colon(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        let left = 
                node.left
                .as_ref()
                .ok_or_else(||
                    self.err(JasonErrorKind::MissingValue, format!("left side of the expression is missing"))
                )?;

        let typed_value:JasonType = 
            self.to_type(
                node
                .right
                .as_ref()
                .ok_or_else(||
                    self.err(JasonErrorKind::MissingValue,format!("right side of the expression is missing"))
                )?
            )?;        

        match &left.token.token_type {
            TokenType::ID => {
                self.variable_types.insert(left.token.plain(), typed_value);
                println!("{:?}", self.variable_types);
                Ok(None)
            },
            TokenType::FnCall(args) => {
                let typed_args = args
                        .to_nodes()?
                        .iter()
                        .map(|e| self.to_type(e))
                        .collect::<JasonResult<Vec<JasonType>>>()?;
 
                self.template_types.insert(
                    left.token.plain(), 
                    (
                        typed_args,
                        typed_value
                    )
                );
                println!("{:?}", self.template_types);
                Ok(None)
            },
            _ => {
                Err(
                    self.err(
                        JasonErrorKind::Custom, 
                        format!("Cannot set Type to value other than ID and Template()")
                    )   
                )
            }
        }

    }

    pub fn to_json(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        match &node.token.token_type {
            TokenType::Null => Ok(Some(serde_json::Value::Null)),
            TokenType::Map(_) => self.eval_map(node),
            TokenType::Mult => self.eval_mult(node),
            TokenType::Repeat => self.eval_repeat(node),
            TokenType::Plus => self.eval_plus(node),
            TokenType::At => self.eval_at(node),
            TokenType::Pick => self.eval_pick(node),
            TokenType::UPick => self.eval_upick(node),
            TokenType::StringConverion(args) => {
                let args:Vec<ASTNode> = args.to_nodes()?;

                let inner_value = args.get(0).ok_or_else(|| 
                    JasonError::new(
                        JasonErrorKind::ValueError, 
                        self.source_path.clone(), 
                        self.local_root.clone(), 
                        format!("needs an inner value to convert from")
                    )
                )?;
                
                let value = self.to_json(inner_value)?.ok_or_else(|| 
                    JasonError::new(
                        JasonErrorKind::ConversionError, 
                        self.source_path.clone(), 
                        self.local_root.clone(), 
                        format!("failed to evaluate value into string")
                    )
                )?;
                
                let result = match value {
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => String::from("null"),
                    v => return Err(JasonError::new(
                        JasonErrorKind::ConversionError, 
                        self.source_path.clone(), 
                        self.local_root.clone(), 
                        format!("Cannot convert type {} into string", v)
                    ))
                };

                Ok(Some(Value::String(result)))
            },
            TokenType::ID => {
                if !self.variables.contains_key(&node.token.plain()) {
                    return Err(JasonError::new(JasonErrorKind::UndefinedVariable(node.token.plain()), self.source_path.clone(),self.local_root.clone(),
                        format!("the variable {} does not exist in file {}", node.token.plain(), self.source_path.clone())));
                }
                Ok(Some(self.variables.get(&node.token.plain()).unwrap().clone()))
            },
            TokenType::BoolLiteral(value) => {
                Ok(Some(serde_json::Value::Bool(value.clone())))
            },
            TokenType::Block(_) => Ok(Some(self.block_to_json(node)?.ok_or_else(||
                JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(),"block returned None"))?)),
            TokenType::NumberLiteral(num) => {
                let parsed = num.parse::<f64>().map_err(|_|
                    JasonError::new(JasonErrorKind::ConversionError, self.source_path.clone(), self.local_root.clone(), format!("failed to parse number: {}", num)))?;
                Ok(Some(serde_json::Value::Number(Number::from_f64(parsed).ok_or_else(||
                    JasonError::new(JasonErrorKind::ConversionError, self.source_path.clone(), self.local_root.clone(), 
                        format!("Failed to convert number")))?)))
            },
            TokenType::Equals => self.eval_equal(node),
            TokenType::Narwhal => self.eval_narwhal(node),
            TokenType::StringLiteral(s) => Ok(Some(serde_json::Value::String(s.to_string()))), 
            TokenType::List(args) => {
                let json_values: Vec<Value> = args.to_nodes()?
                    .iter()
                    .map(|node| {
                        self.to_json(node)?
                            .ok_or_else(|| JasonError::new(jason_errors::JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(), "List item is None"))
                    })
                    .collect::<JasonResult<Vec<Value>>>()?;                
                Ok(Some(Value::Array(json_values)))
            }
            TokenType::From => self.eval_from(node),
            TokenType::Out => {
                if let Some(right_node) = node.right.as_ref() {
                    self.out = self.to_json(right_node)?.ok_or_else(||
                        JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(), "out value is None"))?;
                    return Ok(None);
                }
                Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                    "out statement must have valid jason expression.\n example: out \"Hello!\""))
            },
            TokenType::TemplateDef(args, block) => {
                let args = args.to_nodes()?;
                if args.len() > 0 {
                    let args:Vec<String> = args.into_iter().map(|node| node.token.plain()).collect();
                
                    self.templates.insert(node.token.plain(), Template::new(args, block.clone()));
                    return Ok(None);
                }
                self.templates.insert(node.token.plain(), Template::new(Vec::new(), block.clone()));
                return Ok(None);
            },
            TokenType::LuaFnCall(_) => self.eval_lua_fn(node),
            TokenType::FnCall(args) => { 
                if !self.templates.contains_key(&node.token.plain()) {
                    return Err(JasonError::new(JasonErrorKind::UndefinedTemplate(node.token.plain()), self.source_path.clone(), self.local_root.clone(), format!("the template {} does not exist in file {}", node.token.plain(), self.source_path)));
                }

                let template = self.templates.get(&node.token.plain()).unwrap().clone();
                template.resolve(self, args.to_vec())
            },
            token => {
                Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                    format!("Unknown token: {:?}", token)))
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
    pub fn block_to_json(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let TokenType::Block(args) = &node.token.token_type {
            let nodes = args.to_nodes()?;
            let mut map = Map::new(); // this will become our JSON object
            for node in nodes {
                if node.token.token_type == TokenType::Colon {
                    let key_node = node.left.as_ref().ok_or_else(||
                        JasonError::new(JasonErrorKind::MissingKey, self.source_path.clone(), self.local_root.clone(), "Missing key"))?;
                    let value_node = node.right.as_ref().ok_or_else(||
                        JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(), self.local_root.clone(), "Missing value"))?;
                    if key_node.token.token_type != TokenType::ID {
                        return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(),
                            self.local_root.clone(), "Key must be an ID"));
                    }
                    let key = key_node.token.plain();
                    let value = self.to_json(&*value_node)?; // recursive call
                    map.insert(key, value.ok_or_else(||
                        JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(), "block value is None"))?);
                    continue;
                }
                return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                    "values must adheere to <key : value> fields in blocks"));
            }
            Ok(Some(Value::Object(map)))
        } else {
            Err(JasonError::new(JasonErrorKind::TypeError(node.token.pretty()), self.source_path.clone(), self.local_root.clone(),
                "block_to_json called on non-block token"))
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
            //make this return a jason error
            panic!("{} is not exported from file {}", arg, self.source_path.clone());
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

    pub fn err(&self, error_kind: JasonErrorKind, msg: String) -> JasonError {
        JasonError::new(
            error_kind, 
            self.source_path.clone(), 
            self.local_root.clone(),
            msg
        )
    }
}
