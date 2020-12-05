use crate::generator::Generator;
use crate::parser::node_type::*;

pub struct JsGenerator;

impl Generator for JsGenerator {
    fn generate(prog: Program) -> String {
        let mut code = prog
            .func
            .into_iter()
            .map(|f| generate_function(f))
            .collect();

        code += "main()";

        return code;
    }
}

fn generate_function(func: Function) -> String {
    let arguments: String = func
        .arguments
        .into_iter()
        .map(|arg: Variable| format!("{},", arg.name))
        .collect();
    let mut raw = format!("function {N}({A})", N = func.name, A = arguments);

    raw += " {\n";

    for statement in func.statements {
        raw += &generate_statement(statement);
    }

    raw += "}\n";

    raw
}

fn generate_statement(statement: Statement) -> String {
    match statement {
        Statement::Return(ret) => generate_return(ret),
        Statement::Declare(_, _) => todo!(),
        Statement::Exp(val) => generate_expression(val),
        Statement::Compound(_) => todo!(),
        Statement::If(_, _, _) => todo!(),
        Statement::While(_, _) => todo!(),
    }
}

fn generate_expression(expr: Expression) -> String {
    match expr {
        Expression::Int(val) => val.to_string(),
        Expression::Variable(val) | Expression::Str(val) => val,
        Expression::Char(_) => todo!(),
        Expression::FunctionCall(_, _) => todo!(),
        Expression::Assign(_, _) => todo!(),
    }
}

fn generate_return(ret: Option<Expression>) -> String {
    match ret {
        Some(expr) => format!("return {}\n", generate_expression(expr)),
        None => "return;\n".to_string(),
    }
}
