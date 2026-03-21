/**
 * Tests for the QBE generator implementation
 */
#[cfg(test)]
mod tests {
    use crate::ast::types::Type as AstType;
    use crate::ast::*;
    use crate::generator::qbe::QbeGenerator;
    use crate::generator::Generator;

    /// Helper function to parse the QBE output and get a normalized representation for comparison
    fn normalize_qbe(qbe_output: &str) -> String {
        // Remove empty lines and trim whitespace to make comparison more robust
        qbe_output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim())
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// Helper function to create a basic function AST node
    fn create_function(name: &str, ret_type: Option<AstType>, body: Statement) -> Function {
        Function {
            name: name.to_string(),
            arguments: Vec::new(),
            ret_type,
            body,
        }
    }

    /// Helper function to create a function with arguments
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

    /// Helper function to create a variable AST node
    fn create_variable(name: &str, typ: AstType) -> Variable {
        Variable {
            name: name.to_string(),
            ty: Some(typ),
        }
    }

    fn create_int_expr(value: usize) -> Expression {
        Expression::Int(value)
    }

    fn create_bool_expr(value: bool) -> Expression {
        Expression::Bool(value)
    }

    fn create_str_expr(value: &str) -> Expression {
        Expression::Str(value.to_string())
    }

    fn create_var_expr(name: &str) -> Expression {
        Expression::Variable(name.to_string())
    }

    fn create_binop_expr(lhs: Expression, op: BinOp, rhs: Expression) -> Expression {
        Expression::BinOp {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }

    fn create_call_expr(fn_name: &str, args: Vec<Expression>) -> Expression {
        Expression::FunctionCall {
            fn_name: fn_name.to_string(),
            args,
        }
    }

    fn create_return_stmt(expr: Option<Expression>) -> Statement {
        Statement::Return(expr)
    }

    fn create_declare_stmt(name: &str, typ: AstType, value: Option<Expression>) -> Statement {
        Statement::Declare {
            variable: create_variable(name, typ),
            value,
        }
    }

    fn create_assign_stmt(lhs: Expression, rhs: Expression) -> Statement {
        Statement::Assign {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    fn create_block_stmt(statements: Vec<Statement>) -> Statement {
        Statement::Block {
            statements,
            scope: Vec::new(),
        }
    }

    fn create_if_stmt(
        condition: Expression,
        body: Statement,
        else_branch: Option<Statement>,
    ) -> Statement {
        Statement::If {
            condition,
            body: Box::new(body),
            else_branch: else_branch.map(Box::new),
        }
    }

    fn create_while_stmt(condition: Expression, body: Statement) -> Statement {
        Statement::While {
            condition,
            body: Box::new(body),
        }
    }

    fn create_struct_def(name: &str, fields: Vec<Variable>) -> StructDef {
        StructDef {
            name: name.to_string(),
            fields,
            methods: Vec::new(),
        }
    }

    fn create_module(funcs: Vec<Function>, structs: Vec<StructDef>) -> Module {
        Module {
            func: funcs,
            structs,
            globals: Vec::new(),
        }
    }

    #[test]
    fn test_empty_function() {
        let ret_stmt = create_return_stmt(Some(create_int_expr(0)));
        let func = create_function("empty", Some(AstType::Int), ret_stmt);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function w $empty() {
            @start
                %tmp.1 =w copy 0
                ret %tmp.1
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_function_with_args() {
        let arg1 = create_variable("a", AstType::Int);
        let arg2 = create_variable("b", AstType::Int);
        let add_expr =
            create_binop_expr(create_var_expr("a"), BinOp::Addition, create_var_expr("b"));
        let ret_stmt = create_return_stmt(Some(add_expr));
        let func = create_function_with_args("add", vec![arg1, arg2], Some(AstType::Int), ret_stmt);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function w $add(w %tmp.1, w %tmp.2) {
            @start
                %tmp.3 =w add %tmp.1, %tmp.2
                ret %tmp.3
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_void_function() {
        let func = create_function("void_func", None, create_block_stmt(vec![]));
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function $void_func() {
            @start
                ret
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_variable_declaration() {
        let decl_stmt = create_declare_stmt("x", AstType::Int, Some(create_int_expr(42)));
        let ret_stmt = create_return_stmt(Some(create_var_expr("x")));
        let block = create_block_stmt(vec![decl_stmt, ret_stmt]);
        let func = create_function("var_decl", Some(AstType::Int), block);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function w $var_decl() {
            @start
                %tmp.2 =w copy 42
                %tmp.1 =w copy %tmp.2
                ret %tmp.1
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_variable_assignment() {
        let decl_stmt = create_declare_stmt("x", AstType::Int, Some(create_int_expr(42)));
        let assign_stmt = create_assign_stmt(create_var_expr("x"), create_int_expr(100));
        let ret_stmt = create_return_stmt(Some(create_var_expr("x")));
        let block = create_block_stmt(vec![decl_stmt, assign_stmt, ret_stmt]);
        let func = create_function("var_assign", Some(AstType::Int), block);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function w $var_assign() {
            @start
                %tmp.2 =w copy 42
                %tmp.1 =w copy %tmp.2
                %tmp.3 =w copy 100
                %tmp.1 =w copy %tmp.3
                ret %tmp.1
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_arithmetic_operations() {
        let operations = vec![
            (BinOp::Addition, "add"),
            (BinOp::Subtraction, "sub"),
            (BinOp::Multiplication, "mul"),
            (BinOp::Division, "div"),
        ];

        for (op, op_name) in operations {
            let decl_a = create_declare_stmt("a", AstType::Int, Some(create_int_expr(10)));
            let decl_b = create_declare_stmt("b", AstType::Int, Some(create_int_expr(5)));
            let binop_expr = create_binop_expr(create_var_expr("a"), op, create_var_expr("b"));
            let ret_stmt = create_return_stmt(Some(binop_expr));
            let block = create_block_stmt(vec![decl_a, decl_b, ret_stmt]);
            let func = create_function(&format!("test_{}", op_name), Some(AstType::Int), block);
            let module = create_module(vec![func], Vec::new());
            let result = QbeGenerator::generate(module).unwrap();

            let expected = normalize_qbe(&format!(
                r#"
                export function w $test_{op_name}() {{
                @start
                    %tmp.2 =w copy 10
                    %tmp.1 =w copy %tmp.2
                    %tmp.4 =w copy 5
                    %tmp.3 =w copy %tmp.4
                    %tmp.5 =w {op_name} %tmp.1, %tmp.3
                    ret %tmp.5
                }}
            "#
            ));

            assert_eq!(normalize_qbe(&result), expected);
        }
    }

    #[test]
    fn test_comparison_operations() {
        let operations = vec![
            (BinOp::Equal, "ceq", "ceqw"),
            (BinOp::NotEqual, "cne", "cnew"),
            (BinOp::LessThan, "cslt", "csltw"),
            (BinOp::LessThanOrEqual, "csle", "cslew"),
            (BinOp::GreaterThan, "csgt", "csgtw"),
            (BinOp::GreaterThanOrEqual, "csge", "csgew"),
        ];

        for (op, op_name, qbe_instr) in operations {
            let decl_a = create_declare_stmt("a", AstType::Int, Some(create_int_expr(10)));
            let decl_b = create_declare_stmt("b", AstType::Int, Some(create_int_expr(5)));
            let binop_expr = create_binop_expr(create_var_expr("a"), op, create_var_expr("b"));
            let ret_stmt = create_return_stmt(Some(binop_expr));
            let block = create_block_stmt(vec![decl_a, decl_b, ret_stmt]);
            let func = create_function(&format!("test_{}", op_name), Some(AstType::Int), block);
            let module = create_module(vec![func], Vec::new());
            let result = QbeGenerator::generate(module).unwrap();

            let expected = normalize_qbe(&format!(
                r#"
                export function w $test_{op_name}() {{
                @start
                    %tmp.2 =w copy 10
                    %tmp.1 =w copy %tmp.2
                    %tmp.4 =w copy 5
                    %tmp.3 =w copy %tmp.4
                    %tmp.5 =w {qbe_instr} %tmp.1, %tmp.3
                    ret %tmp.5
                }}
            "#
            ));

            assert_eq!(normalize_qbe(&result), expected);
        }
    }

    #[test]
    fn test_if_statement() {
        let decl_cond = create_declare_stmt("cond", AstType::Int, Some(create_int_expr(1)));
        let if_stmt = create_if_stmt(
            create_var_expr("cond"),
            create_return_stmt(Some(create_int_expr(10))),
            None,
        );
        let block = create_block_stmt(vec![
            decl_cond,
            if_stmt,
            create_return_stmt(Some(create_int_expr(20))),
        ]);
        let func = create_function("test_if", Some(AstType::Int), block);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function w $test_if() {
            @start
                %tmp.2 =w copy 1
                %tmp.1 =w copy %tmp.2
                jnz %tmp.1, @cond.3.if, @cond.3.end
            @cond.3.if
                %tmp.4 =w copy 10
                ret %tmp.4
            @cond.3.end
                %tmp.5 =w copy 20
                ret %tmp.5
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_if_else_statement() {
        let decl_cond = create_declare_stmt("cond", AstType::Int, Some(create_int_expr(1)));
        let if_stmt = create_if_stmt(
            create_var_expr("cond"),
            create_return_stmt(Some(create_int_expr(10))),
            Some(create_return_stmt(Some(create_int_expr(20)))),
        );
        let block = create_block_stmt(vec![
            decl_cond,
            if_stmt,
            create_return_stmt(Some(create_int_expr(30))),
        ]);
        let func = create_function("test_if_else", Some(AstType::Int), block);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function w $test_if_else() {
            @start
                %tmp.2 =w copy 1
                %tmp.1 =w copy %tmp.2
                jnz %tmp.1, @cond.3.if, @cond.3.else
            @cond.3.if
                %tmp.4 =w copy 10
                ret %tmp.4
            @cond.3.else
                %tmp.5 =w copy 20
                ret %tmp.5
            @cond.3.end
                %tmp.6 =w copy 30
                ret %tmp.6
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_while_loop() {
        let decl_i = create_declare_stmt("i", AstType::Int, Some(create_int_expr(0)));
        let decl_sum = create_declare_stmt("sum", AstType::Int, Some(create_int_expr(0)));
        let loop_body = create_block_stmt(vec![
            create_assign_stmt(
                create_var_expr("sum"),
                create_binop_expr(
                    create_var_expr("sum"),
                    BinOp::Addition,
                    create_var_expr("i"),
                ),
            ),
            create_assign_stmt(
                create_var_expr("i"),
                create_binop_expr(create_var_expr("i"), BinOp::Addition, create_int_expr(1)),
            ),
        ]);
        let while_stmt = create_while_stmt(
            create_binop_expr(create_var_expr("i"), BinOp::LessThan, create_int_expr(10)),
            loop_body,
        );
        let block = create_block_stmt(vec![
            decl_i,
            decl_sum,
            while_stmt,
            create_return_stmt(Some(create_var_expr("sum"))),
        ]);
        let func = create_function("test_while", Some(AstType::Int), block);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function w $test_while() {
            @start
                %tmp.2 =w copy 0
                %tmp.1 =w copy %tmp.2
                %tmp.4 =w copy 0
                %tmp.3 =w copy %tmp.4
            @loop.5.cond
                %tmp.6 =w copy 10
                %tmp.7 =w csltw %tmp.1, %tmp.6
                jnz %tmp.7, @loop.5.body, @loop.5.end
            @loop.5.body
                %tmp.8 =w add %tmp.3, %tmp.1
                %tmp.3 =w copy %tmp.8
                %tmp.9 =w copy 1
                %tmp.10 =w add %tmp.1, %tmp.9
                %tmp.1 =w copy %tmp.10
                jmp @loop.5.cond
            @loop.5.end
                ret %tmp.3
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_break_continue() {
        let decl_i = create_declare_stmt("i", AstType::Int, Some(create_int_expr(0)));
        let if_break = create_if_stmt(
            create_binop_expr(create_var_expr("i"), BinOp::Equal, create_int_expr(5)),
            Statement::Break,
            None,
        );
        let if_continue = create_if_stmt(
            create_binop_expr(
                create_binop_expr(create_var_expr("i"), BinOp::Modulus, create_int_expr(2)),
                BinOp::Equal,
                create_int_expr(0),
            ),
            Statement::Continue,
            None,
        );
        let loop_body = create_block_stmt(vec![
            if_break,
            if_continue,
            create_assign_stmt(
                create_var_expr("i"),
                create_binop_expr(create_var_expr("i"), BinOp::Addition, create_int_expr(1)),
            ),
        ]);
        let while_stmt = create_while_stmt(
            create_binop_expr(create_var_expr("i"), BinOp::LessThan, create_int_expr(10)),
            loop_body,
        );
        let block = create_block_stmt(vec![
            decl_i,
            while_stmt,
            create_return_stmt(Some(create_var_expr("i"))),
        ]);
        let func = create_function("test_break_continue", Some(AstType::Int), block);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function w $test_break_continue() {
            @start
                %tmp.2 =w copy 0
                %tmp.1 =w copy %tmp.2
            @loop.3.cond
                %tmp.4 =w copy 10
                %tmp.5 =w csltw %tmp.1, %tmp.4
                jnz %tmp.5, @loop.3.body, @loop.3.end
            @loop.3.body
                %tmp.6 =w copy 5
                %tmp.7 =w ceqw %tmp.1, %tmp.6
                jnz %tmp.7, @cond.8.if, @cond.8.end
            @cond.8.if
                jmp @loop.3.end
            @cond.8.end
                %tmp.9 =w copy 2
                %tmp.10 =w rem %tmp.1, %tmp.9
                %tmp.11 =w copy 0
                %tmp.12 =w ceqw %tmp.10, %tmp.11
                jnz %tmp.12, @cond.13.if, @cond.13.end
            @cond.13.if
                jmp @loop.3.cond
            @cond.13.end
                %tmp.14 =w copy 1
                %tmp.15 =w add %tmp.1, %tmp.14
                %tmp.1 =w copy %tmp.15
                jmp @loop.3.cond
            @loop.3.end
                ret %tmp.1
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_struct_definition() {
        let point_struct = create_struct_def(
            "Point",
            vec![
                create_variable("x", AstType::Int),
                create_variable("y", AstType::Int),
            ],
        );
        let func = create_function("test_struct", None, create_block_stmt(vec![]));
        let module = create_module(vec![func], vec![point_struct]);
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            type :struct.1 = align 4 { w, w }
            export function $test_struct() {
            @start
                ret
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_string_literal() {
        let str_expr = create_str_expr("Hello, world!");
        let decl_stmt = create_declare_stmt("message", AstType::Str, Some(str_expr));
        let func = create_function("test_string", None, create_block_stmt(vec![decl_stmt]));
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function $test_string() {
            @start
                %tmp.1 =l copy $string.2
                ret
            }
            export data $string.2 = { b "Hello, world!", b 0 }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_function_call() {
        let call_expr = create_call_expr("print", vec![create_str_expr("Hello, world!")]);
        let stmt = Statement::Exp(call_expr);
        let func = create_function("test_call", None, create_block_stmt(vec![stmt]));
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function $test_call() {
            @start
                %tmp.2 =w call $print(l $string.1)
                ret
            }
            export data $string.1 = { b "Hello, world!", b 0 }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_compound_expressions() {
        let expr = create_binop_expr(
            create_binop_expr(create_int_expr(1), BinOp::Addition, create_int_expr(2)),
            BinOp::Multiplication,
            create_binop_expr(create_int_expr(3), BinOp::Addition, create_int_expr(4)),
        );
        let ret_stmt = create_return_stmt(Some(expr));
        let func = create_function("compound_expr", Some(AstType::Int), ret_stmt);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function w $compound_expr() {
            @start
                %tmp.1 =w copy 1
                %tmp.2 =w copy 2
                %tmp.3 =w add %tmp.1, %tmp.2
                %tmp.4 =w copy 3
                %tmp.5 =w copy 4
                %tmp.6 =w add %tmp.4, %tmp.5
                %tmp.7 =w mul %tmp.3, %tmp.6
                ret %tmp.7
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_string_concatenation() {
        let decl_a = create_declare_stmt("a", AstType::Str, Some(create_str_expr("hello ")));
        let decl_b = create_declare_stmt("b", AstType::Str, Some(create_str_expr("world")));
        let concat_expr =
            create_binop_expr(create_var_expr("a"), BinOp::Addition, create_var_expr("b"));
        let ret_stmt = create_return_stmt(Some(concat_expr));
        let block = create_block_stmt(vec![decl_a, decl_b, ret_stmt]);
        let func = create_function("test_concat", Some(AstType::Str), block);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function l $test_concat() {
            @start
                %tmp.1 =l copy $string.2
                %tmp.3 =l copy $string.4
                %tmp.5 =l call $_str_concat(l %tmp.1, l %tmp.3)
                ret %tmp.5
            }
            export data $string.2 = { b "hello ", b 0 }
            export data $string.4 = { b "world", b 0 }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_int_addition_not_str_concat() {
        let decl_a = create_declare_stmt("a", AstType::Int, Some(create_int_expr(1)));
        let decl_b = create_declare_stmt("b", AstType::Int, Some(create_int_expr(2)));
        let add_expr =
            create_binop_expr(create_var_expr("a"), BinOp::Addition, create_var_expr("b"));
        let ret_stmt = create_return_stmt(Some(add_expr));
        let block = create_block_stmt(vec![decl_a, decl_b, ret_stmt]);
        let func = create_function("test_int_add", Some(AstType::Int), block);
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function w $test_int_add() {
            @start
                %tmp.2 =w copy 1
                %tmp.1 =w copy %tmp.2
                %tmp.4 =w copy 2
                %tmp.3 =w copy %tmp.4
                %tmp.5 =w add %tmp.1, %tmp.3
                ret %tmp.5
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_function_call_return_type_int() {
        // Define add_one(x: int): int that returns x + 1
        let arg = create_variable("x", AstType::Int);
        let add_expr = create_binop_expr(create_var_expr("x"), BinOp::Addition, create_int_expr(1));
        let add_one = create_function_with_args(
            "add_one",
            vec![arg],
            Some(AstType::Int),
            create_return_stmt(Some(add_expr)),
        );

        // Define main() that calls add_one and returns the result
        let call_expr = create_call_expr("add_one", vec![create_int_expr(5)]);
        let ret_stmt = create_return_stmt(Some(call_expr));
        let main_func = create_function("main", Some(AstType::Int), ret_stmt);

        let module = create_module(vec![add_one, main_func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        // The call to add_one should use =w (Word) since it returns int
        let expected = normalize_qbe(
            r#"
            export function w $add_one(w %tmp.1) {
            @start
                %tmp.2 =w copy 1
                %tmp.3 =w add %tmp.1, %tmp.2
                ret %tmp.3
            }
            export function w $main() {
            @start
                %tmp.4 =w copy 5
                %tmp.5 =w call $add_one(w %tmp.4)
                ret %tmp.5
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_function_call_return_type_string() {
        // Define greet(name: string): string that returns name
        let arg = create_variable("name", AstType::Str);
        let greet = create_function_with_args(
            "greet",
            vec![arg],
            Some(AstType::Str),
            create_return_stmt(Some(create_var_expr("name"))),
        );

        // Define main() that calls greet and stores the result
        let call_expr = create_call_expr("greet", vec![create_str_expr("World")]);
        let decl = create_declare_stmt("msg", AstType::Str, Some(call_expr));
        let main_func = create_function("main", None, create_block_stmt(vec![decl]));

        let module = create_module(vec![greet, main_func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        // The call to greet should use =l (Long) since it returns string
        let expected = normalize_qbe(
            r#"
            export function l $greet(l %tmp.1) {
            @start
                ret %tmp.1
            }
            export function $main() {
            @start
                %tmp.4 =l call $greet(l $string.3)
                %tmp.2 =l copy %tmp.4
                ret
            }
            export data $string.3 = { b "World", b 0 }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_function_call_unknown_defaults_to_word() {
        // Calling an unknown/external function should fall back to Word
        let call_expr = create_call_expr("unknown_fn", vec![create_int_expr(42)]);
        let stmt = Statement::Exp(call_expr);
        let func = create_function("test", None, create_block_stmt(vec![stmt]));
        let module = create_module(vec![func], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function $test() {
            @start
                %tmp.1 =w copy 42
                %tmp.2 =w call $unknown_fn(w %tmp.1)
                ret
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_boolean_operations() {
        let operations = vec![(BinOp::And, "and"), (BinOp::Or, "or")];

        for (op, op_name) in operations {
            let decl_a = create_declare_stmt("a", AstType::Bool, Some(create_bool_expr(true)));
            let decl_b = create_declare_stmt("b", AstType::Bool, Some(create_bool_expr(false)));
            let binop_expr = create_binop_expr(create_var_expr("a"), op, create_var_expr("b"));
            let ret_stmt = create_return_stmt(Some(binop_expr));
            let block = create_block_stmt(vec![decl_a, decl_b, ret_stmt]);
            let func = create_function(&format!("test_{}", op_name), Some(AstType::Bool), block);
            let module = create_module(vec![func], Vec::new());
            let result = QbeGenerator::generate(module).unwrap();

            let expected = normalize_qbe(&format!(
                r#"
                export function w $test_{op_name}() {{
                @start
                    %tmp.2 =w copy 1
                    %tmp.1 =w copy %tmp.2
                    %tmp.4 =w copy 0
                    %tmp.3 =w copy %tmp.4
                    %tmp.5 =w {op_name} %tmp.1, %tmp.3
                    ret %tmp.5
                }}
            "#
            ));

            assert_eq!(normalize_qbe(&result), expected);
        }
    }

    #[test]
    fn test_any_type_parameter_int_widening() {
        // Define print_any(x: any) with empty body
        let print_any = create_function_with_args(
            "print_any",
            vec![create_variable("x", AstType::Any)],
            None,
            create_block_stmt(vec![]),
        );

        // Define main() that calls print_any(5) — int (Word) must be widened to Long
        let call = create_call_expr("print_any", vec![create_int_expr(5)]);
        let main_fn = create_function(
            "main",
            None,
            create_block_stmt(vec![Statement::Exp(call)]),
        );

        let module = create_module(vec![print_any, main_fn], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        let expected = normalize_qbe(
            r#"
            export function $print_any(l %tmp.1) {
            @start
                ret
            }
            export function $main() {
            @start
                %tmp.2 =w copy 5
                %tmp.4 =l extuw %tmp.2
                %tmp.3 =w call $print_any(l %tmp.4)
                ret
            }
        "#,
        );

        assert_eq!(normalize_qbe(&result), expected);
    }

    #[test]
    fn test_any_type_parameter_string_no_widening() {
        // Define print_any(x: any) with empty body
        let print_any = create_function_with_args(
            "print_any",
            vec![create_variable("x", AstType::Any)],
            None,
            create_block_stmt(vec![]),
        );

        // Define main() that calls print_any("hello") — string is already Long, no widening
        let call = create_call_expr("print_any", vec![create_str_expr("hello")]);
        let main_fn = create_function(
            "main",
            None,
            create_block_stmt(vec![Statement::Exp(call)]),
        );

        let module = create_module(vec![print_any, main_fn], Vec::new());
        let result = QbeGenerator::generate(module).unwrap();

        // String arg should pass through as Long without extuw
        assert!(!result.contains("extuw"));
        assert!(!result.contains("extub"));
    }
}
