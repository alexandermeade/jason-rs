use crate::{
    astnode::ASTNode, jason_errors::{JasonError, JasonResult}, jason_types::JasonType, lua_instance::LuaInstance, template::Template, token::TokenType
};

use colored::*;

use crate::jason_to_json;
use std::{collections::{HashMap, HashSet}, path::Path};
use mlua::Table;
use rand::Rng;
use serde_json::{Map, Number, Value};
use std::rc::Rc;
use std::cell::RefCell;
use crate::jason_hidden;
#[allow(unused_imports)]
use crate::jason_errors;
use crate::jason_errors::JasonErrorKind;

#[derive(Debug)]
pub enum ExportType {
    Template(String, Template),
    Variable(String, serde_json::Value),
    TemplateType(String, (Vec<JasonType>, JasonType)),
    VariableType(String, JasonType),
    Type(String, JasonType)
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
    pub local_root:Option<Rc<ASTNode>>,
    pub imported_from: Rc<RefCell<HashSet<String>>>,
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
            local_root: None,
            imported_from: RefCell::new(HashSet::new()).into()
        })
    }
    
    pub fn set_local_root(&mut self, root: &ASTNode) {
        self.local_root = Some(Rc::new(root.clone()));
    }

    pub fn clear_local_root(&mut self) {
        self.local_root = None;
    }
    //repeat and evaluate
    /*
    fn repeat_value(&mut self, count: &i64, repeated_node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        
        if *count < 0 {
            return Err(
                JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(),"Count must be positive")
            );  
        }

        let mut result: Vec<Value> = Vec::with_capacity(*count as usize);
        for _ in 0..*count{
            let value = self.to_json(repeated_node)?.ok_or_else(||
                JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(),"value is None"))?;
            result.push(value);
        }
        return Ok(Some(Value::Array(result)));
    }
    */

    //repeat and don't evaluate
    /*
    fn dumb_repeat_value(&mut self, count: &i64, repeated_node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        
        if *count < 0 {
            return Err(
                JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(),"Count must be positive")
            ); 
        }

        let mut result: Vec<Value> = Vec::with_capacity(*count as usize);

        let value = self.to_json(repeated_node)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(),"value is None"))?;
        for _ in 0..*count{
            result.push(value.clone());
        }
        return Ok(Some(Value::Array(result)));
    }*/
    /*
    fn extract_repeat_count(&self, value: serde_json::Value) -> JasonResult<i64> {
        match value {
            Value::Number(n) => n.as_i64().ok_or_else(|| {
                JasonError::new(
                    JasonErrorKind::InvalidOperation(n.to_string()),
                    self.source_path.clone(),
                    self.local_root.clone(),
                    "repeat count must be of type int",
                )
            }),
            v => Err(JasonError::new(
                JasonErrorKind::InvalidOperation(self.value_to_string(&v)?),
                self.source_path.clone(),
                self.local_root.clone(),
                "repeat count must be of type Int",
            )),
        }
    }*/

    fn eval_mult(&mut self, node: &ASTNode) -> JasonResult<Option<Value>> {
        let (left, right) = match (node.left.as_ref(), node.right.as_ref()) {
            (Some(l), Some(r)) => (l, r),
            _ => {
                return Err(JasonError::new(
                    JasonErrorKind::MissingValue,
                    self.source_path.clone(),
                    self.local_root.clone(),
                    "mult statement failed",
                ))
            }
        };

        // Fast path for literal arithmetic (no evaluation needed)
        match (&left.token.token_type, &right.token.token_type) {
            (TokenType::IntLiteral(n1), TokenType::IntLiteral(n2)) => {
                return Ok(Some(Value::Number((n1 * n2).into())));
            },
            (TokenType::IntLiteral(n1), TokenType::FloatLiteral(n2)) => {
                return Ok(Some(Value::Number(Number::from_f64((*n1 as f64) * n2).unwrap())));
            },
            (TokenType::FloatLiteral(n1), TokenType::IntLiteral(n2)) => {
                return Ok(Some(Value::Number(Number::from_f64(n1 * (*n2 as f64)).unwrap())));
            },
            (TokenType::FloatLiteral(n1), TokenType::FloatLiteral(n2)) => {
                return Ok(Some(Value::Number(Number::from_f64(n1 * n2).unwrap())));
            },
            _ => {}
        }

        // Evaluate both sides to determine operation
        let left_val = self.to_json(left)?;
        let right_val = self.to_json(right)?;

        match (left_val, right_val) {
            // Both are numbers -> arithmetic multiplication
            (Some(Value::Number(n1)), Some(Value::Number(n2))) => {
                match (n1.as_i64(), n2.as_i64()) {
                    (Some(i1), Some(i2)) => {
                        Ok(Some(Value::Number((i1 * i2).into())))
                    },
                    _ => {
                        let f1 = n1.as_f64().ok_or_else(|| {
                            JasonError::new(
                                JasonErrorKind::ValueError,
                                self.source_path.clone(),
                                self.local_root.clone(),
                                "Failed to convert to f64",
                            )
                        })?;
                        let f2 = n2.as_f64().ok_or_else(|| {
                            JasonError::new(
                                JasonErrorKind::ValueError,
                                self.source_path.clone(),
                                self.local_root.clone(),
                                "Failed to convert to f64",
                            )
                        })?;
                        Ok(Some(Value::Number(
                            Number::from_f64(f1 * f2).ok_or_else(|| {
                                JasonError::new(
                                    JasonErrorKind::ValueError,
                                    self.source_path.clone(),
                                    self.local_root.clone(),
                                    "Multiplication overflow",
                                )
                            })?
                        )))
                    }
                }
            },

            // Right is a number, left is anything else -> repeat left n times
            (Some(left_value), Some(Value::Number(n))) => {
                let count = n.as_i64().ok_or_else(|| {
                    JasonError::new(
                        JasonErrorKind::InvalidOperation(n.to_string()),
                        self.source_path.clone(),
                        self.local_root.clone(),
                        "Cannot repeat with a count of type float!",
                    )
                })?;

                // Repeat the left value
                let mut result = Vec::new();
                for _ in 0..count {
                    result.push(left_value.clone());
                }
                Ok(Some(Value::Array(result)))
            },

            // Left is a number, right is anything else -> repeat right n times
            (Some(Value::Number(n)), Some(right_value)) => {
                let count = n.as_i64().ok_or_else(|| {
                    JasonError::new(
                        JasonErrorKind::InvalidOperation(n.to_string()),
                        self.source_path.clone(),
                        self.local_root.clone(),
                        "Cannot repeat with a count of type float!",
                    )
                })?;

                // Repeat the right value
                let mut result = Vec::new();
                for _ in 0..count {
                    result.push(right_value.clone());
                }
                Ok(Some(Value::Array(result)))
            },

            // One or both sides evaluated to None
            (None, _) => Err(JasonError::new(
                JasonErrorKind::MissingValue,
                self.source_path.clone(),
                self.local_root.clone(),
                "Left side of * evaluated to None",
            )),
            (_, None) => Err(JasonError::new(
                JasonErrorKind::MissingValue,
                self.source_path.clone(),
                self.local_root.clone(),
                "Right side of * evaluated to None",
            )),
            _ => Err(JasonError::new(
                JasonErrorKind::MissingValue,
                self.source_path.clone(),
                self.local_root.clone(),
                "Both side of * evaluated to None",
            )),
        }
    }

    pub fn eval_minus(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        let left = self.to_json(node.left.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(), self.local_root.clone(), 
                "left side of the expression is missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(), 
                "left value is None"))?;
        
        let right = self.to_json(node.right.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(), self.local_root.clone(), 
                "right node missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(), 
                "right value is None"))?;
        
        match (left, right) {
            (Value::Number(n1), Value::Number(n2)) => {
                // Try integer subtraction first
                if let (Some(a), Some(b)) = (n1.as_i64(), n2.as_i64()) {
                    Ok(Some(Value::Number((a - b).into())))
                }
                // Fall back to float
                else if let (Some(a), Some(b)) = (n1.as_f64(), n2.as_f64()) {
                    if let Some(num) = Number::from_f64(a - b) {
                        Ok(Some(Value::Number(num)))
                    } else {
                        Err(JasonError::new(
                            JasonErrorKind::ValueError,
                            self.source_path.clone(),
                            self.local_root.clone(),
                            "Subtraction resulted in NaN or infinity"
                        ))
                    }
                } else {
                    Err(JasonError::new(
                        JasonErrorKind::ValueError,
                        self.source_path.clone(),
                        self.local_root.clone(),
                        "Failed to convert numbers for subtraction"
                    ))
                }
            },
            _ => Err(JasonError::new(
                JasonErrorKind::InvalidOperation("minus".to_string()),
                self.source_path.clone(),
                self.local_root.clone(),
                "invalid - operation: both operands must be numbers"
            )),
        }
    }

    pub fn eval_div(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        let left = self.to_json(node.left.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(), self.local_root.clone(), 
                "left side of the expression is missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(), 
                "left value is None"))?;
        
        let right = self.to_json(node.right.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(), self.local_root.clone(), 
                "right node missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(), 
                "right value is None"))?;
        
        match (left, right) {
            (Value::Number(n1), Value::Number(n2)) => {
                if let (Some(a), Some(b)) = (n1.as_f64(), n2.as_f64()) {
                    if b == 0.0 {
                        return Err(JasonError::new(
                            JasonErrorKind::ValueError,
                            self.source_path.clone(),
                            self.local_root.clone(),
                            "Division by zero"
                        ));
                    }
                    
                    if let Some(num) = Number::from_f64(a / b) {
                        Ok(Some(Value::Number(num)))
                    } else {
                        Err(JasonError::new(
                            JasonErrorKind::ValueError,
                            self.source_path.clone(),
                            self.local_root.clone(),
                            "Division resulted in NaN or infinity"
                        ))
                    }
                } else {
                    Err(JasonError::new(
                        JasonErrorKind::ValueError,
                        self.source_path.clone(),
                        self.local_root.clone(),
                        "Failed to convert numbers for division"
                    ))
                }
            },
            _ => Err(JasonError::new(
                JasonErrorKind::InvalidOperation("div".to_string()),
                self.source_path.clone(),
                self.local_root.clone(),
                "invalid / operation: both operands must be numbers"
            )),
        }
    }

    pub fn eval_mod(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        let left = self.to_json(node.left.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(), self.local_root.clone(), 
                "left side of the expression is missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(), 
                "left value is None"))?;
        
        let right = self.to_json(node.right.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(), self.local_root.clone(), 
                "right node missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(), 
                "right value is None"))?;
        
        match (left, right) {
            (Value::Number(n1), Value::Number(n2)) => {
                // Try integer modulo first (most common use case)
                if let (Some(a), Some(b)) = (n1.as_i64(), n2.as_i64()) {
                    if b == 0 {
                        return Err(JasonError::new(
                            JasonErrorKind::ValueError,
                            self.source_path.clone(),
                            self.local_root.clone(),
                            "Modulo by zero"
                        ));
                    }
                    Ok(Some(Value::Number((a % b).into())))
                }
                // Fall back to float modulo
                else if let (Some(a), Some(b)) = (n1.as_f64(), n2.as_f64()) {
                    if b == 0.0 {
                        return Err(JasonError::new(
                            JasonErrorKind::ValueError,
                            self.source_path.clone(),
                            self.local_root.clone(),
                            "Modulo by zero"
                        ));
                    }
                    
                    if let Some(num) = Number::from_f64(a % b) {
                        Ok(Some(Value::Number(num)))
                    } else {
                        Err(JasonError::new(
                            JasonErrorKind::ValueError,
                            self.source_path.clone(),
                            self.local_root.clone(),
                            "Modulo resulted in NaN or infinity"
                        ))
                    }
                } else {
                    Err(JasonError::new(
                        JasonErrorKind::ValueError,
                        self.source_path.clone(),
                        self.local_root.clone(),
                        "Failed to convert numbers for modulo"
                    ))
                }
            },
            _ => Err(JasonError::new(
                JasonErrorKind::InvalidOperation("mod".to_string()),
                self.source_path.clone(),
                self.local_root.clone(),
                "invalid % operation: both operands must be numbers"
            )),
        }
    }

    fn eval_repeat(&mut self, node: &ASTNode) -> JasonResult<Option<Value>> {
        let (left, right) = match (node.left.as_ref(), node.right.as_ref()) {
            (Some(l), Some(r)) => (l, r),
            _ => {
                return Err(JasonError::new(
                    JasonErrorKind::MissingValue,
                    self.source_path.clone(),
                    self.local_root.clone(),
                    "repeat statement failed",
                ))
            }
        };

        // Evaluate both sides
        let right_val = self.to_json(right)?;

        match right_val {
            // Right is a number -> repeat left n times (without re-evaluation)
            Some(Value::Number(n)) => {
                let count = n.as_i64().ok_or_else(|| {
                    JasonError::new(
                        JasonErrorKind::InvalidOperation(n.to_string()),
                        self.source_path.clone(),
                        self.local_root.clone(),
                        "repeat count must be of type Int",
                    )
                })?;

                let mut result:Vec<Value> = Vec::new();
                for _ in 0..count {
                    match self.to_json(&left)?.clone() {
                        Some(value) => result.push(value),
                        None => result.push(Value::Null)
                    }
                }
                Ok(Some(Value::Array(result)))
            },

            _ => Err(JasonError::new(
                JasonErrorKind::InvalidOperation("*ALL*".to_string()),
                self.source_path.clone(),
                self.local_root.clone(),
                "invalid repeat operation must be of the form ... repeat n ",
            )),
        }
    }

    pub fn eval_equal(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
            let left = &left_node;
            let right = &right_node;
            #[allow(unused)]
            let mut var_name:String = "".to_string();
            let mut typed:Option<JasonType> = None;
            match &left.token.token_type {
                TokenType::ID => {
                    var_name = left.token.plain();
                },
                TokenType::Colon => {
                    if let (Some(inner_left), Some(inner_right)) = (left.left.as_ref(), left.right.as_ref()) {
                        var_name = inner_left.token.plain();
                        typed = Some(self.to_type(&inner_right)?); 
                    } else {
                        return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                            format!("Type def is missing left and or right side! {} : {}",
                            left.token.plain(),
                            right.token.plain()))
                        )

                    } 
                    
                },

                _ => return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                    format!("[ERROR] {} = {}, variable name must be a valid identifier!",
                        left.token.plain(),
                        right.token.plain())))
            };

            let right_value = self.to_json(right)?.ok_or_else(||
                JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(), None,"right value is None"))?;
            
            if let Some(var_type) = typed {
                if self.variable_types.contains_key(&var_name) {
                    return Err(
                        self.err(
                            JasonErrorKind::TypeError(var_name.clone()),
                            format!("Type already exists for {}", var_name)
                        )
                    )
                }

                self.variable_types.insert(var_name.clone(), var_type.clone());
            }

            if self.variable_types.contains_key(&var_name) {
                let infered_type = self.infer_type_from(&right_value)?;
                let typed_var = self.variable_types.get(&var_name).unwrap();
                if !typed_var.matches(&right_value) {
                    return Err(
                        self.err(
                            JasonErrorKind::TypeError(var_name),
                            format!("type mismatches\n expected {}, found {}\n{}", typed_var, infered_type,
                                if let (JasonType::Object(o1), JasonType::Object(o2)) = (typed_var, &infered_type) {
                                    JasonType::diff_objects(&o1, &o2)                                 
                                } else {"".to_string()}
                            )
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
            self.variables.insert(var_name, right_value);             
        }
        Ok(None)
    }

    pub fn eval_spiderwalrus(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let (Some(left_node), Some(right_node)) = (node.left.as_ref(), node.right.as_ref()) {
            let left = &left_node;
            let right = &right_node;
            if left.token.token_type != TokenType::ID {
                return Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                    format!("[ERROR] {} = {}, variable name must be a valid identifier!",
                        left.token.plain(),
                        right.token.plain())));
            }

            let right_type = self.to_type(right)?;
            let var_name = left.token.plain();

            if self.variable_types.contains_key(&var_name) {
                return Err(self.err(
                    JasonErrorKind::TypeError(var_name.clone()),
                    format!("cannot reassign type of {}, existing type is, {}", var_name, self.variable_types.get(&var_name).unwrap())
                ))
            }
            self.variable_types.insert(var_name.clone(), right_type);
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
                    
                    if !self.imported_from.borrow_mut().insert(import_path.clone()) {
                        return Err(self.err(
                            JasonErrorKind::CircularImport,
                            format!("found circular import in file {} from {}", import_path, self.source_path)
                        ));
                    } 

                    // Pass it to the child context
                    let context = jason_hidden::jason_context_from_file_with_imports(import_path.clone(), self.lua_instance.clone(), self.imported_from.clone())?;

                    /*let mut context = match jason_hidden::jason_context_from_file(import_path.clone(), self.lua_instance.clone()) {
                        Ok(v) => Ok(v),
                        Err(mut err) => {
                            err.file = self.source_path.clone();
                            err.node = self.local_root.clone();
                            Err(err)
                        },
                    }?;

                    context.imported_from = self.imported_from.clone();
                    */
                    
                    let args:Vec<String> = args.into_iter().map(|node| node.token.plain()).collect();
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
                    let args:Vec<String> = args.into_iter().map(|node| node.token.plain()).collect();                                     
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
            let json_values: Vec<Value> = args
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
    

    pub fn merge(o1: Value, o2: Value) -> JasonResult<Value> {
        let mut result:Map<String, Value> = Map::new();
        if let (Value::Object(obj1), Value::Object(obj2)) = (&o1, o2) {
            for (k, v) in obj1 {
                // if obj1 doesn't contain a key in obj2 add key to obj 1
                if !obj2.contains_key(k) {
                    result.insert(k.clone(), v.clone());
                }     

                if obj2.contains_key(k) {
                    result.insert(k.clone(), Self::merge(v.clone(), obj2.get(k).unwrap().clone())?);
                }
            }
            for (k, v) in obj2 {
                if !obj1.contains_key(&k) {
                    result.insert(k.clone(), v.clone());
                }
            }

        } else {
            return Ok(o1);
        }
        
        Ok(Value::Object(result))
    }

    pub fn eval_merge(&mut self, node:&ASTNode) -> JasonResult<Option<serde_json::Value>> {
         let left = self.to_json(node.left.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue,self.source_path.clone(), self.local_root.clone(), format!("left side of the expression is missing")))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(),self.local_root.clone(), "left value is None"))?;
        let right = self.to_json(node.right.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(),self.local_root.clone(), "right node missing"))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError,self.source_path.clone(), self.local_root.clone(), "right value is None"))?;
        
        match (&left, &right) {
            (Value::Object(_), Value::Object(_)) => {
                let result = Self::merge(left, right)?;
                Ok(Some(result))
            }
            _ => Err(JasonError::new(JasonErrorKind::InvalidOperation("*ALL*".to_string()), self.source_path.clone(), self.local_root.clone(),
                format!("invalid + operation for values ", ))),
        }
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
            },
            (Value::Number(n1), Value::Number(n2)) => {
                // Try to handle as integers first
                if let (Some(a), Some(b)) = (n1.as_i64(), n2.as_i64()) {
                    Ok(Some(Value::Number((a + b).into())))
                }
                // Fall back to float if either operand is a float
                else if let (Some(a), Some(b)) = (n1.as_f64(), n2.as_f64()) {
                    if let Some(num) = Number::from_f64(a + b) {
                        Ok(Some(Value::Number(num)))
                    } else {
                        Err(JasonError::new(
                            JasonErrorKind::ValueError,
                            self.source_path.clone(),
                            self.local_root.clone(),
                            "Floating point operation resulted in NaN or infinity"
                        ))
                    }
                } else {
                    Err(JasonError::new(
                        JasonErrorKind::ValueError,
                        self.source_path.clone(),
                        self.local_root.clone(),
                        "Failed to convert numbers for addition"
                    ))
                }
            },
            _ => Err(JasonError::new(JasonErrorKind::InvalidOperation("*ALL*".to_string()), self.source_path.clone(), self.local_root.clone(),
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
            _ => Err(JasonError::new(JasonErrorKind::InvalidOperation("*ALL*".to_string()), self.source_path.clone(), self.local_root.clone(),
                format!("invalid at operation for values ", ))),
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
                    return Err(
                        JasonError::new(
                            JasonErrorKind::IndexError, 
                            self.source_path.clone(), 
                            self.local_root.clone(), 
                            format!("unable to pick from array with no elements")
                        )
                    )

                }

                let mut result:Vec<serde_json::Value> = Vec::with_capacity(count);
                
                if count == 1 {
                    let index = rand::rng().random_range(0..a.len());

                    let value = a.get(index).ok_or_else(|| JasonError::new(
                            JasonErrorKind::IndexError, 
                            self.source_path.clone(), 
                            self.local_root.clone(), 
                            format!("unable to index array while picking with values {}", count)
                        )
                    )?.clone();

                    return Ok(Some(value)); 
                }

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
            _ => Err(JasonError::new(JasonErrorKind::InvalidOperation("*ALL*".to_string()), self.source_path.clone(), self.local_root.clone(),
                format!("invalid pick operation for values ", ))),
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

                if count > a.len() {
                    return Err(
                        JasonError::new(
                            JasonErrorKind::InvalidOperation(count.to_string()), 
                            self.source_path.clone(), 
                            self.local_root.clone(), 
                            format!("unable to use upick operaton when count {} is larger than the count of elements {}", count, a.len())
                        )
                    )
                }

                if count == 1 {
                    let index = rand::rng().random_range(0..a.len());

                    let value = a.get(index).ok_or_else(|| JasonError::new(
                            JasonErrorKind::IndexError, 
                            self.source_path.clone(), 
                            self.local_root.clone(), 
                            format!("unable to index array while picking with values {}", count)
                        )
                    )?.clone();

                    return Ok(Some(value)); 
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
            v => Err(JasonError::new(JasonErrorKind::InvalidOperation(self.value_to_string(&v)?), self.source_path.clone(), self.local_root.clone(),
                format!("invalid repeat operation for values ", ))),
        }        
    }

    pub fn eval_map(&mut self, node:&ASTNode) -> JasonResult<Option<serde_json::Value>>{
        let left = self.to_json(node.left.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue,self.source_path.clone(), self.local_root.clone(), format!("left side of the expression is missing")))?)?.ok_or_else(||
            JasonError::new(JasonErrorKind::ValueError, self.source_path.clone(),self.local_root.clone(), "left value is None"))?;
        let right = node.right.as_ref().ok_or_else(||
            JasonError::new(JasonErrorKind::MissingValue, self.source_path.clone(),self.local_root.clone(), "right node missing"))?;
        

        let mut args:Vec<ASTNode> = match &node.token.token_type {
            TokenType::Map(args) => args.to_vec(),
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
        //let mut flat_args: Vec<Token> = args.iter().flat_map(|v| v.iter().cloned()).collect();

        if args.is_empty() {
            return Err(JasonError::new(
                JasonErrorKind::ValueError,
                self.source_path.clone(),
                self.local_root.clone(),
                "map must have at least one argument",
            ));
        }

        let argument = args.remove(0).token.plain(); // first token
        let index_argument = if !args.is_empty() {
            args.remove(0).token.plain() // second token, if exists
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
                self.types.insert(left.token.plain(), typed_value);
                Ok(None)
            },
            TokenType::FnCall(args) => {
                let typed_args = args
                        
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
                Ok(None)
            },
            _ => {
                Err(
                    self.err(
                        JasonErrorKind::UndefinedVariable(left.token.plain()), 
                        format!("Cannot set Type to value other than ID and Template()")
                    )   
                )
            }
        }
    }

    pub fn eval_string_conversion(&mut self, node:&ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let TokenType::StringConverion(args) = &node.token.token_type {
            let args:&Vec<ASTNode> = args;

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
            
            let result = self.value_to_string(&value)?;
            
            Ok(Some(Value::String(result)))
        }else {
            Err(
                self.err(JasonErrorKind::Custom, format!("reached string conversion from not string conversion"))
            )
        }
    }

    pub fn value_to_string(&self, value: &Value) -> JasonResult<String> {
        match value {
            Value::Number(n) => Ok(n.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::String(s) => Ok(s.clone()),
            Value::Array(v) => {
                let mut result = String::from("[");
                for element in v {
                    let str_result = self.value_to_string(&element)?;
                    result.push_str(&(str_result + ","));
                }
                result.pop();
                result.push_str("]");
                return Ok(result);
            },
            Value::Null => Ok(String::from("null")),
            Value::Object(o) => Ok(serde_json::to_string(&Value::Object(o.clone()))
    .unwrap_or_else(|_| "{}".to_string())),
        }
    }


    pub fn eval_int_conversion(&mut self, node:&ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let TokenType::IntConverion(args) = &node.token.token_type {
            let args:&Vec<ASTNode> = args;

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
                    format!("failed to evaluate value into int")
                )
            )?;

            if let Some(second_value) = args.get(1) {

                let second_value = self.to_json(second_value)?.ok_or_else(|| 
                    JasonError::new(
                        JasonErrorKind::ConversionError, 
                        self.source_path.clone(), 
                        self.local_root.clone(), 
                        format!("Values for int(a, b) must be of Number")
                    )
                )?;


                if let (Value::Number(min), Value::Number(max)) = (&value, second_value) {                    
                    let mut rng = rand::rng();
                    let min = min.as_f64().ok_or_else(||
                        self.err(JasonErrorKind::ConversionError, format!("failed to convert argument one in {} into float", node.plain_sum)))? as i64;

                    let max = max.as_f64().ok_or_else(||
                        self.err(JasonErrorKind::ConversionError, format!("failed to convert argument two in {} into float", node.plain_sum)))? as i64;

                    let int_rand = rng.random_range(min..=max);
                
                    return Ok(
                        Some(
                            Value::Number(
                                int_rand.into()
                            )
                        )
                    );
                }
            }

            self.value_to_int(&value)
            
        }else {
            Err(
                self.err(JasonErrorKind::Custom, format!("reached string conversion from not string conversion"))
            )
        }
    }


    fn value_to_int(&self, value: &Value) -> JasonResult<Option<Value>> {
        match value {
            Value::Number(n) => {
                let i = n.as_f64().ok_or_else(|| {
                    self.err(
                        JasonErrorKind::ConversionError,
                        format!("failed to convert {} to int", value)
                    )
                })? as i64;
                Ok(Some(Value::Number(Number::from(i))))
            }

            Value::String(s) => {
                let i = s.parse::<i64>().map_err(|_| {
                    self.err(
                        JasonErrorKind::ConversionError,
                        format!("failed to convert {} to int", value)
                    )
                })?;
                Ok(Some(Value::Number(Number::from(i))))
            }

            Value::Bool(b) => {
                let i = if *b { 1 } else { 0 };
                Ok(Some(Value::Number(Number::from(i))))
            }

            Value::Array(a) => {
                let mut new_arr = Vec::with_capacity(a.len());
                for e in a {
                    let value = self.value_to_int(&e)?.ok_or_else(||
                        self.err(JasonErrorKind::ConversionError, format!("failed to convert {}", e))
                    )?;
                    new_arr.push(value);

                }
                Ok(Some(Value::Array(new_arr)))
            }

            v => Err(self.err(
                JasonErrorKind::ConversionError,
                format!("Cannot convert type {} into int", v)
            )),
        }
    }

    fn value_to_float(&self, value: &Value) -> JasonResult<Option<Value>> {
        match value {
            Value::Number(n) => {
                let f = n.as_f64().ok_or_else(|| {
                    self.err(
                        JasonErrorKind::ConversionError,
                        format!("failed to convert {} to float", value),
                    )
                })?;
                Ok(Some(Value::Number(
                    Number::from_f64(f).ok_or_else(|| {
                        self.err(
                            JasonErrorKind::ConversionError,
                            "failed to convert number into serde_json Number".to_string(),
                        )
                    })?,
                )))
            }

            Value::String(s) => {
                let f = s.parse::<f64>().map_err(|_| {
                    self.err(
                        JasonErrorKind::ConversionError,
                        format!("failed to convert {} to float", value),
                    )
                })?;
                Ok(Some(Value::Number(
                    Number::from_f64(f).ok_or_else(|| {
                        self.err(
                            JasonErrorKind::ConversionError,
                            "failed to convert number into serde_json Number".to_string(),
                        )
                    })?,
                )))
            }

            Value::Bool(b) => {
                let f = if *b { 1.0 } else { 0.0 };
                Ok(Some(Value::Number(
                    Number::from_f64(f).ok_or_else(|| {
                        self.err(
                            JasonErrorKind::ConversionError,
                            "failed to convert bool into serde_json Number".to_string(),
                        )
                    })?,
                )))
            }

            Value::Array(a) => {
                let mut new_arr = Vec::with_capacity(a.len());
                for e in a {
                    // compute string for error messages
                    let s = self.value_to_string(e)?;
                    let f = e.as_f64().ok_or_else(|| {
                        self.err(
                            JasonErrorKind::ConversionError,
                            format!("failed to convert {} to float", s),
                        )
                    })?;
                    new_arr.push(Value::Number(
                        Number::from_f64(f).ok_or_else(|| {
                            self.err(
                                JasonErrorKind::ConversionError,
                                format!("failed to convert {} into serde_json Number", s),
                            )
                        })?,
                    ));
                }
                Ok(Some(Value::Array(new_arr)))
            }

            v => Err(self.err(
                JasonErrorKind::ConversionError,
                format!("Cannot convert type {} into float", v),
            )),
        }
    }


    pub fn eval_float_conversion(&mut self, node:&ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let TokenType::FloatConverion(args) = &node.token.token_type {
            let args:&Vec<ASTNode> = args;

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
                    format!("failed to evaluate value into int")
                )
            )?;

            if let Some(second_value) = args.get(1) {

                let second_value = self.to_json(second_value)?.ok_or_else(|| 
                    JasonError::new(
                        JasonErrorKind::ConversionError, 
                        self.source_path.clone(), 
                        self.local_root.clone(), 
                        format!("Values for int(a, b) must be of Number")
                    )
                )?;


                if let (Value::Number(min), Value::Number(max)) = (&value, second_value) {                    
                    let mut rng = rand::rng();
                    let min = min.as_f64().ok_or_else(||
                        self.err(JasonErrorKind::ConversionError, format!("failed to convert argument one in {} into float", node.plain_sum)))?;
                    let max = max.as_f64().ok_or_else(||
                        self.err(JasonErrorKind::ConversionError, format!("failed to convert argument two in {} into float", node.plain_sum)))?;

                    let rand:f64 = rng.random_range(min..=max);
                    return Ok(
                        Some(
                            Value::Number(
                                Number::from_f64(rand).ok_or_else(|| 
                                    self.err(
                                        JasonErrorKind::ConversionError,
                                        format!("failed to convert random number into serde_json Number")
                                    )
                                )?
                            )
                        )
                    );
                }
            }
 
            self.value_to_float(&value)
        }else {
            Err(
                self.err(JasonErrorKind::Custom, format!("reached string conversion from not string conversion"))
            )
        }
    }

    pub fn eval_composite_string(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let TokenType::CompositeString(strings, nodes) = &node.token.token_type {
            let mut result = String::new();
            for (i, string) in strings.iter().enumerate() {
                result.push_str(string);  
                if let Some(n) = nodes.get(i) {
                    let node_result = self.to_json(n)?;
                    let value_result = match node_result {
                        Some(res) => self.value_to_string(&res)?,
                        None => return Err(
                            self.err(
                                JasonErrorKind::Custom, 
                                format!("all values in composite string must return a value")
                            )
                        ) 
                    };
                    result.push_str(&value_result);
                }
            } 
            return Ok(Some(Value::String(result))); 
        }
        Err(
            self.err(
                JasonErrorKind::Custom, 
                format!("reached composite string from token that isn't composite string here: {}", node.plain_sum)
            )
        )
    }
    pub fn to_json(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        match &node.token.token_type {
            TokenType::Null => Ok(Some(serde_json::Value::Null)),
            TokenType::Map(_) => self.eval_map(node),

            TokenType::Plus  => self.eval_plus(node),
            TokenType::Minus => self.eval_minus(node),

            TokenType::Mult => self.eval_mult(node),
            TokenType::Divide => self.eval_div(node),
            TokenType::Mod    => self.eval_mod(node),

            TokenType::Repeat => self.eval_repeat(node),
            TokenType::Merge => self.eval_merge(node),
            TokenType::At => self.eval_at(node),
            TokenType::Pick => self.eval_pick(node),
            TokenType::UPick => self.eval_upick(node),
            TokenType::DoubleColon => self.eval_double_colon(node),
            TokenType::StringConverion(_) => self.eval_string_conversion(node),
            TokenType::CompositeString(_, _) => self.eval_composite_string(node),
            TokenType::IntConverion(_) => self.eval_int_conversion(node),
            TokenType::FloatConverion(_) => self.eval_float_conversion(node),
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
            TokenType::IntLiteral(num) => { 
                Ok(Some(Value::Number(Number::from(*num)))) 
            },
            TokenType::FloatLiteral(num) => { 
                let n = Number::from_f64(*num).ok_or_else(|| {
                    JasonError::new(
                        JasonErrorKind::InvalidOperation(num.to_string()),
                        self.source_path.clone(),
                        self.local_root.clone(),
                        "invalid floating-point number",
                    )
                })?;

                Ok(Some(Value::Number(Number::from(n)))) 
            },
            TokenType::Equals => self.eval_equal(node),
            TokenType::Narwhal => self.eval_narwhal(node),
            //TokenType::SpiderWalrus => self.eval_spiderwalrus(node),

            TokenType::SpiderWalrus => self.eval_spiderwalrus(node),
            TokenType::StringLiteral(s) => {
                Ok(Some(serde_json::Value::String(s.to_string())))
            }, 
            TokenType::List(args) => {
                let json_values: Vec<Value> = args
                    .iter()
                    .map(|node| {
                        self.to_json(node)?
                            .ok_or_else(|| JasonError::new(jason_errors::JasonErrorKind::ValueError, self.source_path.clone(), self.local_root.clone(), "List item is None"))
                    })
                    .collect::<JasonResult<Vec<Value>>>()?;                
                Ok(Some(Value::Array(json_values)))
            }
            TokenType::From => self.eval_from(node),


            TokenType::Info => {
                if let Some(right_node) = node.right.as_ref() {
                    let code_line = format!("{:>5} | {}", node.token.row, node.root().plain_sum.clone());
                    let value = self.to_json(right_node)?; 
                    if let Some(v) = value {
                        let value_type = self.infer_type_from(&v)?;
                        println!("{}", format!("┌[Info] Token at line {}, col {}. in file: {}", node.token.row, node.token.colmn, self.source_path.clone()).cyan().bold());
                        println!("{}", "│ Code: ".cyan());
                        println!("{}", "│".cyan());
                        println!("{}", format!("│ {}", code_line).cyan());
                        println!("{}", "│".cyan());
                        println!("{} {}", "│ Value:".cyan(), v);
                        println!("{} {}", "│ Type:".cyan(), value_type.to_string().green());
                        println!("{}", "└─────────────".cyan());
                        return Ok(None);
                    }
                }
                return Err(self.err(
                    JasonErrorKind::MissingValue,
                    "info statements must have right side".to_string(),
                ));
            },
            TokenType::InfoT => {
                if let Some(right_node) = node.right.as_ref() {
                    let code_line = format!("{:>5} | {}", node.token.row, node.root().plain_sum.clone());
                    let value_type = self.to_type(right_node)?;
                    println!("{}", format!("┌[Type-Info] Token at line {}, col {}. in file: {}", node.token.row, node.token.colmn, self.source_path.clone()).cyan().bold());
                    println!("{}", "│ Code: ".cyan());
                    println!("{}", "│".cyan());
                    println!("{}", format!("│ {}", code_line).cyan());
                    println!("{}", "│".cyan());
                    println!("{} {}", "│ Type:".cyan(), value_type.to_string().green());
                    println!("{}", "└─────────────".cyan());
                    return Ok(None);
                }
                return Err(self.err(
                    JasonErrorKind::MissingValue,
                    "info statements must have right side".to_string(),
                ));
            },

            TokenType::Include => {
                 if let Some(right_node) = node.right.as_ref() {
                    if let TokenType::StringLiteral(path) = &right_node.token.token_type {
                        let file_path = Path::new(path);
                        if file_path.exists() {
                            let value = jason_to_json(path)?;
                            return Ok(Some(value));
                        } else {

                            return Err(self.err(JasonErrorKind::Custom, format!("failed to find path {}", path)));
                        }
                    }
                
                    return Err(self.err(JasonErrorKind::TypeError(right_node.plain_sum.clone()), format!("failed to build value from include statement")));
                }
                
                return Err(self.err(JasonErrorKind::MissingValue, format!("include statements must have right side")));
    
            },
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
                let args = args;
                if args.len() > 0 {
                    let args:Vec<String> = args.into_iter().map(|node| node.token.plain()).collect();
                    self.templates.insert(
                        node.token.plain(), 
                        Template::new(
                            &self,
                            node.token.plain(), 
                            args, 
                            block.clone(), 
                            self.template_types.get(&node.token.plain()).cloned()
                        )?
                    );
                    return Ok(None);
                }
                
                

                self.templates.insert(
                    node.token.plain(), 
                    Template::new(
                        &self,
                        node.token.plain(), 
                        Vec::new(), 
                        block.clone(), 
                        self.template_types.get(&node.token.plain()).cloned()
                    )?
                );
                return Ok(None);
            },
            TokenType::LuaFnCall(_) => self.eval_lua_fn(node),
            TokenType::FnCall(args) => { 
                if !self.templates.contains_key(&node.token.plain()) {
                    return Err(JasonError::new(JasonErrorKind::UndefinedTemplate(node.token.plain()), self.source_path.clone(), self.local_root.clone(), format!("the template {} does not exist in file {}", node.token.plain(), self.source_path)));
                }

                let template = self.templates.get(&node.token.plain()).unwrap().clone();
                template.resolve(self, &args)
            },
            token => {
                Err(JasonError::new(JasonErrorKind::SyntaxError, self.source_path.clone(), self.local_root.clone(),
                    format!("Unexpected token: {:?}", token)))
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
        }
        
        for pair in lua_instance.base_env.clone().pairs::<mlua::Value, mlua::Value>() {
            let (k, v) = pair.unwrap();
        }
        */
        Ok(())
    }
    pub fn block_to_json(&mut self, node: &ASTNode) -> JasonResult<Option<serde_json::Value>> {
        if let TokenType::Block(args) = &node.token.token_type {
            let nodes = args;
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

    pub fn add_var(&mut self, key: String, value: serde_json::Value, typing: JasonType) {
        self.variable_types.insert(key.clone(), typing);
        self.variables.insert(key, value);
    }

    pub fn remove_var(&mut self, key: String) {
        self.variable_types.remove(&key);
        self.variables.remove(&key);
    }

    pub fn export(&self, args: Vec<String>) -> Vec<ExportType> {
        let mut exported_values:Vec<ExportType> = Vec::new(); 

        for arg in &args {
            
            if arg == "$" {
                for (name, value) in &self.variables {
                    exported_values.push(ExportType::Variable(name.clone(), value.clone()));
                }
                for (name, value) in &self.variable_types { 
                    exported_values.push(ExportType::VariableType(name.clone(), value.clone()));
                }
                for (name, value) in self.types.clone() {
                    exported_values.push(ExportType::Type(name.clone(), value.clone()));
                }
                for (name, value) in &self.template_types {
                    exported_values.push(ExportType::TemplateType(name.clone(), value.clone()));
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

            if self.variable_types.contains_key(arg) {
                let variable = self.variable_types.get(arg).unwrap().clone();
                exported_values.push(ExportType::VariableType(arg.clone(), variable));
                continue;
            }
            if self.types.contains_key(arg) {
                let variable = self.types.get(arg).unwrap().clone();
                exported_values.push(ExportType::Type(arg.clone(), variable));
                continue;
            }
            if self.template_types.contains_key(arg) {
                let variable = self.template_types.get(arg).unwrap().clone();
                exported_values.push(ExportType::TemplateType(arg.clone(), variable));
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
        for (name, value) in self.variable_types.clone() {
            exported_values.push(ExportType::VariableType(name, value));
        }
        for (name, value) in self.types.clone() {
            exported_values.push(ExportType::Type(name, value));
        }
        for (name, value) in self.template_types.clone() {
            exported_values.push(ExportType::TemplateType(name, value));
        }
;
        exported_values
    }
    pub fn absorb_exports(&mut self, exports: Vec<ExportType>) {
        for exp in exports {
            match exp {
                ExportType::Template(name, template) => {
                    self.templates.insert(name, template);
                },
                ExportType::Variable(name, variable) => {
                    self.variables.insert(name, variable);
                },
                ExportType::TemplateType(name, t) => {
                    self.template_types.insert(name, t);
                },
                ExportType::VariableType(name, t) => {
                    self.variable_types.insert(name, t);
                },
                ExportType::Type(name, t) => {
                    self.types.insert(name, t);
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
