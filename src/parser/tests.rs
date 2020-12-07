use crate::lexer::*;
use crate::parser::*;

#[test]
fn test_parse_empty_function() {
    let raw = "main :: () {}";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_function_with_return() {
    let raw = "
    main :: () {
        return 1
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_redundant_semicolon() {
    let raw = "
    main :: () {
        return 1;
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_err())
}

#[test]
fn test_parse_no_function_context() {
    let raw = "
    let x = 1
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_err())
}

#[test]
fn test_parse_multiple_functions() {
    let raw = "
    foo :: () {
        let x = 2
        return x
    }

    bar :: () {
        let y = 5
        return y
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_variable_declaration() {
    let raw = "
    main :: () {
        let x = 1
        return x
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_function_with_args() {
    let raw = "
    main :: (foo) {
        return foo
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_function_call() {
    let raw = "
    main :: (foo) {
        foo()
    }

    foo :: () {
        foo(2)
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_return_function_call() {
    let raw = "
    main :: () {
        return fib(2)
    }

    fib :: () {
        return fib(2)
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_function_call_multiple_arguments() {
    let raw = "
    main :: () {
        fib(1, 2, 3)
    }

    fib :: () {
        return 2
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_nexted_function_call() {
    let raw = "
    main :: () {
        fib(fib(2), 2)
    }

    fib :: (n) {
        return 2
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_basic_ops() {
    let raw = "
    main :: () {
        return 2 * 5
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_compound_ops() {
    let raw = "
    main :: () {
        2 * 5 / 3
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_compound_ops_with_function_call() {
    let raw = "
    main :: () {
        return 2 * fib(1) / 3
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_compound_ops_with_strings() {
    let raw = "
    main :: () {
        return 2 * \"Hello\"
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_compound_ops_with_identifier() {
    let raw = "
    main :: (n) {
        return 2 * n
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
#[ignore]
fn test_parse_compound_ops_with_identifier_first() {
    let raw = "
    main :: (n) {
        return n * 2
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_compound_ops_return() {
    let raw = "
    main :: (n) {
        return 2 * n
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_basic_conditional() {
    let raw = "
    main :: (n) {
        if n {
            return n
        }
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}

#[test]
fn test_parse_basic_conditional_with_multiple_statements() {
    let raw = "
    main :: (n) {
        if n {
            let x = 2 * n
            return x
        }
    }
    ";
    let tokens = tokenize(raw);
    let tree = parse(tokens, Some(raw.to_string()));
    assert!(tree.is_ok())
}
