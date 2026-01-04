
use std::fs;
use serde_json::*;

use jason_rs::{JasonBuilder, jason_src_to_json, jason_to_json};

use serde_json::json;

#[test]
fn test_type_composition_from_file() {
    let jason = include_str!("../examples/type_composition.jason");
    
    let result = JasonBuilder::new()
        .jason_src_to_json(jason)
        .expect("failed to compile");
    
    let expected = json!([
        {"type":"a","value":42},
        {"type":"b","value":"hello"},
        {"extra":true,"type":"a","value":10}
    ]);
    
    assert_eq!(result, expected);
}


#[test]
fn debug_single_test() {
    let jason = r#"
        A :: { type : "a", value : Number }
        B :: { type : "b", value : String }
        C :: A + { extra : Bool }
        Result :: A | B | C
        r1 : Result = { type : "a", value : 42 }
        r2 : Result = { type : "b", value : "hello" }
        r3 : Result = { type : "a", value : 10, extra : true }
        out [r1, r2, r3]
    "#;
    
    // Debug output
    eprintln!("=== DEBUG INFO ===");
    eprintln!("String length: {}", jason.len());
    eprintln!("String bytes: {}", jason.as_bytes().len());
    eprintln!("First 50 chars: {:?}", &jason.chars().take(50).collect::<String>());
    eprintln!("Last 50 chars: {:?}", &jason.chars().rev().take(50).collect::<String>());
    
    // Check for hidden characters
    for (i, ch) in jason.chars().enumerate().take(100) {
        if !ch.is_ascii_graphic() && ch != ' ' && ch != '\n' && ch != '\t' {
            eprintln!("Non-standard char at position {}: {:?} (U+{:04X})", i, ch, ch as u32);
        }
    }
    
    let result = JasonBuilder::new()
        .jason_src_to_json(jason);
    
    match result {
        Ok(val) => {
            let expected = json!([
                {"type":"a","value":42},
                {"type":"b","value":"hello"},
                {"extra":true,"type":"a","value":10}
            ]);
            assert_eq!(val, expected);
            println!("✓ Test passed!");
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            panic!("Test failed");
        }
    }
}


//FIX
//outputs: 
// Error: Unclosed '{' in composite string at 2 7

#[test]
fn test_composite_string_with_object() {
    let jason = r#"
        out $"Result: {name: \"Alex\", age: 20}"
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = Value::String(r#"Result: {{name: "Alex", age:20}}"#.to_string());
    assert_eq!(result, expected);
}
//FIX 
//outputs: [[2],[2,2],[2,2,2]]
#[test]
fn test_map_operator_simple() {
    let jason = r#"
        out [1, 2, 3] map(n) (n * 2)
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    // Each element multiplied by 2 means repeated 2 times
    let expected = json!([[1, 1], [2, 2], [3, 3]]);
    
    assert_eq!(result, expected)
}

#[test]
fn test_type_composition() {
    let jason = r#"
        A :: { type : "a", value : Number }
        B :: { type : "b", value : String }
        C :: A + { extra : Bool }
        Result :: A | B | C
        r1 : Result = { type : "a", value : 42 }
        r2 : Result = { type : "b", value : "hello" }
        r3 : Result = { type : "a", value : 10, extra : true }
        out [r1, r2, r3]
    "#;
    
    let result = JasonBuilder::new()  // <- USE THIS
        .jason_src_to_json(jason)
        .expect("failed to compile");
    
    let expected = json!([
        {"type": "a", "value": 42},
        {"type": "b", "value": "hello"},
        {"extra": true, "type": "a", "value": 10}
    ]);
    
    assert_eq!(result, expected);
}

//FIX
//outputs: ["a","b","c"]
#[test]
fn test_map_operator_with_index() {
    let jason = r#"
        out ["a", "b", "c"] map(n, i) {value: n, index: i}
    "#;
    
    let expected = json!([
        {"value": "a", "index": 0},
        {"value": "b", "index": 1},
        {"value": "c", "index": 2}
    ]);


    let result = jason_src_to_json(jason).expect("failed to compile");
     
    assert_eq!(result, expected);
}

#[test]
fn test_merge_operator() {
    let jason = r#"Base :: {
        server: {
            http: {
                host: String,
                port: Int,
                headers: {
                    cors: Bool,
                    cache: Bool
                }
            },
            https: {
                port: Int
            }
        },
        logging: {
            level: String,
            format: String
        }
    }

    Prod :: {
        server: {
            http: {
                host: String,
                headers: {
                    cache: Bool
                }
            },
            https: {
                enabled: Bool
            }
        },
        logging: String
    }


    base := {
        server: {
            http: {
                host: "localhost",
                port: 8080,
                headers: {
                    cors: true,
                    cache: false
                }
            },
            https: {
                enabled: false,
                port: 8443
            }
        },
        logging: {
            level: "info",
            format: "json"
        }
    }


    prod:Prod = {
        server: {
            http: {
                host: "prod.example.com",
                headers: {
                    cache: true
                }
            },
            https: {
                enabled: true
            }
        },
        logging: "stdout"
    }

    result: Base & Prod = {
        logging: {format: "format", level: "level"} , 
        server: {
            http: {
                headers: {
                    cache: false, 
                    cors: true
                }, 
            host: "gator", 
            port: 8000
            }, 
            https: {
                enabled: true,
                port: 8000
            }
        }
    }

     out base & prod"#;
     let expected = json!({
        "logging": {
            "format": "json",
            "level": "info",
        },
        "server": {
            "http": {
                "headers": {
                    "cache": false,
                    "cors": true,
                },
                "host": "localhost",
                "port": 8080,
            },
            "https": {
                "enabled": false,
                "port": 8443,
            },
        },
     });
 
 
     let result = jason_src_to_json(jason).expect("failed to compile");

     assert!(result == expected);

}
