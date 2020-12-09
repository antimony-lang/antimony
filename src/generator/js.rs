/**
 * Copyright 2020 Garrit Franke
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
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

    raw += &generate_block(func.body);

    raw
}

fn generate_block(block: Statement) -> String {
    let mut generated = String::from("{\n");

    let statements = match block {
        Statement::Block(blk) => blk,
        _ => panic!("Block body should be of type Statement::Block"),
    };

    for statement in statements {
        generated += &generate_statement(statement);
    }

    generated += "}\n";

    generated
}

fn generate_statement(statement: Statement) -> String {
    match statement {
        Statement::Return(ret) => generate_return(ret),
        Statement::Declare(name, val) => generate_declare(name.name, val),
        Statement::Exp(val) => generate_expression(val),
        Statement::If(expr, if_state, else_state) => {
            generate_conditional(expr, *if_state, else_state.map(|x| *x))
        }
        Statement::Block(_) => generate_block(statement),
        Statement::While(expr, body) => generate_while_loop(expr, *body),
    }
}

fn generate_expression(expr: Expression) -> String {
    match expr {
        Expression::Int(val) => val.to_string(),
        Expression::Variable(val) | Expression::Str(val) => val,
        Expression::Char(_) => todo!(),
        Expression::Bool(b) => b.to_string(),
        Expression::FunctionCall(name, e) => generate_function_call(name, e),
        Expression::Assign(_, _) => todo!(),
        Expression::Array(els) => generate_array(els),
        Expression::BinOp(left, op, right) => generate_bin_op(*left, op, *right),
    }
}

fn generate_while_loop(expr: Expression, body: Statement) -> String {
    let mut out_str = String::from("while (");

    out_str += &generate_expression(expr);
    out_str += ") ";
    out_str += &generate_block(body);
    out_str
}

fn generate_array(elements: Vec<Expression>) -> String {
    let mut out_str = String::from("[");

    out_str += &elements
        .iter()
        .map(|el| match el {
            Expression::Int(x) => x.to_string(),
            Expression::Str(x) => x.to_string(),
            _ => todo!("Not yet implemented"),
        })
        .collect::<Vec<String>>()
        .join(", ");

    out_str += "]";
    out_str
}

fn generate_conditional(
    expr: Expression,
    if_state: Statement,
    else_state: Option<Statement>,
) -> String {
    let expr_str = generate_expression(expr);

    let body = match if_state {
        Statement::Block(blk) => blk,
        _ => panic!("Conditional body should be of type block"),
    };

    let mut outcome = format!("if ({})", expr_str);

    outcome += "{\n";
    for statement in body {
        outcome += &generate_statement(statement);
    }
    outcome += "}";

    if let Some(else_state) = else_state {
        outcome += "else ";
        outcome += &generate_statement(else_state);
    }
    outcome
}

fn generate_declare(name: String, val: Option<Expression>) -> String {
    // var is used here to not collide with scopes.
    // TODO: Can let be used instead?
    match val {
        Some(expr) => format!("var {} = {};\n", name, generate_expression(expr)),
        None => format!("var {};\n", name),
    }
}

fn generate_function_call(func: String, args: Vec<Expression>) -> String {
    let formatted_args = args
        .into_iter()
        .map(|arg| match arg {
            Expression::Char(c) => c.to_string(),
            Expression::Int(i) => i.to_string(),
            Expression::Bool(v) => v.to_string(),
            Expression::FunctionCall(n, a) => generate_function_call(n, a),
            Expression::Str(s) | Expression::Variable(s) => s,
            Expression::Assign(_, _) => todo!(),
            Expression::Array(_) => todo!(),
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
