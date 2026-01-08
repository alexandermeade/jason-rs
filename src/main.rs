
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _json = jason_rs::JasonBuilder::new()
        .include_lua(r#"
            function inc(n) 
                return n + 1
            end
        "#)?
        .jason_to_json("./testing.jason")?;
        //.jason_to_json("./test.jason")?;

    println!("{}", _json);
    Ok(())
}
