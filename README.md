# jason-RS

[![Crates.io](https://img.shields.io/crates/v/jason-rs)](https://crates.io/crates/jason-rs)
[![Docs](https://docs.rs/jason-rs/badge.svg)](https://docs.rs/jason-rs)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

**jason** is a lightweight JSON templating tool that transforms reusable `.jason` template files into standard JSON. Build modular and composable JSON structures with parameter support and file inclusion.

## ✨ Features

- **Template Parameters** - Define reusable templates with named parameters
- **File Inclusion** - Compose JSON from multiple `.jason` files
- **1:1 Conversion** - Clean mapping from `.jason` syntax to standard JSON
- **Library-First** - Designed for seamless integration into Rust projects

## 🚀 Quick Start

Add `jason-rs` to your `Cargo.toml`:

```toml
[dependencies]
jason-rs = "0.1"
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

### Jason Templates

**Person.jason** - A reusable person template
```jason
(name, age) {
    "name": #name,
    "age": #age
}
```

**Studio.jason** - A static studio object
```jason
{
    "name": "GameInc",
    "ceo": "Dave Something"
}
```

**Team.jason** - Composing multiple templates
```jason
(project) {
    "studio": <./Studio.jason>,
    "project": #project,
    "workers": [
        <./Person.jason | "jay", 12>,
        <./Person.jason | "mark", 14>,
        <./Person.jason | "lee", 15>
    ]
}
```

**Page.jason** - The head of the composition.
```jason
{
    "team": <./Team.jason | "Mario">
}
```

### Generated JSON

```json
{
  "team": {
    "project": "Mario",
    "studio": {
      "name": "GameInc",
      "ceo": "Dave Something"
    },
    "workers": [
      {
        "name": "jay",
        "age": 12
      },
      {
        "name": "mark",
        "age": 14
      },
      {
        "name": "lee",
        "age": 15
      }
    ]
  }
}
```

## Syntax Overview

| Syntax | Description |
|--------|-------------|
| `(param1, param2)` | Define template parameters |
| `#param` | Reference a parameter value |
| `<./File.jason>` | Include another Jason file |
| `<./File.jason \| arg1, arg2>` | Include with arguments |



Parses a `.jason` file at the given path and returns the resulting JSON as a `String`.

##  License

Licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) for details.


