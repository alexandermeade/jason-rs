use jason_rs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = jason_rs::JasonBuilder::new()
        .include_lua(r#"
            function inc(n) 
                return n + 1
            end
        "#)?
        .jason_to_json("./test.jason")?;
        //.jason_to_json("./test.jason")?;
    println!("{}", json);
    Ok(())
}
