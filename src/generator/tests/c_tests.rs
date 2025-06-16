use crate::ast::types::Type;
use crate::ast::BinOp::*;
use crate::ast::Expression::*;
use crate::ast::Statement::*;
use crate::ast::{Function, StructDef, Variable};
use crate::generator::c::*;
use std::collections::HashMap;

#[test]
fn test_generate_block_empty() {
    let t = generate_block(
        Block {
            statements: vec![],
            scope: vec![],
        },
        None,
    );
    assert_eq!(t, "{\n}\n")
}

#[test]
fn test_generate_block_with_statement() {
    let t = generate_block(
        Block {
            statements: vec![Return(Some(Int(42)))],
            scope: vec![],
        },
        None,
    );
    assert_eq!(t, "{\n    return 42;\n}\n")
}

#[test]
fn test_generate_function() {
    let func = Function {
        name: "test_func".to_string(),
        arguments: vec![],
        ret_type: Some(Type::Int),
        body: Block {
            statements: vec![Return(Some(Int(0)))],
            scope: vec![],
        },
    };
    let result = generate_function(func);
    assert_eq!(result, "int test_func(void) {\n    return 0;\n}\n\n")
}

#[test]
fn test_generate_arguments_empty() {
    let args = vec![];
    assert_eq!(generate_arguments(args), "void")
}

#[test]
fn test_generate_arguments_with_params() {
    let args = vec![
        Variable {
            name: "x".to_string(),
            ty: Some(Type::Int),
        },
        Variable {
            name: "y".to_string(),
            ty: Some(Type::Bool),
        },
    ];
    assert_eq!(generate_arguments(args), "int x, bool y")
}

#[test]
fn test_generate_expression_int() {
    assert_eq!(generate_expression(Int(42)), "42")
}

#[test]
fn test_generate_expression_string() {
    assert_eq!(generate_expression(Str("hello".to_string())), "\"hello\"")
}

#[test]
fn test_generate_expression_bool() {
    assert_eq!(generate_expression(Bool(true)), "true")
}

#[test]
fn test_generate_binary_operation() {
    let expr = BinOp {
        lhs: Box::new(Int(1)),
        op: Addition,
        rhs: Box::new(Int(2)),
    };
    assert_eq!(generate_expression(expr), "1 + 2")
}

#[test]
fn test_generate_function_call() {
    let call = FunctionCall {
        fn_name: "test".to_string(),
        args: vec![Int(1), Int(2)],
    };
    assert_eq!(generate_expression(call), "test(1, 2)")
}

#[test]
fn test_generate_array() {
    let arr = Array {
        capacity: 3,
        elements: vec![Int(1), Int(2), Int(3)],
    };
    assert_eq!(generate_expression(arr), "{1, 2, 3}")
}

#[test]
fn test_generate_array_access() {
    let access = ArrayAccess {
        name: "arr".to_string(),
        index: Box::new(Int(0)),
    };
    assert_eq!(generate_expression(access), "arr[0]")
}

#[test]
fn test_generate_struct_definition() {
    let struct_def = StructDef {
        name: "TestStruct".to_string(),
        fields: vec![Variable {
            name: "field1".to_string(),
            ty: Some(Type::Int),
        }],
        methods: vec![],
    };
    let result = generate_struct_definition(struct_def);
    assert_eq!(
        result,
        "typedef struct TestStruct {\n    int field1;\n} TestStruct;\n\n"
    )
}

#[test]
fn test_generate_conditional() {
    let if_stmt = If {
        condition: Bool(true),
        body: Box::new(Block {
            statements: vec![Return(Some(Int(1)))],
            scope: vec![],
        }),
        else_branch: None,
    };
    let result = generate_statement(if_stmt);
    assert_eq!(result, "    if (true) {\n    return 1;\n}\n;\n")
}

#[test]
fn test_generate_while_loop() {
    let while_stmt = While {
        condition: Bool(true),
        body: Box::new(Block {
            statements: vec![Break],
            scope: vec![],
        }),
    };
    let result = generate_statement(while_stmt);
    assert_eq!(result, "    while (true) {\n    break;\n}\n;\n")
}

#[test]
fn test_generate_struct_initialization() {
    let mut fields = HashMap::new();
    fields.insert("x".to_string(), Box::new(Int(1)));
    let init = StructInitialization {
        name: "Point".to_string(),
        fields,
    };
    assert_eq!(generate_expression(init), "(Point) {.x = 1}")
}

#[test]
fn test_generate_field_access() {
    let access = FieldAccess {
        expr: Box::new(Variable("point".to_string())),
        field: Box::new(Variable("x".to_string())),
    };
    assert_eq!(generate_expression(access), "point.x")
}

#[test]
fn test_generate_for_loop() {
    let for_stmt = For {
        ident: Variable {
            name: "i".to_string(),
            ty: Some(Type::Int),
        },
        expr: Array {
            capacity: 2,
            elements: vec![Int(1), Int(2)],
        },
        body: Box::new(Block {
            statements: vec![],
            scope: vec![],
        }),
    };
    let result = generate_statement(for_stmt);
    assert!(result.contains("for(int i = 0;"));
}

#[test]
fn test_generate_declare() {
    let var = Variable {
        name: "x".to_string(),
        ty: Some(Type::Int),
    };
    let decl = generate_declare(var, Some(Int(42)));
    assert_eq!(decl, "int x = 42")
}

#[test]
fn test_generate_return() {
    assert_eq!(generate_return(Some(Int(42))), "return 42");
    assert_eq!(generate_return(None), "return");
}
