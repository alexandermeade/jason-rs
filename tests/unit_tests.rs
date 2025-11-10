
use std::fs;
use crate::jason::*; // your library modules

#[test]
fn run_compiler_tests() {
    let input_dir = "tests/inputs";
    let expected_dir = "tests/expected_outputs";

    for entry in fs::read_dir(input_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().unwrap() != "jason" { continue; }

        let source = fs::read_to_string(&path).unwrap();
        let mut parser = Parser::new_from_str(&source);
        let ast = parser.parse().unwrap();

        let result = Compiler::compile_ast(&ast).unwrap(); // or your JSON serialization

        // Compare to expected output
        let expected_path = path.with_file_name(path.file_name().unwrap())
                                .with_file_name(format!("{}.expected", path.file_stem().unwrap().to_string_lossy()));
        let expected = fs::read_to_string(expected_path).unwrap();

        assert_eq!(result, expected, "Test failed for {:?}", path);
    }
}

