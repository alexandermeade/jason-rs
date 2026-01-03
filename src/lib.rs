#![doc = include_str!("../README.md")]

pub mod jason;
mod jason_hidden;
mod lexer; 
mod token;
mod parser;
mod context;
mod template;
mod astnode;
mod lua_instance;
mod jason_errors;
mod jason_types;
pub use jason::*;

use serde_json::{json, Value};

/// Helper function to parse jason source and compare with expected JSON
fn assert_jason_eq(jason_src: &str, expected: Value) {
    let result = JasonBuilder::new()
        .jason_src_to_json(jason_src)
        .expect("Failed to parse jason");
    assert_eq!(result, expected, "Jason output doesn't match expected JSON");
}

#[test]
fn test_basic_object() {
    let jason = r#"
        out {
            name: "Alex",
            project: "jason-rs",
            money: 0
        }
    "#;
    
    let expected = json!({
        "name": "Alex",
        "project": "jason-rs",
        "money": 0
    });
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_variables() {
    let jason = r#"
        project_name = "jason-rs"
        author = "Alex"
        version = 1.0
        
        out {
            project: project_name,
            author: author,
            version: version
        }
    "#;
    
    let expected = json!({
        "project": "jason-rs",
        "author": "Alex",
        "version": 1.0
    });
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_simple_template() {
    let jason = r#"
        Dev(name, project, money) {
            name: name,
            project: project,
            money: money
        }
        
        out Dev("alex", "jason-rs", 0)
    "#;
    
    let expected = json!({
        "name": "alex",
        "project": "jason-rs",
        "money": 0
    });
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_template_multiple_invocations() {
    let jason = r#"
        Person(name, age) {
            name: name,
            age: age
        }
        
        out [
            Person("Alice", 30),
            Person("Bob", 25),
            Person("Charlie", 35)
        ]
    "#;
    
    let expected = json!([
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25},
        {"name": "Charlie", "age": 35}
    ]);
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_object_concatenation() {
    let jason = r#"
        base = {name: "Alex"}
        extra = {age: 20}
        
        out base + extra
    "#;
    
    let expected = json!({
        "name": "Alex",
        "age": 20
    });
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_object_concatenation_override() {
    let jason = r#"
        out {a: 1, b: 2} + {b: 3, c: 4}
    "#;
    
    let expected = json!({
        "a": 1,
        "b": 3,
        "c": 4
    });
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_array_concatenation() {
    let jason = r#"
        out [1, 2, 3] + [4, 5]
    "#;
    
    let expected = json!([1, 2, 3, 4, 5]);
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_string_concatenation() {
    let jason = r#"
        out "hello" + " world"
    "#;
    
    let expected = json!("hello world");
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_string_concatenation_unicode() {
    let jason = r#"
        out "😀" + "🚀"
    "#;
    
    let expected = json!("😀🚀");
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_composite_string_simple() {
    let jason = r#"
        name = "Alex"
        out $"Hello, {name}!"
    "#;
    
    let expected = json!("Hello, Alex!");
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_composite_string_with_object() {
    let jason = r#"
        out $"Result: {{name: \"Alex\", age: 20}}"
    "#;
    
    // The output should be a string representation of the JSON
    let result = JasonBuilder::new()
        .jason_src_to_json(r#"out $"Result: {{name: \"Alex\", age: 20}}""#)
        .expect("Failed to parse");
    
    // Check that it's a string containing JSON
    assert!(result.is_string());
    assert!(result.as_str().unwrap().contains("Result:"));
}

#[test]
fn test_repeat_operator() {
    let jason = r#"
        out "hello" repeat 3
    "#;
    
    let expected = json!(["hello", "hello", "hello"]);
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_multiply_operator() {
    let jason = r#"
        out 5 * 4
    "#;
    
    // The * operator with static values should create a list
    let expected = json!([5, 5, 5, 5]);
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_at_operator_string() {
    let jason = r#"
        out "alex" at 0
    "#;
    
    let expected = json!("a");
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_at_operator_array() {
    let jason = r#"
        out ["alex", "jason"] at 1
    "#;
    
    let expected = json!("jason");
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_at_operator_object() {
    let jason = r#"
        out {name: "alex", age: 20} at "age"
    "#;
    
    let expected = json!(20);
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_map_operator_simple() {
    let jason = r#"
        out [1, 2, 3] map(n) n * 2
    "#;
    
    // Each element multiplied by 2 means repeated 2 times
    let expected = json!([[1, 1], [2, 2], [3, 3]]);
    
    assert_jason_eq(jason, expected);
}

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
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_map_with_template() {
    let jason = r#"
        Item(value) {
            item: value
        }
        
        out [1, 2, 3] map(n) Item(n)
    "#;
    
    let expected = json!([
        {"item": 1},
        {"item": 2},
        {"item": 3}
    ]);
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_type_conversion_str() {
    let jason = r#"
        out str(123)
    "#;
    
    let expected = json!("123");
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_type_conversion_int() {
    let jason = r#"
        out int(3.7)
    "#;
    
    let expected = json!(3);
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_type_conversion_float() {
    let jason = r#"
        out float(5)
    "#;
    
    let expected = json!(5.0);
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_nested_templates() {
    let jason = r#"
        Address(street, city) {
            street: street,
            city: city
        }
        
        Person(name, address) {
            name: name,
            address: address
        }
        
        out Person("Alex", Address("123 Main St", "Springfield"))
    "#;
    
    let expected = json!({
        "name": "Alex",
        "address": {
            "street": "123 Main St",
            "city": "Springfield"
        }
    });
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_complex_type_union_example() {
    let jason = r#"
        Container(value) {
            value: value
        }
        
        out [
            Container(32),
            Container("Alex"),
            Container(12312.341341235),
            Container({name: "dave", bob: 12})
        ]
    "#;
    
    let expected = json!([
        {"value": 32},
        {"value": "Alex"},
        {"value": 12312.341341235},
        {"value": {"name": "dave", "bob": 12}}
    ]);
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_typed_variable_assignment() {
    let jason = r#"
        Dev :: {name: String, lang: String}
        Value :: {value: Float}
        
        programmer = {
            name: "Alex",
            lang: "Java"
        } + {
            value: 3.0
        }
        
        out programmer
    "#;
    
    let expected = json!({
        "name": "Alex",
        "lang": "Java",
        "value": 3.0
    });
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_lua_integration() {
    let jason = r#"
        add_numbers(a, b) {
            result: lua_add(a, b)!
        }
        
        out add_numbers(5, 7)
    "#;
    
    let result = JasonBuilder::new()
        .include_lua(r#"function lua_add(a, b) return a + b end"#)
        .expect("Failed to include Lua")
        .jason_src_to_json(jason)
        .expect("Failed to parse");
    
    let expected = json!({"result": 12});
    
    assert_eq!(result, expected);
}

#[test]
fn test_lua_string_manipulation() {
    let jason = r#"
        out uppercase("hello")!
    "#;
    
    let result = JasonBuilder::new()
        .include_lua(r#"function uppercase(s) return string.upper(s) end"#)
        .expect("Failed to include Lua")
        .jason_src_to_json(jason)
        .expect("Failed to parse");
    
    let expected = json!("HELLO");
    
    assert_eq!(result, expected);
}

#[test]
fn test_empty_template() {
    let jason = r#"
        Empty() {
            empty: true
        }
        
        out Empty()
    "#;
    
    let expected = json!({"empty": true});
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_null_value() {
    let jason = r#"
        out {
            name: "Alex",
            middle: null,
            age: 30
        }
    "#;
    
    let expected = json!({
        "name": "Alex",
        "middle": null,
        "age": 30
    });
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_boolean_values() {
    let jason = r#"
        out {
            isActive: true,
            isDeleted: false
        }
    "#;
    
    let expected = json!({
        "isActive": true,
        "isDeleted": false
    });
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_nested_arrays() {
    let jason = r#"
        out [[1, 2], [3, 4], [5, 6]]
    "#;
    
    let expected = json!([[1, 2], [3, 4], [5, 6]]);
    
    assert_jason_eq(jason, expected);
}

#[test]
fn test_mixed_type_array() {
    let jason = r#"
        out [1, "two", 3.0, true, null]
    "#;
    
    let expected = json!([1, "two", 3.0, true, null]);
    
    assert_jason_eq(jason, expected);
}

#[cfg(test)]
mod file_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    
    fn create_test_file(name: &str, content: &str) -> PathBuf {
        let path = PathBuf::from(format!("test_files/{}", name));
        fs::create_dir_all("test_files").unwrap();
        fs::write(&path, content).unwrap();
        path
    }
    
    #[test]
    fn test_file_import_template() {
        // Create the imported file
        let dev_content = r#"
            Dev(name, project, money) {
                name: name,
                project: project,
                money: money
            }
        "#;
        create_test_file("Dev.jason", dev_content);
        
        // Create main file that imports
        let main_content = r#"
            import(Dev) from "./test_files/Dev.jason"
            
            out Dev("alex", "jason-rs", 0)
        "#;
        
        let result = JasonBuilder::new()
            .jason_src_to_json(main_content)
            .expect("Failed to parse with import");
        
        let expected = json!({
            "name": "alex",
            "project": "jason-rs",
            "money": 0
        });
        
        assert_eq!(result, expected);
        
        // Cleanup
        fs::remove_dir_all("test_files").ok();
    }
}
