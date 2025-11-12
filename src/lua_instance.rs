use mlua::{Lua, Table, Value, StdLib, Result};
use std::collections::HashMap;
use std::fs;
use std::ffi::OsStr; 
use include_dir::*;

use include_dir::{include_dir, Dir};

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
        let code = load_all_base_lua();

        lua.load(code)
        .set_environment(base_env.clone())
        .exec()?;
        
        Ok(LuaInstance {
            lua_instance: lua,
            base_env
        })

    }

    pub fn create_environment(&mut self, name: &'static str) -> Result<()> {
        let env = self.lua_instance.create_table()?;

        // Execute the shared module in this environment
        env.set_metatable(Some(self.base_env.clone()));
        Ok(())
    }

}
