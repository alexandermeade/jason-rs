
# jason-RS

[![Crates.io](https://img.shields.io/crates/v/jason-rs)](https://crates.io/crates/jason-rs)
[![Docs](https://docs.rs/jason-rs/badge.svg)](https://docs.rs/jason-rs)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

Jason builds, structures, and reuses data exactly as you expect.

# ✨ Features

* Template Parameters - Define reusable templates with named parameters
* File Inclusion - Compose JSON from multiple `.jason` files
* 1:1 Conversion - Clean mapping from .jason syntax to standard JSON
* Library-First - Designed for seamless integration into Rust projects

## 🚀 Quick Start

Add jason-rs to your Cargo.toml:

[dependencies]
jason-rs = "0.2.5"

Parse a Jason file:
```rust, ignore
use jason_rs::jason_to_json;

fn main() {
    //outputs raw json source
    let json = jason_to_json("./Page.jason").unwrap();
    println!("{}", json);
}
```

# Jason Overview
```jason, ignore
//a variable that holds a value
project_name = "jason-rs"

//what gets exported at the top level
out {
    name: "Alex",
    project: "jason-rs",
    money: 0,
}
```

## Jason Templates

```jason, ignore
Dev(name, project, money) {
    name: name,
    project: project,
    money: money,
}
// invokes the template and fills in the variables with the passed-in arguments
out Dev("alex", "jason-rs", 0) 
```

You can also have implicit fields by doing.

```jason,ignore
Dev(*name, *project, *money) {}
```

What this does is create a field with the key name of the given identifier so they compile to the same result. You can also mix it up as well so.

```jason,ignore
Dev(*name, project, *money) {
    project: $"{project}-{name}"
}

out Dev("alex", "skywart", 32000.52) 
```

is valid and compiles to 

```json, ignore
{
    "money": 32000.52,
    "name":"alex",
    "project":"skywart-alex"
}
```
## importing

Dev.jason - A file containing the dev `template`

```jason, ignore
Dev(name, project, money) {
    name: name,
    project: project,
    money: money,
}
```
main.jason - The top-level file being compiled
```jason, ignore
import(Dev) from "./Dev.jason"

out Dev("alex", "jason-rs", 0) 
```

Note: this will not `import` the context around DEV, so variables will be ignored unless imported as well.

## Including 

The `include` operator lets you include the result of one `jason` file into your current one as an inline value.
```jason, ignore
*showcase.jason*

out {name: "Alex", age: 20}
```
*main.jason*

value = include "./showcase.jason" // value is  {name: "Alex", age: 20}


# Composite Strings in Jason

`Jason` supports composite `strings` via
```jason, ignore
name = "Alex"
age = 20
out $"name {name}, my age is {age} and my account looks like {{name: name, age: age}}"
```

which yields

```jason, ignore
"name Alex, my age is 20 and my account looks like {\"age\":20,\"name\":\"Alex\"}"
```

Basic Operations in Jason

`Jason` supports math in its jason expressions so
```jason, ignore
    value = 3.14 * 200 + 2 - 1 /4 + 4%4 // value yeilds 629.75
```

It is completely valid `Jason`. However it is importnant to note that `*` and `+` are type sysnsitive operations and their behavior changes depending on what you’re using them with.

# Type Conversions and Random Numbers

Jason offers a host of built-in functions to do things like convert types from one to another and to generate values that you may need.

## Int

The `int()` function takes in one value and gives back an `Int`

```jason, ignore
value1 = int("231") // 231
value2 = int(300.24123) // 300
value3 = int(300) // 300
```

However, the `int()` function can also generate a random `Int` by supplying a second argument so

```jason, ignore
random_int = int(0, 300)
```

generates a random Int between 0 and 300

## Float

The `float()` function takes in one value and gives back a `float`

```jason, ignore
value1 = float("231.231") // 231.231
value2 = float(300.24123) // 300.24123
value3 = float(300) // 300.0
```

However, the `float()` function can also generate a random `Float` by supplying a second argument, so

```jason, ignore
random_int = float(0, 300)
```

generates a random `Float` between 0 and 300

## String

The `str()` function takes in one value and gives back a `String`

```jason, ignore
value1 = str("231.231") // "231.231"
value2 = str(300.24123) // "300.24123"
value3 = str(300) // "300"
```

# the + operation

The `+` operation works as both a concatenation operation with `strings`, `lists`, and `objects`, but as an arithmetic `plus` operation against `Numbers`, for example.

    value1 = 3 + 3 // 6
    value2 = [1, 2] + [3, 4] // [1, 2, 3, 4]
    value3 = {name: "Alex"} + {age: 20} // {name: "Alex", age: 20}
    value4 = "😀" + "🚀"  //"😀" + "🚀"

# The * operation

The `*` operation works as a copy operator as well as a multiplicitive one so.

    value1 = 3 * 3 // 9
    value2 = "alex" * 3 // ["alex", "alex", "alex"]
    value3 = int(0, 300) * 3 // note Int(min, max) gets a random int. [33, 33, 33] 

# Unique Jason Operators

Jason provides a selection of operators to help with transforming and manipulating data in a way that doesn’t feel too pragmatic.

## The at Operator

The `at` operator lets you index a `List` or an `Object` at a specific `index` and or `key`.

```jason, ignore
value1 = [1,2,3,4,5] at 0 // 1
value2 = {name: "alex", age: 20} at "name" // "alex"
```

The `at` operator will let you know if you `index` out of bounds or index a `key` that doesn’t exist.

```jason, ignore
Error: Indexing Error in file ./testing.jason on line 5: invalid convert number 231 at list with len 5
    5 | value1 = [1, 2, 3, 4, 5] at 231

Indexing Error in file ./testing.jason on line 6: key doesn't exit e
    6 | value2 = { name : "alex", age : 20 } at "e"
```

## The `repeat` Operator

The first unique and useful `Jason` operator is the `repeat` operator which is similar to the `*` operator where it copies a value for an `Int` amount of times. However, it also `reevaluates` said values.

```jason, ignore
values = int(0, 300) repeat 3 //yeilds [99,27,127] 
```

## The pick and upick operators

The `pick` operator lets you pick a `Int` of randomly selected elements from a list. You can pick more elements than there are, since it builds a new list from those randomly selected elements.

We’ll introduce a list `nums` for our examples
```jason, ignore
nums = [1,2,3]
picked_nums = nums pick 5 // yields [2,3,2,3,1]
```

However, note that `pick` operations are random, so my examples may differ from what you run.

The `upick` operator lets you pick a number of unique elements from a `List`, and what this means by unique is not quite in value but in index, so if you have a linear list of different values, you can upick `4` values from it and get 4 unique values. However, you can not upick more than the number of elements in the list since they must be of unique index.

```jason, ignore
nums = [1,2,3]
picked_nums = nums upick 3 //[1,2,3]
```

# the map operator

If you’ve ever used a language with `map` operations before, this operator is basically the same except it works as a binary operator between a list of values and some expression.
```jason, ignore
[1,2,3] map(n) n * 2 //yields [2,4,6]
```

So it iterates over each element in the list and `maps` it to the correct value in the expression.

The `map` operator also has an overload to allow you to get the index along with the value by simply.
```jason, ignore
[1,2,3] map(n, i) {value: n, index: i} 
//yields [{"index":0,"value":1},{"index":1,"value":2},{"index":2,"value":3}]
```

Debuging in jason via info and `infoT`

The `info` operator lets prints out the `type` and `value` of an `expression` or `variable` along with any other relevant info.

```jason, ignore
info [30, 20.124, "hello"]
```

The `info` it produces: (This does not appear in the compiled JSON)

```jason, ignore
┌[Info] Token at line 4, col 5. in file: ./testing.jason
│ Code: 
│
│     4 |  info [30, 20.124, "hello"]
│
│ Value: [30,20.124,"hello"]
│ Type: [Float | String | Int]
└─────────────
```

You can also do a similar thing to test `jason` `types`.

```jason, ignore
infoT >= 0 while < 100 | Null | {name: String, age: >= 0}'
```

The `infoT` produces (This also does not appear in the compiled `JSON`)

```jason, ignore
┌[Type-Info] Token at line 4, col 6. in file: ./testing.jason
│ Code: 
│
│     4 |  infoT  >= 0 while  < 100 | Null | { name : String, age :  >= 0 } ' 
│
│ Type: [0,100) | Null | {age: [0,Infinity), name: String}'
└─────────────
```

# Basic Types

Typing in `Jason` is completely `optional`; however, if you do plan to use them, `jason` has an algebra with a few pretty simple operators to allow easy composable `types`.

Jason includes a small but useful collection of base `types` to choose from that accurately reflect all types of json that may appear in day-to-day problems.

* Generic and Null types
  * `Any` - Allows any values of all `types`.
  * `Null` - Allows the value `null`.
* Numeric Types
  * `Number` - Includes all numeric values.
  * `Int` - Includes all non-floating point values.
  * `Float` - Includes all numbers but expects it in the format of a `float` I.E., 2.0, not 2.
* String Type
  * `String` - permits `String` values I.E. "paul", "dave"
* Bool Type
  * `Bool` - permits Boolean values I.E. `true`, `false`.

examples:
```jason, ignore
value1: Int = 3 // ✅
value2: Float = 3 // ❌
value3: Float = 3.0 // ✅
value4: Bool = true // ✅
value5: String = "Hello World :)" // ✅
value6: Any = 213.231 // ✅
value7: Null = null // ✅
```
a sample error message:

```jason, ignore
Error: Type Error in file ./testing.jason on line 2: type mismatches
 expected Float, found Int

    2 | value2 : Float = 3
        ^^^^^^            
```

# Type Constraints

While it feels that `jason`’s base `types` are pretty constraining (get it), you can extend the complexity of `types` via type constraints.

## Union Operator |

The union operator allows you to check against multiple types for one value.

```jason, ignore
value1: Int | Null = 3 // ✅
value1 = null // ✅
value1 = 3.0 // ❌
```

## Value Constraints

You can also ensure that the value is the specific value you have in mind, and you can do this for `String`, `Float`, `Bool` and `Int` types.
```jason, ignore
value1: 302 | 3.14 | "paul" = 302 // ✅
value1 = 3.14 // ✅
value1 = "paul" // ✅
value1 = 2 // ❌
```

You can also constrain values to a range, like so

```jason, ignore
pos_num: >= 0 = 3 // ✅
pos_num = 0 // ✅
pos_num = -1 // ❌
```

You can also do ranges with `>, <, >=, and <=`, and if you want to combine intervals, you simply use the `while` operator, so
```jason, ignore
    value: > 0 while < 20 = 4
```

yields the interval of `(0, 20)`.

Note Constraints via ranges get reinterpreted as intervals by the compiler and get presented as such via errors. For example:
```jason, ignore
Error: Type Error in file ./testing.jason on line 4: type mismatches
 expected [0,Infinity), found Int

    4 | pos_num = -1
        ^^^^^^^     
```

And with this, this stops impossible intervals from occurring, like in the case of
```jason, ignore
    num: > 0 while < 0 = 2
```

with an error like
```jason, ignore
Error: Value Error in file ./testing.jason on line 1: Empty interval: bounds meet at 0 but not both inclusive
    1 | num :  > 0 while  < 0 = 2
```

## List Types

Types for `Lists` are pretty simple, where you can express them as such.
```jason, ignore
    values: [T] = [...]
    nums: [Number] = [20, 14.231, 40] // ✅
```

You can also express any possible `type` as a `List`, so
```jason, ignore
    ages: [>= 0 | "unknown"] = [20, 30, 40, "unknown"] // ✅
    ages = [20, 30, 40, "unknown", true] // ❌
```

with an example error
```jason, ignore
Error: Type Error in file ./testing.jason on line 1: type mismatches
 expected [[0,Infinity) | "unknown"], found [Bool | Int | String]

    1 | ages : [ >= 0 | "unknown"] = [20, 30, 40, "unknown", true]
        ^^^^                                                      
```

# Type syntax

As a note, you can separate type definitions from value assignment since from this point on types are gonna get bigger, so a few operators to mention are

## The `::` Operator

Allows you to define a type so
```jason, ignore
    Age :: 0 >= while < 130
```

creates a type Age where you can use anywhere else, so the general form is.
```jason, ignore
    NewType :: T
```

# Tempalte Types
To define a `type` for a template you need to use the `::` operator and with this you supply the name of the template `Tempalte(Type1, Type2) :: ResultType` in this format where each `type` in the parathenses relates to a parameter.

exmaple:


```jason, ignore
Person :: {
    name: String,
    age: >= 0
}

Citizen(String, >= 0) :: Person
Citizen(name, age) {
    name: name,
    age: age
}

out Citizen("alex", 20)
```

this outputs

```jason, ignore
{"age":20,"name":"alex"}
```

however if we made the `age` `negative` we get
```jason, ignore
Error: Type Error in file ./testing.jason on line 14: expected type [0,Infinity) for age found Int in template Citizen
   14 |  out Citizen("alex", -20)
```

because it doesn't match our parameter type for age and if our age didn't require `>= 0` then 

```jason, ignore
Error: Type Error in file ./testing.jason on line 14: Template Citizen resulted in {age: Int, name: String} expected {age: [0,Infinity), name: String}
  Type mismatches:
    ~ age: expected [0,Infinity), found Int

   14 |  out Citizen("alex", -20)
```

Our `result` `type` would catch it.


# Object Types

`Object` `Types` in `Jason` not only validate that the values are gonna be what you expect, but also that the structure is preserved after compile time, so you don’t have to worry about unexpected labels added to add typing and structure to your data.

`Object` `Types` function pretty similarly to how the List types look, so you slot in the type where you’d expect the value to be at each `key`, so the general representation would be
```jason,ignore
    NewType :: {
        key1: Type1,
        key2: Type2,    
    }
```
You can also nest object types so
```jason,ignore
    NewType :: {
        key1: Type1,
        key2: {
            InnerKey1: InnerType1,
            InnerKey2: InnerType2,
            ...
        },    
        ...
    }
```
It is also completely valid.

A more concrete example would be
```jason,ignore
    Person :: {
        name: String,
        age: >= 0
    }
    person: Person = {name: "Alex", age: 20}
```
and we can also see that if we try to give it a `value` that isn’t in our `type` definition of `Person`, like in the case of
```jason,ignore
    person: Person = {name: "Alex", age: -1}
```
We get a detailed breakdown of what’s wrong with our object types here.
```jason,ignore
Error: Type Error in file ./testing.jason on line 6: type mismatches
 expected {age: [0,Infinity), name: String}, found {age: Int, name: String}

  Type mismatches:
    ~ age: expected [0,Infinity), found Int

    6 | person : Person = { name : "Alex", age : -1 }
        ^^^^^^                                       
```

# Object Type Operators `+`, `&`, `'`, `with`

This is a pretty big jumble of operators for just `Object` types, but I feel they bring a pretty good edge to dealing with a variety of possible `Object` `Types` you may need in a way where you don’t have to manually write `type` data.

## The `&` Operator

The `&` operator allows you to combine fields of type objects, and if fields are shared across two `objects`, they `union` together.

Examples:
```jason,ignore
Person :: {age: >= 0, language: String, name: String, social_media: {}}
PhotoBookUser :: {age: "privated", email: String, social_media: {photobook_url: String}}
```
Person & PhotoBookUser resolves into
```jason,ignore
{
    age: [0,Infinity) | "privated", 
    email: String, 
    language: String, 
    name: String, 
    social_media: {
    photobook_url: String
    }
}
```
## The `+` operator 
The `+` operator functions similarly to the `&` operator, except it’s right-dominant (so values on the right take precedence over values on the left), and it doesn’t affect heavily nested fields, so if we refer to our other example, and we use `Person + PhotoBookUser`, it resolves into
```jason,ignore
{
    age: "privated", 
    email: String, 
    language: String, 
    name: String, 
    social_media: {
        photobook_url: String
    }
}
```

## The ' Operator

The `'` operator is probably the simplest operator in this series of operators, where the only thing it requires is that you have at least one field of the desired `Object` type.
```jason,ignore
    person: Person' = {age: 20} // ✅
    person = {name: "Alex"} // ✅
    person = {} // ❌
```

## The ‘with’ Operator

The `with` operator allows you to fill in an `Object`'s `type` so if we take our `Person` object, we can do something like
```jason,ignore
    NulledPerson :: Person with Null
```

and the type of `NulledPerson` will look like

```jason,ignore
{
    age: Null, 
    language: Null, 
    name: Null, 
    social_media: {}
}
```

And this may not seem as purposeful as the other operators, however, combining this with the original `Person` `Type` can net some pretty cool.

```jason,ignore
NullablePerson :: Person & (Person with Null) 

In the above example, NullablePerson will result in the type.

{
    age: [0,Infinity) | Null, 
    language: String | Null, 
    name: String | Null, 
    social_media: {}
}
```

and this makes your Person type full Nullable without having to write any extra type information!

# JasonBuilder

`JasonBuilder` allows you to add Lua dependencies to your `.jason` parsing pipeline.

Start with no Lua dependencies:
```rust, ignore
use jason_rs::JasonBuilder;

let builder = JasonBuilder::new();
```

Adding Lua Dependencies

Include Lua files:
```rust, ignore
let builder = JasonBuilder::new()
    .include_lua_file("scripts/helpers.lua")?
    .include_lua_file("scripts/math.lua")?;
```

Or raw Lua source:

```rust, ignore
let lua_code = r#"function add(a,b) return a+b end"#;
let builder = JasonBuilder::new().include_lua(lua_code)?;
```

Both methods are chainable, allowing you to add multiple Lua dependencies easily.

Then you can just run the standard functions for converting jason to json using builder.
```rust, ignore
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
            User(String, String, Int) :: {
                email: String,
                password: String,
                username: String,
                ip: Int
            }
            User(email, password, ip) {
                email: email,
                password: password,
                username: split_first(email, "@")!,
                ip: ip
            }
            out User("jasondev@gmail.com", "user.passxp", 23123)  
        "#)?;         
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

result:
```jason,ignore
{
  "email": "jasondev@gmail.com",
  "ip": 23123,
  "password": "user.passxp",
  "username": "jasondev"
}
```

# Errors

Error outputs are nice and concise and propagate nicely.

Example error
```jason,ignore
Error: Lua Function Error in file ./main.jason on line 8: failed to find function random_name: error converting Lua nil to function
    8 |  out Some(Person(random_name()!, random_int(30)!)) * 50
                         ^^^^^^^^^^^^^^                        
```
License

Licensed under the Apache License 2.0. See LICENSE for details.

