use crate::jason_hidden::{compile_jason_from_src, compile_jason_from_file};
use crate::lua_instance::LuaInstance;
use crate::jason_errors::{JasonError};
use std::rc::Rc;
use std::cell::RefCell;

pub type CompilerResult<T> = Result<T, JasonError>;

/// Builder for constructing Jason parsing with optional Lua dependencies.
pub struct JasonBuilder {
    lua_src: String,
}

impl JasonBuilder {
    
    /// Creates a new JasonBuilder with no Lua dependencies.
    ///
    /// # Example
    /// ```rust,ignore
    /// use jason_rs::JasonBuilder;
    /// let builder = JasonBuilder::new();
    /// ```
    pub fn new() -> Self {
        JasonBuilder { lua_src: String::new() }   
    }

    /// Includes a Lua file as a dependency for `.jason` parsing.
    ///
    /// # Arguments
    /// * `file_path` - Path to the Lua file to include.
    ///
    /// # Errors
    /// Returns an error if reading the Lua file fails.
    ///
    /// # Example
    /// ```rust,ignore
    /// use jason_rs::JasonBuilder;
    /// let builder = JasonBuilder::new().include_lua_file("scripts/helpers.lua").unwrap();
    /// ```
    pub fn include_lua_file(mut self, file_path: &'static str) -> CompilerResult<JasonBuilder> {
        let src = std::fs::read_to_string(file_path);
        if !src.is_ok() {
            return Err(JasonError::new(crate::jason_errors::JasonErrorKind::FileError, Rc::new(file_path.to_string()), None, format!("failed to read from file {}", file_path)));
        }
        let src = src.unwrap(); 
        self.lua_src.push_str(&src); 
        Ok(self)
    }

    /// Includes raw Lua source code as a dependency for `.jason` parsing.
    ///
    /// # Arguments
    /// * `src` - Lua source code as a string.
    ///
    /// # Example
    /// ```rust, ignore
    /// use jason_rs::JasonBuilder;
    /// let lua_code = r#"function add(a,b) return a+b end"#;
    /// let builder = JasonBuilder::new().include_lua(lua_code)?;
    /// ```
    pub fn include_lua(mut self, src: &'static str) -> CompilerResult<JasonBuilder> {
        self.lua_src.push_str(&src); 
        Ok(self)
    }

    /// Converts a `.jason` file into a JSON value using the Lua dependencies included in the builder.
    ///
    /// # Arguments
    /// * `file_path` - Path to the `.jason` file.
    ///
    /// # Errors
    /// Returns an error if reading or parsing the `.jason` file fails.
    ///
    /// # Example
    /// ```rust,ignore
    /// use jason_rs::JasonBuilder;
    /// let json = JasonBuilder::new().jason_to_json("Page.jason").unwrap();
    /// println!("{}", json);
    /// ```
    
    pub fn jason_to_json(self, file_path: &str) -> CompilerResult<serde_json::Value> {
        let lua = Rc::new(RefCell::new(LuaInstance::new_with_src(self.lua_src)?));
        let json = compile_jason_from_file(file_path, lua)?;
        Ok(json)
    }

    /// Converts raw `.jason` source into a JSON value using the Lua dependencies included in the builder.
    ///
    /// # Arguments
    /// * `src` - `.jason` source code as a string.
    ///
    /// # Errors
    /// Returns an error if parsing fails.
    ///
    /// # Example
    /// ```rust,ignore
    /// use jason_rs::JasonBuilder;
    /// let src = r#"out {name: "alex", age: 20}"#;
    /// let json = JasonBuilder::new().jason_src_to_json(src)?;
    /// println!("{}", json);
    /// ```
    pub fn jason_src_to_json(self, src: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let lua = Rc::new(RefCell::new(LuaInstance::new_with_src(self.lua_src)?));
        let json = compile_jason_from_src(src, lua)?;
        Ok(json)
    }
}

/// Converts a `.jason` file into JSON using a default Lua environment.
///
/// # Arguments
/// * `file_path` - Path to the `.jason` file.
///
/// # Errors
/// Returns an error if reading or parsing fails.
///
/// # Example
/// ```rust,ignore
/// use jason_rs::jason_to_json;
/// let json = jason_to_json("Page.jason")?;
/// println!("{}", json);
/// ```
pub fn jason_to_json(file_path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let lua = Rc::new(RefCell::new(LuaInstance::new()?));
    let json = compile_jason_from_file(file_path, lua)?;
    Ok(json)
}

/// Converts raw `.jason` source into JSON using a default Lua environment.
///
/// # Arguments
/// * `src` - `.jason` source code as a string.
///
/// # Errors
/// Returns an error if parsing fails.
///
/// # Example
/// ```rust,ignore
/// use jason_rs::jason_src_to_json;
/// let src = r#"out {name: "alex", age: 20}"#;
/// let json = jason_src_to_json(src).unwrap();
/// println!("{}", json);
/// ```
pub fn jason_src_to_json(src: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let lua = Rc::new(RefCell::new(LuaInstance::new()?));
    let json = compile_jason_from_src(src, lua)?;
    Ok(json)
}

