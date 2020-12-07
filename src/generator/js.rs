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

        // Until we have a stdlib, it should suffice to print the result of main to stdout
        code += "console.log(main())";

        return code;
    }
}

fn generate_function(func: Function) -> String {
    let arguments: String = func
        .arguments
        .into_iter()
        .map(|var| var.name)
        .collect::<Vec<String>>()
        .join(", ");
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
        Statement::If(expr, if_state, else_state) => {
            generate_conditional(expr, *if_state, else_state.map(|x| *x))
        }
        Statement::While(_, _) => todo!(),
    }
}

fn generate_expression(expr: Expression) -> String {
    match expr {
        Expression::Int(val) => val.to_string(),
        Expression::Variable(val) | Expression::Str(val) => val,
        Expression::Char(_) => todo!(),
        Expression::FunctionCall(name, e) => generate_function_call(name, e),
        Expression::Assign(_, _) => todo!(),
        Expression::BinOp(left, op, right) => generate_bin_op(*left, op, *right),
    }
}

fn generate_conditional(
    expr: Expression,
    if_state: Statement,
    else_state: Option<Statement>,
) -> String {
    let expr_str = generate_expression(expr);
    let if_str = generate_statement(if_state);

    let mut outcome = format!("if ({})", expr_str);

    outcome += "{\n";
    outcome += &if_str;
    outcome += "}";
    outcome
}

fn generate_function_call(func: String, args: Vec<Expression>) -> String {
    let formatted_args = args
        .into_iter()
        .map(|arg| match arg {
            Expression::Char(c) => c.to_string(),
            Expression::Int(i) => i.to_string(),
            Expression::FunctionCall(n, a) => generate_function_call(n, a),
            Expression::Str(s) | Expression::Variable(s) => s,
            Expression::Assign(_, _) => todo!(),
            Expression::BinOp(left, op, right) => generate_bin_op(*left, op, *right),
        })
        .collect::<Vec<String>>()
        .join(",");
    format!("{N}({A})\n", N = func, A = formatted_args)
}

fn generate_return(ret: Option<Expression>) -> String {
    match ret {
        Some(expr) => format!("return {}\n", generate_expression(expr)),
        None => "return;\n".to_string(),
    }
}

fn generate_bin_op(left: Expression, op: BinOp, right: Expression) -> String {
    let op_str = match op {
        BinOp::Addition => "+",
        BinOp::And => "&&",
        BinOp::Division => "/",
        BinOp::Equal => "===",
        BinOp::GreaterThan => ">",
        BinOp::GreaterThanOrEqual => ">=",
        BinOp::LessThan => "<",
        BinOp::LessThanOrEqual => "<=",
        BinOp::Modulus => "%",
        BinOp::Multiplication => "*",
        BinOp::NotEqual => "!==",
        BinOp::Or => "||",
        BinOp::Subtraction => "-",
    };
    format!(
        "{l} {op} {r}",
        l = generate_expression(left),
        op = op_str,
        r = generate_expression(right)
    )
}
