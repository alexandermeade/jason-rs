<p align="center">
<img width="195" height="75" alt="screenshot(2)" src="https://github.com/user-attachments/assets/7a7ebbee-49f2-40d8-800e-6ff7e8c262e6" />
</p>


[![Crates.io](https://img.shields.io/crates/v/jason-rs)](https://crates.io/crates/jason-rs)
[![Docs](https://docs.rs/jason-rs/badge.svg)](https://docs.rs/jason-rs)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

**jason-rs** build, structure, and reuse data exactly as you expect. 

## ✨ Features

- **Template Parameters** - Define reusable templates with named parameters
- **File Inclusion** - Compose JSON from multiple `.jason` files
- **1:1 Conversion** - Clean mapping from `.jason` syntax to standard JSON
- **Library-First** - Designed for seamless integration into Rust projects

## 🚀 Quick Start

Add `jason-rs` to your `Cargo.toml`:

```toml
[dependencies]
jason-rs = "0.3.2"
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
### Jason File

`Jason` is a langauge that compiles directly into `JSON`. In the example below we create a variable and then output some values via the `out` keyword

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

`Templates` allow you to create Objects in a way that doesn't feel redundant.

```jason
Dev(name, project, money) {
    name: name,
    project: project,
    money: money,
}
// invokes the template and fills in the variables with the passed in arguments
out Dev("alex", "jason-rs", 0) 
```

### Types 

`Types` in `Jason` are very easy to use and are completely optional to use. 

```jason
//creates a type for Projects
//| is the union symbol meaning it can be a String or Null
//you can union multiple types together
Project :: String | Null

//creates a type for Developer (similar to a schema)
Developer :: {
    name: String,
    project: String,
    money: Float 
}

//creates a typing for Dev that expects it to output 
//the Developer schema
Dev(String, Project, Number) :: Developer 
Dev(name, project, money) {
    name: name,
    project: project,
    money: money,
}

//creates a variable with type Developer and assignes it
alex:Developer = Dev("alex", "jason-rs", 0.0)

//alex2's type is infered from alex and the value is stored
alex2 := alex

out alex2
```

Errors with types are pretty clear.
For example if Dev had incorrect inputs it will emit

```jason
Error: Type Error in file ./trans.jason on line 23: Template Dev resulted in {name: String, paul: Number, project: Null} expected {money: Number, name: String, project: String}
  Missing fields:
    - money: Number

  Extra fields:
    + paul: Number

  Type mismatches:
    ~ project: expected String, found Null

   23 | alex : Developer = Dev("alex", null, 0)
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
import(Dev) from "./Dev.jason"

out Dev("alex", "jason-rs", 0) 
```

note: this will not import the context around DEV so variables will be ignored unless imported as well. 
note note: To import types from another file use `$` in your import. This will also import all variables over from the other file. (This will become better I swear)


## The `+` Operator

The `+` operator is **overloaded** and behaves differently depending on the types given to it. It works with **objects**, **arrays**, and **strings**.

### Object Concatenation

When both operands are objects (`{...} + {...}`), the operator **merges them into a single object**. If a key exists in both objects, the **value from the right-hand object overrides** the value from the left-hand object.  

**Examples:**

```jason
{name: "Alex"} + {age: 20}       // yields {name: "Alex", age: 20}
{a: 1, b: 2} + {b: 3, c: 4}     // yields {a: 1, b: 3, c: 4}
```
Array Concatenation

### Object Concatenation

When both operands are arrays ([...] + [...]), the operator joins the two arrays into a single array, with elements from the left-hand array appearing first, followed by elements from the right-hand array.

Examples:
```
[1, 2, 3] + [4, 5]             // yields [1, 2, 3, 4, 5]
["a"] + ["b", "c"]             // yields ["a", "b", "c"]
```
### Object Concatenation

When both operands are strings ("..." + "..."), the operator joins them into a single string. Jason strings support Unicode, so you can concatenate emoji, symbols, or other characters.

Examples:
```
"hello" + " world"             // yields "hello world"
"😀" + "🚀"                      // yields "😀🚀"
```

Side note: Jason supports composite strings, which interpolate variables or expressions inside a string, e.g., "Hello, {name}!". Composite strings can be concatenated with other strings in the same way as regular strings:

name = "Alex"
$"Hello, {name}!" // "Hello, Alex!"

you can also write full jason valid expressions in Composite strings

$"Hello, {{name: "Alex", age:20}}!" //"Hello, {\"age\":20,\"name\":\"Alex\"}!"

Note:
You can not concat Composite strings with Composite String or Strings with Composite Strings for consistentcy purposes

## `*`, `repeat`, `pick`, `upick` operators
The `*` operator repeats an expression an integer number of times and then stores it in a list.

```jason
//this returns ["hello👋","hello👋","hello👋","hello👋","hello👋","hello👋","hello👋","hello👋","hello👋","hello👋","hello👋","hello👋"]
out  "hello👋" * 12
```

Note: 
The `repeat` operator functions similar to the `*` operator except it doesn't reevaluate the expression each time so it's more efficent for copying static elements

**pick_example.jason** 
```jason
Person(name, age) {
    name: name,
    age: age
}

//makes a Person witha random name and int from 0 to 67. 2000 times and stores them into a list
main = Person(random_name()!, random_int(67)!) * 2000

//pick one value from main 12 times (with repeats)
//you should use upick for unique elements
result = main pick 12

//out the result
out result
```

- `pick`/`upick` operators return single values instead of `[T]` if given only one item to pick.

-  `Repeat` and `*` when `repeating`/`*` a value by `1` will always return a `[T]`


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

## Errors
Error outputs are nice and concise and propogate nicly. 


Example error
```
Error: Lua Function Error in file ./main.jason on line 8: failed to find function random_name: error converting Lua nil to function
    8 |  out Some(Person(random_name()!, random_int(30)!)) * 50
                         ^^^^^^^^^^^^^^                        
```
## Syntax Overview

| Syntax | Description |
|--------|-------------|
| `name(arg1, arg2, ...) {...}` | Defines a template  |
| `name() {...}` | Defines a template  |
| `name = ...` | Defines a variable name and sets the value to the result of the right hand expression |
| `name := ...` | functions like the `=` operator except it infers a type onto `name`|
| `name ::= T` | functions like the `:=` operator except it assignes a direct type onto `name` without giving it a value this symbol has been dubbed the spider walrus|
| `name: T = ...` | Defines a variable name and sets the value to the result of the right hand expression and the Type to the right hand side of the `:` operator|
| `name(...)` | invokes a template |
| `out <jason expression>` | when the file gets read from at the top level the value is what gets returned|
| `String, Int, Float, Number, Bool, Any, Null, [T], {key1: T, key2: U}` | Note: `T` and `U` are types.  `[T]` defines a List of type `T`. `{key1, T, key2: U}` defines an object with two keys where the keys have their own respective types|
| `Person :: T` | Creates a type alias `Person` of type `T`| 
| `Person(T, ...) :: U` | Provides a type to a template where the arguments are positional types for the Template and the right hand side is the type of the result| 
| `import(template, variable, ...) from "path/to/file.jason"` | imports templates and or variables from file |
| `import(*) from "path/to/file.jason"` | imports all templates, variables and all types from a file |
| `import($) from "path/to/file.jason"` | imports all variables and types from a file |
| `func(...)!` | calls a built in function with passed in arguments|
| `expression * n  OR   n * expression` | repeatedly evaluates expression a positive integer n times and stores it as a list |
| `expression repeat n` | repeats expression a positive integer n times and stores it as a list but does not revaluate expression! Note: it's faster than * if revaluation is not needed |
| `str(expression)` |converts expression result into a `string`|
| `int(expression)` |converts expression result into a `Number` but cuts off the floating point values|
| `float(expression)` |converts expression result into a `Number` but preserves floating point values|
| `int(expression1, expression2)` |chooses a random int between expression1 and or equal to expression 2|
| `float(expression1, expression2)` |chooses a random float between expression1 and or equal to expression 2|
| `{...} + {...}` | Object concat yeilds {name: "Alex"} + {age: 20} = {name: "Alex", age: 20}. Note it overrides keys with right dominance|
| `[...] + [...]` | list concat expressions. yeilds [1,2,3] + [4,5] = [1,2,3,4,5]|
| `"..." + "..."` | String concat yeilds "hello" + " world" = "hello world". strings support unicode!|
| `"alex" at 0` | gets character at index 0 in this case 'a'!|
| `["alex", "jason"] at 1` | gets element at index 1 in this case "jason"!|
| `{name: "alex", age: 20} at "age"` | gets the value with key "age" in this case 20!|
| `[...] pick 2` | picks two values randomly from array!|
| `[...] upick 2` | picks two unique values randomly from array!|
| `[...] map(n) expression` |maps each element from the array on the left (noted as n in this case) with the expression on the right of map|
| `[...] map(n, i) expression` |evaluates similiar to a normal map but the second argument represents position in array|

##  License

Licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) for details.


