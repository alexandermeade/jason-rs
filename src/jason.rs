use std::{fs, error};
use serde_json::Value;
use crate::{context::Context, lexer, parser};

pub fn jason_context_from_file(file_path: String) -> Result<Context, Box<dyn error::Error>> {
    match fs::metadata(file_path.clone()) {
        Ok(_) => {},
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(format!("Path does not exist. {}", file_path).into());
            } else {

                return Err(e.into());
            }
        }
    }

    let src = fs::read_to_string(file_path.clone()).unwrap();    
    let toks = lexer::Lexer::start(src);

    let nodes = parser::Parser::start(toks);
        
    let mut context = Context::new(file_path);

    for node in nodes {
        //println!("{:?}", context.to_json(&node));
        context.to_json(&node);
    } 

    
    Ok(context) 
}

fn compile_jason_from_file(file_path: &str) -> Result<serde_json::Value, Box<dyn error::Error>> {
    match fs::metadata(file_path) {
        Ok(_) => {},
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(format!("Path does not exist. {}", file_path).into());
            } else {

                return Err(e.into());
            }
        }
    }


    let context = jason_context_from_file(file_path.into()).unwrap();
    Ok(context.out)
}


/// Converts a `.jason` file into pretty JSON.
///
/// # Arguments
/// * `file_path` - Path to the `.jason` file.
///
/// # Errors
/// Returns an error if reading or parsing fails.
///
/// # Example
/// ```
/// use jason_rs::jason_to_json;
/// let json_text = jason_to_json("Page.jason").unwrap();
/// println!("{}", json_text);
/// ```
pub fn jason_to_json(file_path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let json = compile_jason_from_file(file_path).unwrap();
    Ok(json)
    //prettify_json(&src) 
}


/// Converts a `.jason` file into YAML.
/// 
/// # Caution
/// Has yet to be fully tested!
///
/// # Arguments
/// * `file_path` - Path to the `.jason` file.
///
/// # Errors
/// Returns an error if reading or parsing fails.
///
/// # Example
/// ```
/// use jason_rs::jason_to_yaml;
/// let yaml_text = jason_to_yaml("Page.jason").unwrap();
/// println!("{}", yaml_text);
/// ```
pub fn jason_to_yaml(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let json = compile_jason_from_file(file_path)?;
        
    let yaml_string = serde_yaml::to_string(&json)?;
    
    Ok(yaml_string)
}


/// Converts a `.jason` file into TOML.
///
/// # Caution
/// This may break due to jason not preforming type checking on produced toml and has yet to be tested!
///
/// # Arguments
/// * `file_path` - Path to the `.jason` file.
///
/// # Errors
/// Returns an error if reading or parsing fails.
///
/// # Example
/// ```
/// use jason_rs::jason_to_toml;
/// let toml_text = jason_to_toml("Page.jason").unwrap();
/// println!("{}", toml_text);
/// ```
///
pub fn jason_to_toml(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let json = compile_jason_from_file(file_path)?;

    let parsed: Value = json;
    
    let toml_string = toml::to_string(&parsed)?;
    
    Ok(toml_string)
}

