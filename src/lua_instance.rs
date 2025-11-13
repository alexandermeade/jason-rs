use mlua::{Lua, Table, StdLib, Result};
use rand::Rng;
use include_dir::{include_dir, Dir};
use crate::context::Context;

static BASE_LUA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/exposed_lua_files");

fn load_all_base_lua() -> String {
    let mut contents = String::new();
    for file in BASE_LUA_DIR.files() {
        if let Some(ext) = file.path().extension().and_then(|s| s.to_str()) {
            if ext == "lua" {
                contents.push_str(file.contents_utf8().unwrap());
            }
        }
    }
    contents
}

#[derive(Debug)]    
pub struct LuaInstance {
    pub lua_instance: Lua,
    pub base_env: Table,
}

impl LuaInstance {
    pub fn new() -> Result<Self> {
        let lua = Lua::new_with(
            StdLib::ALL_SAFE,
            Default::default(),
        )?;
        
        let base_env = lua.create_table()?;
        
        // Make base_env inherit from globals so it has access to math, os, etc.
        let mt = lua.create_table()?;
        mt.set("__index", lua.globals())?;
        base_env.set_metatable(Some(mt));
        
        let code = load_all_base_lua();
        let mut rng = rand::rng();      // renamed from thread_rng
        let seed: i64 = rng.random();   // renamed from gen
        lua.globals().set("SAFE_SEED", seed)?;
        
        // Seed the random number generator
        lua.load(r#"
            math.randomseed(SAFE_SEED)
        "#).exec()?;
        
        lua.load(code)
            .set_environment(base_env.clone())
            .exec()?;
        
        Ok(LuaInstance {
            lua_instance: lua,
            base_env
        })
    }
    
    pub fn import_from_base(&self, context: Context, key: &str) -> mlua::Result<()> {
        let val: mlua::Value = self.base_env.get(key)?;
        context.lua_env.set(key, val)?;
        Ok(())
    }
    
    pub fn create_environment(&self, name: &'static str) -> Result<Table> {
        let lua = &self.lua_instance;
        // Create a new isolated environment
        let env = lua.create_table()?;
        // Inherit from base environment (which inherits from globals)
        let mt = lua.create_table()?;
        mt.set("__index", self.base_env.clone())?;
        env.set_metatable(Some(mt));
        // Optionally store the name for debugging or tracking
        env.set("_NAME", name)?;
        Ok(env)
    }
    pub fn lua_value_to_json(value: &mlua::Value) -> serde_json::Value {
        match value {
            mlua::Value::Nil => serde_json::Value::Null,
            mlua::Value::Boolean(b) => serde_json::Value::Bool(*b),
            mlua::Value::Integer(i) => serde_json::Value::Number((*i).into()),
            mlua::Value::Number(n) => {
                serde_json::Number::from_f64(*n)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            },
            mlua::Value::String(s) => {
                match s.to_str() {
                    Ok(str_val) => serde_json::Value::String(str_val.to_string()),
                    Err(_) => serde_json::Value::Null,
                }
            },
            mlua::Value::Table(table) => {
                // Try to detect if it's an array or object
                if Self::is_lua_array(table) {
                    let arr: Vec<serde_json::Value> = table.clone()
                        .sequence_values()
                        .filter_map(|v| v.ok())
                        .map(|v| Self::lua_value_to_json(&v))
                        .collect();
                    serde_json::Value::Array(arr)
                } else {
                    let mut map = serde_json::Map::new();
                    for pair in table.clone().pairs::<mlua::Value, mlua::Value>() {
                        if let Ok((key, val)) = pair {
                            if let mlua::Value::String(k) = key {
                                if let Ok(key_str) = k.to_str() {
                                    map.insert(
                                        key_str.to_string(),
                                        Self::lua_value_to_json(&val)
                                    );
                                }
                            }
                        }
                    }
                    serde_json::Value::Object(map)
                }
            },
            _ => serde_json::Value::Null, // Functions, threads, userdata
        }
    }

    pub fn json_to_lua_value(&self, value: serde_json::Value) -> mlua::Result<mlua::Value> {
        match value {
            serde_json::Value::Null => Ok(mlua::Value::Nil),
            serde_json::Value::Bool(b) => Ok(mlua::Value::Boolean(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(mlua::Value::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(mlua::Value::Number(f))
                } else {
                    Ok(mlua::Value::Nil)
                }
            },
            serde_json::Value::String(s) => {
                let lua_string = self.lua_instance.create_string(&s)?;
                Ok(mlua::Value::String(lua_string))
            },
            serde_json::Value::Array(arr) => {
                let table = self.lua_instance.create_table()?;
                for (i, val) in arr.iter().enumerate() {
                    let lua_val = self.json_to_lua_value(val.clone())?;
                    table.set(i + 1, lua_val)?;
                }
                Ok(mlua::Value::Table(table))
            },
            serde_json::Value::Object(obj) => {
                let table = self.lua_instance.create_table()?;
                for (key, val) in obj.iter() {
                    let lua_val = self.json_to_lua_value(val.clone())?;
                    table.set(key.as_str(), lua_val)?;
                }
                Ok(mlua::Value::Table(table))
            }
        }
    }
    fn is_lua_array(table: &mlua::Table) -> bool {
        // Check if all keys are consecutive integers starting from 1
        let len = table.len().unwrap_or(0);
        len > 0 && table.contains_key(1).unwrap_or(false)
    }
}
