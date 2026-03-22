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

    fn builtins() -> String {
        let raw = crate::Builtins::get("builtin.js")
            .expect("Could not locate builtin.js")
            .data;
        std::str::from_utf8(&raw)
            .expect("builtin.js is not valid UTF-8")
            .to_string()
    }

    /// Strip the fixed builtins preamble so tests only assert on generated code.
    fn user_code(output: &str) -> &str {
        let prefix = builtins();
        output
            .strip_prefix(prefix.as_str())
            .expect("output did not start with builtins preamble")
    }

    fn block(stmts: Vec<Statement>) -> Statement {
        Statement::Block {
            statements: stmts,
            scope: vec![],
        }
    }

    fn var(name: &str, ty: AstType) -> Variable {
        Variable {
            name: name.to_string(),
            ty: Some(ty),
        }
    }

    fn module(funcs: Vec<Function>, structs: Vec<StructDef>) -> Module {
        Module {
            func: funcs,
            structs,
            globals: vec![],
        }
    }

    fn func(name: &str, args: Vec<Variable>, ret: Option<AstType>, body: Statement) -> Function {
        Function {
            name: name.to_string(),
            arguments: args,
            ret_type: ret,
            body,
        }
    }

    // -------------------------------------------------------------------------
    // Function definitions
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_main() {
        let m = module(vec![func("main", vec![], None, block(vec![]))], vec![]);
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function main(){
}

main();"
        );
    }

    #[test]
    fn test_function_return_int() {
        let body = block(vec![Statement::Return(Some(Expression::Int(42)))]);
        let m = module(
            vec![
                func("answer", vec![], Some(AstType::Int), body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function answer(){
return 42;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_function_with_arguments() {
        let body = block(vec![Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Variable("a".to_string())),
            op: BinOp::Addition,
            rhs: Box::new(Expression::Variable("b".to_string())),
        }))]);
        let m = module(
            vec![
                func(
                    "add",
                    vec![var("a", AstType::Int), var("b", AstType::Int)],
                    Some(AstType::Int),
                    body,
                ),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function add(a, b){
return a + b;
}

function main(){
}

main();"
        );
    }

    // -------------------------------------------------------------------------
    // Arithmetic operations
    // -------------------------------------------------------------------------

    #[test]
    fn test_arithmetic_addition() {
        let body = block(vec![Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Int(3)),
            op: BinOp::Addition,
            rhs: Box::new(Expression::Int(4)),
        }))]);
        let m = module(
            vec![
                func("calc", vec![], Some(AstType::Int), body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function calc(){
return 3 + 4;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_arithmetic_subtraction() {
        let body = block(vec![Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Int(10)),
            op: BinOp::Subtraction,
            rhs: Box::new(Expression::Int(3)),
        }))]);
        let m = module(
            vec![
                func("calc", vec![], Some(AstType::Int), body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function calc(){
return 10 - 3;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_arithmetic_multiplication() {
        let body = block(vec![Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Int(4)),
            op: BinOp::Multiplication,
            rhs: Box::new(Expression::Int(5)),
        }))]);
        let m = module(
            vec![
                func("calc", vec![], Some(AstType::Int), body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function calc(){
return 4 * 5;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_arithmetic_division() {
        let body = block(vec![Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Int(8)),
            op: BinOp::Division,
            rhs: Box::new(Expression::Int(2)),
        }))]);
        let m = module(
            vec![
                func("calc", vec![], Some(AstType::Int), body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function calc(){
return 8 / 2;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_arithmetic_modulus() {
        let body = block(vec![Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Int(9)),
            op: BinOp::Modulus,
            rhs: Box::new(Expression::Int(4)),
        }))]);
        let m = module(
            vec![
                func("calc", vec![], Some(AstType::Int), body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function calc(){
return 9 % 4;
}

function main(){
}

main();"
        );
    }

    // -------------------------------------------------------------------------
    // Comparison operations
    // -------------------------------------------------------------------------

    #[test]
    fn test_comparison_equal() {
        let body = block(vec![Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Variable("a".to_string())),
            op: BinOp::Equal,
            rhs: Box::new(Expression::Variable("b".to_string())),
        }))]);
        let m = module(
            vec![
                func(
                    "cmp",
                    vec![var("a", AstType::Int), var("b", AstType::Int)],
                    Some(AstType::Bool),
                    body,
                ),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function cmp(a, b){
return a === b;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_comparison_not_equal() {
        let body = block(vec![Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Variable("a".to_string())),
            op: BinOp::NotEqual,
            rhs: Box::new(Expression::Variable("b".to_string())),
        }))]);
        let m = module(
            vec![
                func(
                    "cmp",
                    vec![var("a", AstType::Int), var("b", AstType::Int)],
                    Some(AstType::Bool),
                    body,
                ),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function cmp(a, b){
return a !== b;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_comparison_less_than() {
        let body = block(vec![Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Variable("a".to_string())),
            op: BinOp::LessThan,
            rhs: Box::new(Expression::Variable("b".to_string())),
        }))]);
        let m = module(
            vec![
                func(
                    "cmp",
                    vec![var("a", AstType::Int), var("b", AstType::Int)],
                    Some(AstType::Bool),
                    body,
                ),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function cmp(a, b){
return a < b;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_comparison_greater_than() {
        let body = block(vec![Statement::Return(Some(Expression::BinOp {
            lhs: Box::new(Expression::Variable("a".to_string())),
            op: BinOp::GreaterThan,
            rhs: Box::new(Expression::Variable("b".to_string())),
        }))]);
        let m = module(
            vec![
                func(
                    "cmp",
                    vec![var("a", AstType::Int), var("b", AstType::Int)],
                    Some(AstType::Bool),
                    body,
                ),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function cmp(a, b){
return a > b;
}

function main(){
}

main();"
        );
    }

    // -------------------------------------------------------------------------
    // Variable declaration and assignment
    // -------------------------------------------------------------------------

    #[test]
    fn test_variable_declaration_with_value() {
        let body = block(vec![
            Statement::Declare {
                variable: var("x", AstType::Int),
                value: Some(Expression::Int(7)),
            },
            Statement::Return(Some(Expression::Variable("x".to_string()))),
        ]);
        let m = module(
            vec![
                func("get_x", vec![], Some(AstType::Int), body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function get_x(){
var x = 7;
return x;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_variable_declaration_no_value() {
        let body = block(vec![Statement::Declare {
            variable: var("x", AstType::Int),
            value: None,
        }]);
        let m = module(
            vec![
                func("decl_only", vec![], None, body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function decl_only(){
var x;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_uninitialized_array_becomes_empty_array() {
        let body = block(vec![Statement::Declare {
            variable: var("buf", AstType::Array(Box::new(AstType::Int), None)),
            value: None,
        }]);
        let m = module(
            vec![
                func("f", vec![], None, body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function f(){
var buf = [];
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_variable_assignment() {
        let body = block(vec![
            Statement::Declare {
                variable: var("x", AstType::Int),
                value: Some(Expression::Int(1)),
            },
            Statement::Assign {
                lhs: Box::new(Expression::Variable("x".to_string())),
                rhs: Box::new(Expression::Int(99)),
            },
        ]);
        let m = module(
            vec![
                func("reassign", vec![], None, body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function reassign(){
var x = 1;
x = 99;
}

function main(){
}

main();"
        );
    }

    // -------------------------------------------------------------------------
    // Conditionals
    // -------------------------------------------------------------------------

    #[test]
    fn test_if_no_else() {
        let body = block(vec![Statement::If {
            condition: Expression::Bool(true),
            body: Box::new(block(vec![Statement::Return(Some(Expression::Int(1)))])),
            else_branch: None,
        }]);
        let m = module(
            vec![
                func("branching", vec![], Some(AstType::Int), body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function branching(){
if (true){
return 1;
};
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_if_with_else() {
        let body = block(vec![Statement::If {
            condition: Expression::Variable("flag".to_string()),
            body: Box::new(block(vec![Statement::Return(Some(Expression::Int(1)))])),
            else_branch: Some(Box::new(block(vec![Statement::Return(Some(
                Expression::Int(0),
            ))]))),
        }]);
        let m = module(
            vec![
                func(
                    "branch_else",
                    vec![var("flag", AstType::Bool)],
                    Some(AstType::Int),
                    body,
                ),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function branch_else(flag){
if (flag){
return 1;
}else {
return 0;
}
;
;
}

function main(){
}

main();"
        );
    }

    // -------------------------------------------------------------------------
    // Loops
    // -------------------------------------------------------------------------

    #[test]
    fn test_while_loop() {
        let body = block(vec![Statement::While {
            condition: Expression::Bool(true),
            body: Box::new(block(vec![Statement::Break])),
        }]);
        let m = module(
            vec![
                func("loop_fn", vec![], None, body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function loop_fn(){
while (true) {
break;
;
}
;
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_for_loop() {
        let body = block(vec![Statement::For {
            ident: var("item", AstType::Int),
            expr: Expression::Array {
                capacity: 3,
                elements: vec![Expression::Int(1), Expression::Int(2), Expression::Int(3)],
            },
            body: Box::new(block(vec![])),
        }]);
        let m = module(
            vec![
                func("for_fn", vec![], None, body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function for_fn(){
var loop_orig_item = [1, 2, 3];
for (let iter_item = 0; iter_item < loop_orig_item.length; iter_item++){
let item = loop_orig_item[iter_item];
}
;
}

function main(){
}

main();"
        );
    }

    // -------------------------------------------------------------------------
    // Arrays
    // -------------------------------------------------------------------------

    #[test]
    fn test_array_literal() {
        let body = block(vec![Statement::Declare {
            variable: var("nums", AstType::Array(Box::new(AstType::Int), None)),
            value: Some(Expression::Array {
                capacity: 3,
                elements: vec![Expression::Int(1), Expression::Int(2), Expression::Int(3)],
            }),
        }]);
        let m = module(
            vec![
                func("arr_fn", vec![], None, body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function arr_fn(){
var nums = [1, 2, 3];
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_array_access() {
        let body = block(vec![Statement::Return(Some(Expression::ArrayAccess {
            name: "arr".to_string(),
            index: Box::new(Expression::Int(0)),
        }))]);
        let m = module(
            vec![
                func(
                    "first",
                    vec![var("arr", AstType::Array(Box::new(AstType::Int), None))],
                    Some(AstType::Int),
                    body,
                ),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function first(arr){
return arr[0];
}

function main(){
}

main();"
        );
    }

    // -------------------------------------------------------------------------
    // Structs
    // -------------------------------------------------------------------------

    #[test]
    fn test_struct_definition() {
        let struct_def = StructDef {
            name: "Point".to_string(),
            fields: vec![var("x", AstType::Int), var("y", AstType::Int)],
            methods: vec![],
        };
        let m = module(vec![func("main", vec![], None, block(vec![]))], vec![struct_def]);
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function Point(args) {
this.x = args.x;
this.y = args.y;
}
function main(){
}

main();"
        );
    }

    #[test]
    fn test_struct_initialization() {
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Box::new(Expression::Int(3)));
        let body = block(vec![Statement::Declare {
            variable: var("p", AstType::Struct("Point".to_string())),
            value: Some(Expression::StructInitialization {
                name: "Point".to_string(),
                fields,
            }),
        }]);
        let m = module(
            vec![
                func("make_point", vec![], None, body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function make_point(){
var p = new Point({x: 3,});
}

function main(){
}

main();"
        );
    }

    // -------------------------------------------------------------------------
    // String and boolean literals
    // -------------------------------------------------------------------------

    #[test]
    fn test_string_return() {
        let body = block(vec![Statement::Return(Some(Expression::Str(
            "hello".to_string(),
        )))]);
        let m = module(
            vec![
                func("greet", vec![], Some(AstType::Str), body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function greet(){
return \"hello\";
}

function main(){
}

main();"
        );
    }

    #[test]
    fn test_boolean_return() {
        let body = block(vec![Statement::Return(Some(Expression::Bool(true)))]);
        let m = module(
            vec![
                func("get_true", vec![], Some(AstType::Bool), body),
                func("main", vec![], None, block(vec![])),
            ],
            vec![],
        );
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function get_true(){
return true;
}

function main(){
}

main();"
        );
    }

    // -------------------------------------------------------------------------
    // Function call as expression
    // -------------------------------------------------------------------------

    #[test]
    fn test_function_call_statement() {
        let body = block(vec![Statement::Exp(Expression::FunctionCall {
            fn_name: "print".to_string(),
            args: vec![Expression::Str("hi".to_string())],
        })]);
        let m = module(vec![func("main", vec![], None, body)], vec![]);
        let result = JsGenerator::generate(m).unwrap();
        assert_eq!(
            user_code(&result),
            "function main(){
print(\"hi\");
}

main();"
        );
    }

    // -------------------------------------------------------------------------
    // main(); is always the final token
    // -------------------------------------------------------------------------

    #[test]
    fn test_main_call_is_last() {
        let m = module(vec![func("main", vec![], None, block(vec![]))], vec![]);
        let result = JsGenerator::generate(m).unwrap();
        assert!(result.ends_with("main();"));
    }
}
