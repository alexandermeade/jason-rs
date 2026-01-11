use jason_rs::{JasonBuilder, jason_src_to_json};
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
        out $"Result: {{name: "Alex", age: 20}}"
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let result_str = result.as_str().unwrap();
    
    // Extract the JSON part and re-serialize it in a consistent order
    let prefix = "Result: ";
    assert!(result_str.starts_with(prefix));
    
    let json_part = &result_str[prefix.len()..];
    let parsed: serde_json::Value = serde_json::from_str(json_part).unwrap();
    let normalized = serde_json::to_string(&parsed).unwrap();
    
    let expected_json = serde_json::json!({"name": "Alex", "age": 20});
    let expected_normalized = serde_json::to_string(&expected_json).unwrap();
    
    assert_eq!(format!("{}{}", prefix, normalized), format!("{}{}", prefix, expected_normalized));
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
    let expected = json!([2,4,6]);
    
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
    
    let result = JasonBuilder::new()
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

// ============= MATH TESTS =============

#[test]
fn test_basic_integer_arithmetic() {
    let jason = r#"
        addition := 5 + 3
        subtraction := 10 - 4
        multiplication := 6 * 7
        modulo := 10 % 3
        out {
            add: addition,
            sub: subtraction,
            mul: multiplication,
            mod: modulo
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "add": 8,
        "sub": 6,
        "mul": 42,
        "mod": 1
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_division_operations() {
    let jason = r#"
        int_div := 15 / 3
        float_div := 10.0 / 4.0
        mixed_div := 10 / 4
        out {
            int: int_div,
            float: float_div,
            mixed: mixed_div
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "int": 5.0,
        "float": 2.5,
        "mixed": 2.5
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_float_arithmetic() {
    let jason = r#"
        addition := 5.5 + 2.3
        subtraction := 10.0 - 3.5
        multiplication := 2.5 * 4.0
        division := 10.0 / 4.0
        out {
            add: addition,
            sub: subtraction,
            mul: multiplication,
            div: division
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "add": 7.8,
        "sub": 6.5,
        "mul": 10.0,
        "div": 2.5
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_mixed_int_float_arithmetic() {
    let jason = r#"
        int_plus_float := 5 + 2.5
        float_minus_int := 10.0 - 3
        int_mult_float := 4 * 2.5
        int_div_float := 10 / 4.0
        out {
            add: int_plus_float,
            sub: float_minus_int,
            mul: int_mult_float,
            div: int_div_float
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "add": 7.5,
        "sub": 7.0,
        "mul": 10.0,
        "div": 2.5
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_type_preservation() {
    let jason = r#"
        int_result := 5 + 3
        float_result := 5.0 + 3.0
        mixed_result := 5 + 3.0
        division_result := 10 / 2
        out {
            int: int_result,
            float: float_result,
            mixed: mixed_result,
            div: division_result
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "int": 8,
        "float": 8.0,
        "mixed": 8.0,
        "div": 5.0
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_negative_numbers() {
    let jason = r#"
        neg_add := -5 + 3
        neg_sub := 10 - (-5)
        neg_mult := -10 * 2
        neg_div := -20 / 4
        double_neg := -5 + (-3)
        out {
            add: neg_add,
            sub: neg_sub,
            mul: neg_mult,
            div: neg_div,
            double: double_neg
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "add": -2,
        "sub": 15,
        "mul": -20,
        "div": -5.0,
        "double": -8
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_order_of_operations() {
    let jason = r#"
        result1 := 2 + 3 * 4
        result2 := 10 - 6 / 2
        result3 := 8 / 2 + 3
        result4 := 5 * 2 - 3
        result5 := 20 / 4 + 1
        out {
            r1: result1,
            r2: result2,
            r3: result3,
            r4: result4,
            r5: result5
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "r1": 14,
        "r2": 7.0,
        "r3": 7.0,
        "r4": 7,
        "r5": 6.0
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_parentheses_precedence() {
    let jason = r#"
        without_parens := 2 + 3 * 4
        with_parens := (2 + 3) * 4
        nested := ((2 + 3) * 4) - 5
        complex := (10 - 2) / (3 + 1)
        multi_level := ((5 + 3) * 2) / (4 - 2)
        out {
            without: without_parens,
            _with: with_parens,
            nested: nested,
            complex: complex,
            multi: multi_level
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "without": 14,
        "_with": 20,
        "nested": 15,
        "complex": 2.0,
        "multi": 8.0
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_math_in_objects() {
    let jason = r#"
        Person(name, age) {
            name: name,
            age: age,
            special_num: (1 + age) / 3,
            double_age: age * 2,
            age_minus_ten: age - 10
        }
        out Person("alex", 20)
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "name": "alex",
        "age": 20,
        "special_num": 7.0,
        "double_age": 40,
        "age_minus_ten": 10
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_math_in_lists() {
    let jason = r#"
        out [1 + 1, 2 * 3, 10 / 2, 15 - 5, 10 % 3]
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!([2, 6, 5.0, 10, 1]);
    
    assert_eq!(result, expected);
}

#[test]
fn test_complex_expressions() {
    let jason = r#"
        a := 5
        b := 10
        c := 3
        result1 := (a + b) * c - 5
        result2 := a * (b - c) + 2
        result3 := (a + b) / (c - 1)
        result4 := ((a * b) - c) / 2
        out {
            r1: result1,
            r2: result2,
            r3: result3,
            r4: result4
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "r1": 40,
        "r2": 37,
        "r3": 7.5,
        "r4": 23.5
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_math_with_map() {
    let jason = r#"
        out [1, 2, 3, 4, 5] map(n) (n * 2 + 1)
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!([3, 5, 7, 9, 11]);
    
    assert_eq!(result, expected);
}

#[test]
fn test_division_by_zero_error() {
    let jason = r#"
        out 5 / 0
    "#;
    
    let result = jason_src_to_json(jason);
    assert!(result.is_err(), "Division by zero should produce an error");
}

#[test]
fn test_modulo_by_zero_error() {
    let jason = r#"
        out 10 % 0
    "#;
    
    let result = jason_src_to_json(jason);
    assert!(result.is_err(), "Modulo by zero should produce an error");
}

#[test]
fn test_chained_operations() {
    let jason = r#"
        result := 1 + 2 + 3 + 4 + 5
        result2 := 100 - 10 - 5 - 3
        result3 := 2 * 3 * 4
        result4 := 100 / 10 / 2
        out {
            sum: result,
            diff: result2,
            prod: result3,
            div: result4
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "sum": 15,
        "diff": 82,
        "prod": 24,
        "div": 5.0
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_math_with_type_annotations() {
    let jason = r#"
        Calculator :: {
            add: Int,
            multiply: Int,
            divide: Number,
            modulo: Int
        }
        calc: Calculator = {
            add: 5 + 3,
            multiply: 4 * 7,
            divide: 20 / 4,
            modulo: 17 % 5
        }
        out calc
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "add": 8,
        "multiply": 28,
        "divide": 5.0,
        "modulo": 2
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_modulo_with_floats() {
    let jason = r#"
        int_mod := 17 % 5
        float_mod := 17.5 % 5.0
        mixed_mod := 17 % 5.0
        out {
            int: int_mod,
            float: float_mod,
            mixed: mixed_mod
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "int": 2,
        "float": 2.5,
        "mixed": 2.0
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_nested_arithmetic_expressions() {
    let jason = r#"
        result := (((10 + 5) * 2) - 3) / 3
        out result
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!(9.0);
    
    assert_eq!(result, expected);
}

#[test]
fn test_arithmetic_with_variables() {
    let jason = r#"
        x := 10
        y := 5
        z := 2
        sum := x + y + z
        diff := x - y - z
        prod := x * y * z
        complex := (x + y) * z - x / y
        out {
            sum: sum,
            diff: diff,
            prod: prod,
            complex: complex
        }
    "#;
    
    let result = jason_src_to_json(jason).expect("failed to compile");
    let expected = json!({
        "sum": 17,
        "diff": 3,
        "prod": 100,
        "complex": 28.0
    });
    
    assert_eq!(result, expected);
}
