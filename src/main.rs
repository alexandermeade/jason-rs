use jason_rs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let _json = jason_rs::JasonBuilder::new()
        .include_lua(r#"
            function inc(n) 
                return n + 1
            end
        "#)?
        .jason_to_json("./testing.jason")?;
        //.jason_to_json("./test.jason")?;
    let duration = start.elapsed();
    println!("some_function took: {:?}", duration);
    println!("{}", _json);
    Ok(())
}
