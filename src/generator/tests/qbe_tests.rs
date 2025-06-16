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

    /// Helper function to create an integer literal expression
    fn create_int_expr(value: usize) -> Expression {
        Expression::Int(value)
    }

    /// Helper function to create a boolean literal expression
    fn create_bool_expr(value: bool) -> Expression {
        Expression::Bool(value)
    }

    /// Helper function to create a string literal expression
    fn create_str_expr(value: &str) -> Expression {
        Expression::Str(value.to_string())
    }

    /// Helper function to create a variable expression
    fn create_var_expr(name: &str) -> Expression {
        Expression::Variable(name.to_string())
    }

    /// Helper function to create a binary operation expression
    fn create_binop_expr(lhs: Expression, op: BinOp, rhs: Expression) -> Expression {
        Expression::BinOp {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }

    /// Helper function to create a function call expression
    fn create_call_expr(fn_name: &str, args: Vec<Expression>) -> Expression {
        Expression::FunctionCall {
            fn_name: fn_name.to_string(),
            args,
        }
    }

    /// Helper to create a return statement
    fn create_return_stmt(expr: Option<Expression>) -> Statement {
        Statement::Return(expr)
    }

    /// Helper to create a variable declaration statement
    fn create_declare_stmt(name: &str, typ: AstType, value: Option<Expression>) -> Statement {
        Statement::Declare {
            variable: create_variable(name, typ),
            value,
        }
    }

    /// Helper to create an assignment statement
    fn create_assign_stmt(lhs: Expression, rhs: Expression) -> Statement {
        Statement::Assign {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Helper to create a block statement
    fn create_block_stmt(statements: Vec<Statement>) -> Statement {
        Statement::Block {
            statements,
            scope: Vec::new(),
        }
    }

    /// Helper to create an if statement
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

    /// Helper to create a while statement
    fn create_while_stmt(condition: Expression, body: Statement) -> Statement {
        Statement::While {
            condition,
            body: Box::new(body),
        }
    }

    /// Helper to create a struct definition
    fn create_struct_def(name: &str, fields: Vec<Variable>) -> StructDef {
        StructDef {
            name: name.to_string(),
            fields,
            methods: Vec::new(),
        }
    }

    /// Helper to create a module
    fn create_module(funcs: Vec<Function>, structs: Vec<StructDef>) -> Module {
        Module {
            func: funcs,
            structs,
            globals: Vec::new(),
        }
    }

    #[test]
    fn test_empty_function() {
        // Create a simple function that returns 0
        let ret_stmt = create_return_stmt(Some(create_int_expr(0)));
        let func = create_function("empty", Some(AstType::Int), ret_stmt);

        let module = create_module(vec![func], Vec::new());

        let result = QbeGenerator::generate(module).unwrap();

        // Check the generated QBE code
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
        // Create a function that adds two integers
        let arg1 = create_variable("a", AstType::Int);
        let arg2 = create_variable("b", AstType::Int);

        let add_expr =
            create_binop_expr(create_var_expr("a"), BinOp::Addition, create_var_expr("b"));

        let ret_stmt = create_return_stmt(Some(add_expr));
        let func = create_function_with_args("add", vec![arg1, arg2], Some(AstType::Int), ret_stmt);

        let module = create_module(vec![func], Vec::new());

        let result = QbeGenerator::generate(module).unwrap();

        // The exact temporary names may vary, so we'll check for basic structure
        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("export function w $add(w %"));
        assert!(result_norm.contains("add %"));
        assert!(result_norm.contains("ret %"));
    }

    #[test]
    fn test_void_function() {
        // Create a function that doesn't return anything
        let func = create_function("void_func", None, create_block_stmt(vec![]));

        let module = create_module(vec![func], Vec::new());

        let result = QbeGenerator::generate(module).unwrap();

        // Check the generated QBE code
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
        // Create a function with variable declaration
        let decl_stmt = create_declare_stmt("x", AstType::Int, Some(create_int_expr(42)));
        let ret_stmt = create_return_stmt(Some(create_var_expr("x")));

        let block = create_block_stmt(vec![decl_stmt, ret_stmt]);
        let func = create_function("var_decl", Some(AstType::Int), block);

        let module = create_module(vec![func], Vec::new());

        let result = QbeGenerator::generate(module).unwrap();

        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("export function w $var_decl()"));
        assert!(result_norm.contains("copy 42"));
        assert!(result_norm.contains("ret %"));
    }

    #[test]
    fn test_variable_assignment() {
        // Create a function with variable declaration and assignment
        let decl_stmt = create_declare_stmt("x", AstType::Int, Some(create_int_expr(42)));
        let assign_stmt = create_assign_stmt(create_var_expr("x"), create_int_expr(100));
        let ret_stmt = create_return_stmt(Some(create_var_expr("x")));

        let block = create_block_stmt(vec![decl_stmt, assign_stmt, ret_stmt]);
        let func = create_function("var_assign", Some(AstType::Int), block);

        let module = create_module(vec![func], Vec::new());

        let result = QbeGenerator::generate(module).unwrap();

        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("export function w $var_assign()"));
        assert!(result_norm.contains("copy 42"));
        assert!(result_norm.contains("copy 100"));
        assert!(result_norm.contains("ret %"));
    }

    #[test]
    fn test_arithmetic_operations() {
        // Test each arithmetic operation
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

            let result_norm = normalize_qbe(&result);
            assert!(result_norm.contains(&format!("{} %", op_name)));
        }
    }

    #[test]
    fn test_comparison_operations() {
        // Test comparison operations
        let operations = vec![
            (BinOp::Equal, "ceq"),
            (BinOp::NotEqual, "cne"),
            (BinOp::LessThan, "cslt"),
            (BinOp::LessThanOrEqual, "csle"),
            (BinOp::GreaterThan, "csgt"),
            (BinOp::GreaterThanOrEqual, "csge"),
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

            let result_norm = normalize_qbe(&result);
            assert!(result_norm.contains(op_name));
        }
    }

    #[test]
    fn test_if_statement() {
        // Create an if statement
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

        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("jnz %"));
        assert!(result_norm.contains("@cond"));
    }

    #[test]
    fn test_if_else_statement() {
        // Create an if-else statement
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

        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("jnz %"));
        assert!(result_norm.contains("@cond"));
        assert!(result_norm.contains(".if"));
        assert!(result_norm.contains(".else"));
    }

    #[test]
    fn test_while_loop() {
        // Create a while loop
        let decl_i = create_declare_stmt("i", AstType::Int, Some(create_int_expr(0)));
        let decl_sum = create_declare_stmt("sum", AstType::Int, Some(create_int_expr(0)));

        let loop_body = create_block_stmt(vec![
            // sum += i
            create_assign_stmt(
                create_var_expr("sum"),
                create_binop_expr(
                    create_var_expr("sum"),
                    BinOp::Addition,
                    create_var_expr("i"),
                ),
            ),
            // i += 1
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

        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("@loop"));
        assert!(result_norm.contains("jnz %"));
        assert!(result_norm.contains(".cond"));
        assert!(result_norm.contains(".body"));
        assert!(result_norm.contains(".end"));
    }

    #[test]
    fn test_break_continue() {
        // Create a while loop with break and continue
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
            // i += 1
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

        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("jmp @loop"));
        assert!(result_norm.contains(".end"));
        assert!(result_norm.contains(".cond"));
    }

    #[test]
    fn test_struct_definition() {
        // Create a struct definition
        let point_struct = create_struct_def(
            "Point",
            vec![
                create_variable("x", AstType::Int),
                create_variable("y", AstType::Int),
            ],
        );

        // Create a function that uses the struct
        let func = create_function("test_struct", None, create_block_stmt(vec![]));

        let module = create_module(vec![func], vec![point_struct]);

        let result = QbeGenerator::generate(module).unwrap();

        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("type :struct"));
        assert!(result_norm.contains("{ w, w }"));
    }

    #[test]
    fn test_string_literal() {
        // Test string literal generation
        let str_expr = create_str_expr("Hello, world!");

        let decl_stmt = create_declare_stmt("message", AstType::Str, Some(str_expr));

        let func = create_function("test_string", None, create_block_stmt(vec![decl_stmt]));

        let module = create_module(vec![func], Vec::new());

        let result = QbeGenerator::generate(module).unwrap();

        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("data $string"));
        assert!(result_norm.contains("\"Hello, world!\""));
    }

    #[test]
    fn test_function_call() {
        // Test function call generation
        let call_expr = create_call_expr("print", vec![create_str_expr("Hello, world!")]);

        let stmt = Statement::Exp(call_expr);

        let func = create_function("test_call", None, create_block_stmt(vec![stmt]));

        let module = create_module(vec![func], Vec::new());

        let result = QbeGenerator::generate(module).unwrap();

        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("call $print("));
    }

    #[test]
    fn test_compound_expressions() {
        // Test nested expressions
        let expr = create_binop_expr(
            create_binop_expr(create_int_expr(1), BinOp::Addition, create_int_expr(2)),
            BinOp::Multiplication,
            create_binop_expr(create_int_expr(3), BinOp::Addition, create_int_expr(4)),
        );

        let ret_stmt = create_return_stmt(Some(expr));

        let func = create_function("compound_expr", Some(AstType::Int), ret_stmt);

        let module = create_module(vec![func], Vec::new());

        let result = QbeGenerator::generate(module).unwrap();

        let result_norm = normalize_qbe(&result);
        assert!(result_norm.contains("add %"));
        assert!(result_norm.contains("mul %"));
    }

    #[test]
    fn test_boolean_operations() {
        // Test boolean operations (AND, OR)
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

            let result_norm = normalize_qbe(&result);
            assert!(result_norm.contains(&format!("{} %", op_name)));
        }
    }
}
