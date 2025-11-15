# jason-RS

[![Crates.io](https://img.shields.io/crates/v/jason-rs)](https://crates.io/crates/jason-rs)
[![Docs](https://docs.rs/jason-rs/badge.svg)](https://docs.rs/jason-rs)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

**jason** is a lightweight JSON templating tool that transforms reusable `.jason` files into standard JSON. Build modular and composable JSON structures with parameter support and file inclusion.

## ✨ Features

- **Template Parameters** - Define reusable templates with named parameters
- **File Inclusion** - Compose JSON from multiple `.jason` files
- **1:1 Conversion** - Clean mapping from `.jason` syntax to standard JSON
- **Library-First** - Designed for seamless integration into Rust projects

## 🚀 Quick Start

Add `jason-rs` to your `Cargo.toml`:

```toml
[dependencies]
jason-rs = "0.2.5"
```

Parse a Jason file:

```rust
use jason_rs::jason_to_json;

fn main() {
    //outputs raw json source
    let json = jason_to_json("./Page.jason").unwrap();
    println!("{}", json);
}
```

##  Example

### Jason File
```jason
//a variable that holds a value
project_name = "jason-rs"

//what gets exported at the top level
out {
    name: "Alex",
    project: "jason-rs",
    money: 0,
}
```

### Jason Templates
```jsaon
Dev(name, project, money) {
    name: name,
    project: project,
    money: money,
}
// invokes the template and fills in the variables with the passed in arguments
out Dev("alex", "jason-rs", 0) 
```

## importing

**Dev.jason** - A file containg the dev template
```jason
Dev(name, project, money) {
    name: name,
    project: project,
    money: money,
}
```

**main.jason** - The top level file being compiled
```jason
import(Dev) from "./Person.jason"

out Dev("alex", "jason-rs", 0) 
```

## * operator
The * operator repeats an expression an integer number of times and then stores it in a list.

```jason
//this returns ["hello","hello","hello","hello","hello","hello","hello","hello","hello","hello","hello","hello"]
out  "hello" * 12
```

**pick_example.jason** 
```jason
Person(name, age) {
    name: name,
    age: age
}
//makes a Person witha random name and int from 0 to 67. 2000 times and stores them into a list
main = Person(random_name()!, random_int(67)!) * 2000
//pick one value from main 12 times
result = pick(main)!*12
//out the result
out result
```

note: this will not import the context around DEV so variables will be ignored unless imported as well. 
this warning will be patched in a later version with groups.

## JasonBuilder

`JasonBuilder` allows you to add Lua dependencies to your `.jason` parsing pipeline.

Start with no Lua dependencies:

```rust
use jason_rs::JasonBuilder;

let builder = JasonBuilder::new();
```

## Adding Lua Dependencies

Include Lua files:

```rust
let builder = JasonBuilder::new()
    .include_lua_file("scripts/helpers.lua")?
    .include_lua_file("scripts/math.lua")?;
```

Or raw Lua source:

```rust
let lua_code = r#"function add(a,b) return a+b end"#;
let builder = JasonBuilder::new().include_lua(lua_code)?;
```
Both methods are chainable, allowing you to add multiple Lua dependencies easily.

Then you can just run the standard functions from converting `jason` to `json` using builder

```rust
fn main() -> Result<(), Box<dyn std::error::Error>>{
    let result = jason_rs::JasonBuilder::new()
        .include_lua(r#"
            -- Returns the part of `text` before the first occurrence of `delimiter`
            function split_first(text, delimiter)
                local delim_start, _ = string.find(text, delimiter, 1, true)
                if delim_start then
                    return string.sub(text, 1, delim_start - 1)
                else
                    return text  -- no delimiter found, return the whole string
                end
            end
        "#)?.jason_src_to_json(r#"            
            User(email, password, ip) {
                email: email,
                password: password,
                username: split_first(email, "@")!,
                ip: ip
            }
            out User(random_email()!, random_password()!, random_ipv4()!) * 2 
        "#)?;         
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

result
```json
[
  {
    "email": "ptcbkvhhda@www.example.com",
    "ip": "103.121.162.79",
    "password": "qMdC&PK0y8=s",
    "username": "ptcbkvhhda"
  },
  {
    "email": "aabzlr@api.demo.org",
    "ip": "69.44.42.254",
    "password": "DLPng64XhkQF",
    "username": "aabzlr"
  }
]
```

## Syntax Overview

| Syntax | Description |
|--------|-------------|
| `name(arg1, arg2, ...) {...}` | Defines a template name |
| `name() {...}` | Defines a template name  |
| `name = ...` | Defines a variable name  |
| `name(...)` | invokes a template |
| `import(template, variable, ...) from "path/to/file.jason"` | imports templates and or variables from file |
| `import(*) from "path/to/file.jason"` | imports all templates and all variables from a file |
| `import($) from "path/to/file.jason"` | imports all variables from a file |
| `func(...)!` | calls a built in function with passed in arguments|
| `expression * n  OR   n * expression` | repeats expression a positive integer n times and stores it as a list |
| `out <jason expression>` | when the file gets read from at the top level the value is what gets returned|



Parses a `.jason` file at the given path and returns a serde_json value object which can then be converted to structs

##  License

Licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) for details.


