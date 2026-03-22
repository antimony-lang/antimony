/**
 * Tests for the JavaScript generator implementation
 */
#[cfg(test)]
mod tests {
    use crate::ast::types::Type as AstType;
    use crate::ast::*;
    use crate::generator::js::JsGenerator;
    use crate::generator::Generator;
    use std::collections::HashMap;

    fn create_function(name: &str, ret_type: Option<AstType>, body: Statement) -> Function {
        Function {
            name: name.to_string(),
            arguments: Vec::new(),
            ret_type,
            body,
        }
    }

    fn create_function_with_args(
        name: &str,
        arguments: Vec<Variable>,
        ret_type: Option<AstType>,
        body: Statement,
    ) -> Function {
        Function {
            name: name.to_string(),
            arguments,
            ret_type,
            body,
        }
    }

    fn create_variable(name: &str, typ: AstType) -> Variable {
        Variable {
            name: name.to_string(),
            ty: Some(typ),
        }
    }

    fn create_block(statements: Vec<Statement>) -> Statement {
        Statement::Block {
            statements,
            scope: Vec::new(),
        }
    }

    fn create_module(funcs: Vec<Function>, structs: Vec<StructDef>) -> Module {
        Module {
            func: funcs,
            structs,
            globals: Vec::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Function definition tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_void_function() {
        let func = create_function("main", None, create_block(vec![]));
        let module = create_module(vec![func], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("function main()"));
        assert!(result.contains("main();"));
    }

    #[test]
    fn test_function_with_return() {
        let ret = Statement::Return(Some(Expression::Int(42)));
        let func = create_function("answer", Some(AstType::Int), create_block(vec![ret]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("function answer()"));
        assert!(result.contains("return 42"));
    }

    #[test]
    fn test_function_with_arguments() {
        let arg_a = create_variable("a", AstType::Int);
        let arg_b = create_variable("b", AstType::Int);
        let ret = Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Variable("a".to_string())),
            op: BinOp::Addition,
            rhs: Box::new(Expression::Variable("b".to_string())),
        }));
        let func = create_function_with_args(
            "add",
            vec![arg_a, arg_b],
            Some(AstType::Int),
            create_block(vec![ret]),
        );
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("function add(a, b)"));
        assert!(result.contains("return a + b"));
    }

    // -------------------------------------------------------------------------
    // Arithmetic / binary-op tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_arithmetic_operations() {
        let cases = vec![
            (BinOp::Addition, "+"),
            (BinOp::Subtraction, "-"),
            (BinOp::Multiplication, "*"),
            (BinOp::Division, "/"),
            (BinOp::Modulus, "%"),
        ];

        for (op, sym) in cases {
            let expr = Expression::BinOp {
                lhs: Box::new(Expression::Int(10)),
                op,
                rhs: Box::new(Expression::Int(5)),
            };
            let ret = Statement::Return(Some(expr));
            let func = create_function("calc", Some(AstType::Int), create_block(vec![ret]));
            let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
            let result = JsGenerator::generate(module).unwrap();

            let expected = format!("10 {} 5", sym);
            assert!(
                result.contains(&expected),
                "Expected '{}' in output for op {:?}, got:\n{}",
                expected,
                sym,
                result
            );
        }
    }

    #[test]
    fn test_comparison_operations() {
        let cases = vec![
            (BinOp::Equal, "==="),
            (BinOp::NotEqual, "!=="),
            (BinOp::LessThan, "<"),
            (BinOp::LessThanOrEqual, "<="),
            (BinOp::GreaterThan, ">"),
            (BinOp::GreaterThanOrEqual, ">="),
        ];

        for (op, sym) in cases {
            let expr = Expression::BinOp {
                lhs: Box::new(Expression::Variable("a".to_string())),
                op,
                rhs: Box::new(Expression::Variable("b".to_string())),
            };
            let ret = Statement::Return(Some(expr));
            let a = create_variable("a", AstType::Int);
            let b = create_variable("b", AstType::Int);
            let func = create_function_with_args(
                "cmp",
                vec![a, b],
                Some(AstType::Bool),
                create_block(vec![ret]),
            );
            let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
            let result = JsGenerator::generate(module).unwrap();

            let expected = format!("a {} b", sym);
            assert!(
                result.contains(&expected),
                "Expected '{}' in output, got:\n{}",
                expected,
                result
            );
        }
    }

    #[test]
    fn test_logical_operations() {
        let cases = vec![(BinOp::And, "&&"), (BinOp::Or, "||")];

        for (op, sym) in cases {
            let expr = Expression::BinOp {
                lhs: Box::new(Expression::Bool(true)),
                op,
                rhs: Box::new(Expression::Bool(false)),
            };
            let ret = Statement::Return(Some(expr));
            let func = create_function("logic", Some(AstType::Bool), create_block(vec![ret]));
            let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
            let result = JsGenerator::generate(module).unwrap();

            let expected = format!("true {} false", sym);
            assert!(
                result.contains(&expected),
                "Expected '{}' in output, got:\n{}",
                expected,
                result
            );
        }
    }

    // -------------------------------------------------------------------------
    // Variable declaration / assignment
    // -------------------------------------------------------------------------

    #[test]
    fn test_variable_declaration_with_value() {
        let decl = Statement::Declare {
            variable: create_variable("x", AstType::Int),
            value: Some(Expression::Int(7)),
        };
        let ret = Statement::Return(Some(Expression::Variable("x".to_string())));
        let func = create_function("get_x", Some(AstType::Int), create_block(vec![decl, ret]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("var x = 7"));
        assert!(result.contains("return x"));
    }

    #[test]
    fn test_variable_declaration_no_value() {
        let decl = Statement::Declare {
            variable: create_variable("x", AstType::Int),
            value: None,
        };
        let func = create_function("decl_only", None, create_block(vec![decl]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("var x"));
    }

    #[test]
    fn test_variable_assignment() {
        let decl = Statement::Declare {
            variable: create_variable("x", AstType::Int),
            value: Some(Expression::Int(1)),
        };
        let assign = Statement::Assign {
            lhs: Box::new(Expression::Variable("x".to_string())),
            rhs: Box::new(Expression::Int(99)),
        };
        let func = create_function("reassign", None, create_block(vec![decl, assign]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("var x = 1"));
        assert!(result.contains("x = 99"));
    }

    // -------------------------------------------------------------------------
    // Conditional tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_if_statement() {
        let if_body = create_block(vec![Statement::Return(Some(Expression::Int(1)))]);
        let if_stmt = Statement::If {
            condition: Expression::Bool(true),
            body: Box::new(if_body),
            else_branch: None,
        };
        let func = create_function("branching", Some(AstType::Int), create_block(vec![if_stmt]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("if (true)"));
        assert!(result.contains("return 1"));
    }

    #[test]
    fn test_if_else_statement() {
        let if_body = create_block(vec![Statement::Return(Some(Expression::Int(1)))]);
        let else_body = create_block(vec![Statement::Return(Some(Expression::Int(0)))]);
        let if_stmt = Statement::If {
            condition: Expression::Variable("flag".to_string()),
            body: Box::new(if_body),
            else_branch: Some(Box::new(else_body)),
        };
        let flag = create_variable("flag", AstType::Bool);
        let func = create_function_with_args(
            "branch_else",
            vec![flag],
            Some(AstType::Int),
            create_block(vec![if_stmt]),
        );
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("if (flag)"));
        assert!(result.contains("else"));
        assert!(result.contains("return 0"));
    }

    // -------------------------------------------------------------------------
    // Loop tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_while_loop() {
        let body = create_block(vec![Statement::Break]);
        let while_stmt = Statement::While {
            condition: Expression::Bool(true),
            body: Box::new(body),
        };
        let func = create_function("loop_fn", None, create_block(vec![while_stmt]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("while (true)"));
        assert!(result.contains("break"));
    }

    #[test]
    fn test_for_loop() {
        let items_var = create_variable("item", AstType::Int);
        let array_expr = Expression::Array {
            capacity: 3,
            elements: vec![Expression::Int(1), Expression::Int(2), Expression::Int(3)],
        };
        let body = create_block(vec![Statement::Continue]);
        let for_stmt = Statement::For {
            ident: items_var,
            expr: array_expr,
            body: Box::new(body),
        };
        let func = create_function("for_fn", None, create_block(vec![for_stmt]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("for ("));
        assert!(result.contains("iter_item"));
        assert!(result.contains("continue"));
    }

    // -------------------------------------------------------------------------
    // Array tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_array_literal() {
        let arr = Expression::Array {
            capacity: 3,
            elements: vec![Expression::Int(1), Expression::Int(2), Expression::Int(3)],
        };
        let decl = Statement::Declare {
            variable: create_variable("nums", AstType::Array(Box::new(AstType::Int), None)),
            value: Some(arr),
        };
        let func = create_function("arr_fn", None, create_block(vec![decl]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("var nums = [1, 2, 3]"));
    }

    #[test]
    fn test_array_access() {
        let access = Expression::ArrayAccess {
            name: "arr".to_string(),
            index: Box::new(Expression::Int(0)),
        };
        let ret = Statement::Return(Some(access));
        let arr_var = create_variable("arr", AstType::Array(Box::new(AstType::Int), None));
        let func = create_function_with_args(
            "first",
            vec![arr_var],
            Some(AstType::Int),
            create_block(vec![ret]),
        );
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("arr[0]"));
    }

    #[test]
    fn test_uninitialized_array_declaration() {
        let decl = Statement::Declare {
            variable: create_variable("buf", AstType::Array(Box::new(AstType::Int), None)),
            value: None,
        };
        let func = create_function("buf_fn", None, create_block(vec![decl]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        // Uninitialized arrays should be initialized to [] to avoid runtime errors
        assert!(result.contains("var buf = []"));
    }

    // -------------------------------------------------------------------------
    // Struct tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_struct_definition() {
        let struct_def = StructDef {
            name: "Point".to_string(),
            fields: vec![
                create_variable("x", AstType::Int),
                create_variable("y", AstType::Int),
            ],
            methods: Vec::new(),
        };
        let func = create_function("main", None, create_block(vec![]));
        let module = create_module(vec![func], vec![struct_def]);
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("function Point(args)"));
        assert!(result.contains("this.x = args.x"));
        assert!(result.contains("this.y = args.y"));
    }

    #[test]
    fn test_struct_initialization() {
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Box::new(Expression::Int(3)));
        fields.insert("y".to_string(), Box::new(Expression::Int(4)));

        let init = Expression::StructInitialization {
            name: "Point".to_string(),
            fields,
        };
        let decl = Statement::Declare {
            variable: create_variable("p", AstType::Struct("Point".to_string())),
            value: Some(init),
        };
        let func = create_function("make_point", None, create_block(vec![decl]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("new Point({"));
    }

    // -------------------------------------------------------------------------
    // Function call test
    // -------------------------------------------------------------------------

    #[test]
    fn test_function_call() {
        let call = Expression::FunctionCall {
            fn_name: "print".to_string(),
            args: vec![Expression::Str("hello".to_string())],
        };
        let stmt = Statement::Exp(call);
        let func = create_function("main", None, create_block(vec![stmt]));
        let module = create_module(vec![func], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("print("));
        assert!(result.contains("\"hello\""));
    }

    // -------------------------------------------------------------------------
    // String literal test
    // -------------------------------------------------------------------------

    #[test]
    fn test_string_literal() {
        let ret = Statement::Return(Some(Expression::Str("world".to_string())));
        let func = create_function("greet", Some(AstType::Str), create_block(vec![ret]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.contains("\"world\""));
    }

    // -------------------------------------------------------------------------
    // Boolean literal test
    // -------------------------------------------------------------------------

    #[test]
    fn test_boolean_literals() {
        let ret_true = Statement::Return(Some(Expression::Bool(true)));
        let func = create_function("get_true", Some(AstType::Bool), create_block(vec![ret_true]));
        let module = create_module(vec![func, create_function("main", None, create_block(vec![]))], Vec::new());
        let result = JsGenerator::generate(module).unwrap();
        assert!(result.contains("return true"));

        let ret_false = Statement::Return(Some(Expression::Bool(false)));
        let func2 = create_function("get_false", Some(AstType::Bool), create_block(vec![ret_false]));
        let module2 = create_module(vec![func2, create_function("main", None, create_block(vec![]))], Vec::new());
        let result2 = JsGenerator::generate(module2).unwrap();
        assert!(result2.contains("return false"));
    }

    // -------------------------------------------------------------------------
    // main(); is always appended
    // -------------------------------------------------------------------------

    #[test]
    fn test_main_call_appended() {
        let func = create_function("main", None, create_block(vec![]));
        let module = create_module(vec![func], Vec::new());
        let result = JsGenerator::generate(module).unwrap();

        assert!(result.ends_with("main();"));
    }
}
