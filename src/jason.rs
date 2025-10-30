use regex::Regex;
use serde_json::Value;
use std::fs;
use std::error;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;


type FileChain =  Rc<RefCell<Vec<String>>>;

fn print_file_chain(file_chain_rc: FileChain) -> String {
    let file_chain = file_chain_rc.borrow();
    
    file_chain.join(" -> ") + " (HEAD)"
}

fn split_arguments(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut start = 0;

    let mut depth_square = 0;
    let mut depth_curly = 0;

    let mut in_string = false;
    let mut escaped = false;

    let mut in_angle = false;
    let mut in_angle_string = false;

    for (i, c) in s.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if in_angle {
            if in_angle_string {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_angle_string = false;
                }
            } else {
                match c {
                    '"' => in_angle_string = true,
                    '>' => in_angle = false, // end of angle block
                    _ => {}
                }
            }
        } else {
            match c {
                '"' => in_string = true,
                '[' => depth_square += 1,
                ']' => depth_square -= 1,
                '{' => depth_curly += 1,
                '}' => depth_curly -= 1,
                '<' => in_angle = true,
                ',' if depth_square == 0 && depth_curly == 0 && !in_angle => {
                    let arg = s[start..i].trim();
                    if !arg.is_empty() {
                        args.push(arg.to_string());
                    }
                    start = i + 1;
                }
                _ => {}
            }
        }
    }

    // Add last argument
    if start < s.len() {
        let arg = s[start..].trim();
        if !arg.is_empty() {
            args.push(arg.to_string());
        }
    }

    args
}

fn expand_json(mut src: String, input_args: Vec<String>, file_chain_rc: FileChain) -> Result<String, Box<dyn error::Error>> {
    // Regex to match ...<...> pattern
    let re = Regex::new(r"<([^>]+)>").unwrap();
    
    let re_arguments = Regex::new(r"(?s)\((.*?)\)\s*([\{\[])").unwrap();
    
    let args:Vec<String> = {
        if let Some(caps) = re_arguments.captures(&src.clone()) {
            let args = &caps[1]; // "name, health"
            let delim = &caps[2]; // '{' or '['
            // Remove parentheses from src if needed
            let stripped = re_arguments.replace(&src, delim);
            src = stripped.into();

            args
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            Vec::new()
        }
    };

    let mut variable_map: HashMap<String, String> = HashMap::new();
    
    if input_args.len() != args.len() {
        println!("input_args: {} , args: {}\n\n", input_args.len(), args.len());
        return Err(format!("input arguments don't match actual arguments\n input_args: {:?} \n  args: {:?}", input_args.join(","), args.join(",")).into());
    }

    for (i, arg) in args.into_iter().enumerate() {
        if arg.trim() == "" {
            continue;
        }
        variable_map.insert(arg, input_args[i].clone());
    }

    for key in variable_map.keys() {
        src = src.replace(&format!("#{}", key), variable_map.get(key).unwrap());
    }

    let replaced = re.replace_all(&src, |caps: &regex::Captures| {
        let inner_content = &caps[1];

        if inner_content.contains("|") {
            let contents: Vec<String> = inner_content.splitn(2, '|').map(|s| s.trim().to_string()).collect();
            let file = &contents[0].trim();
            let arguments = split_arguments(&contents[1]);
            println!("arguments3: {:?}", arguments);
            expand_json_from_file(file, arguments, file_chain_rc.clone()).expect(
                &format!("[ERROR] head file: {}", print_file_chain(file_chain_rc.clone())) 
            )
                .to_string()
        } else {
            expand_json_from_file(&inner_content.trim(), vec![], file_chain_rc.clone())
                .expect(&format!("[ERROR] head file: {}", print_file_chain(file_chain_rc.clone()))).to_string()

        }
    });

    Ok(replaced.to_string())
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
    prettify_json(&src)
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

