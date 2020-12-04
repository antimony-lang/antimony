use crate::lexer::*;
use crate::parser::*;

#[test]
fn test_parse_empty_function() {
    let raw = "fn main() {}";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_function_with_return() {
    let raw = "
    fn main() {
        return 1;
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_missing_semicolon() {
    let raw = "
    fn main() {
        return 1
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_err())
}

#[test]
fn test_parse_no_function_context() {
    let raw = "
    let x = 1;
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_err())
}

#[test]
fn test_parse_multiple_functions() {
    let raw = "
    fn foo() {
        let x = 2;
        return x;
    }

    fn bar() {
        let y = 5;
        return y;
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_variable_declaration() {
    let raw = "
    fn main() {
        let x = 1;
        return x;
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}
