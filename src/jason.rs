use regex::Regex;
use serde_json::Value;
use std::fs;
use std::error;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use crate::lexer;
use crate::parser;
use crate::token;


type FileChain =  Rc<RefCell<Vec<String>>>;

fn expand_json(mut src: String, input_args: Vec<String>, file_chain_rc: FileChain) -> Result<String, Box<dyn error::Error>> {
    let toks = lexer::Lexer::start(src);

    println!("toks: {:#?}", toks);    
    let nodes = parser::Parser::start(toks);
    println!("result: {:#?}", nodes);   
    
    println!("result -----------");

    for node in nodes {

        println!("{}", &node.to_json());
        println!("{}", serde_json::to_string_pretty(&node.to_json()).unwrap());
    }
    
    println!("over");
    Ok(String::from(""))
}

fn expand_json_from_file(file_path: &str, input_args: Vec<String>, file_chain_rc: FileChain) -> Result<String, Box<dyn error::Error>> {
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
    
    {// limit scope of borrow mut
        let mut file_chain = file_chain_rc.borrow_mut();
        let cond = file_path != match file_chain.last() {
            Some(s) => s,
            None => "" 
        }.to_string();
        
        if file_chain.contains(&file_path.to_string()) && cond {
            return Err(format!("Path: {} has already been traveled to", file_path).into())
        }

        file_chain.push(file_path.to_string());
    }



    let contents = fs::read_to_string(file_path).unwrap();    

    expand_json(contents, input_args, file_chain_rc.clone())
}

fn prettify_json(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Parse using JSON5 so unquoted keys are allowed
    let parsed: Value = json5::from_str(input).unwrap();
    
    // Pretty-print with standard indentation
    let pretty = serde_json::to_string_pretty(&parsed).unwrap();
    
    Ok(pretty)
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
pub fn jason_to_json(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let src = expand_json_from_file(file_path, vec![], Rc::new(RefCell::new(vec![]))).unwrap();
    Ok(src.to_string())
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
    let src = expand_json_from_file(file_path, vec![], Rc::new(RefCell::new(vec![])))?;
    
    let parsed: Value = json5::from_str(&src)?;
    
    let yaml_string = serde_yaml::to_string(&parsed)?;
    
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
    let src = expand_json_from_file(file_path, vec![], Rc::new(RefCell::new(vec![])))?;

    let parsed: Value = json5::from_str(&src)?;
    
    let toml_string = toml::to_string(&parsed)?;
    
    Ok(toml_string)
}

