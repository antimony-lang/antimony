use crate::generator::Generator;
use crate::parser::node_type::*;
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
use crate::util::Either;

pub struct CGenerator;

impl Generator for CGenerator {
    fn generate(prog: Program) -> String {
        let mut code = String::new();

        let raw_builtins =
            crate::Builtins::get("builtin.c").expect("Could not locate builtin functions");
        code += std::str::from_utf8(raw_builtins.as_ref())
            .expect("Unable to interpret builtin functions");
        let funcs: String = prog
            .func
            .into_iter()
            .map(|f| generate_function(f))
            .collect();

        code += &funcs;

        return code;
    }
}

pub(super) fn generate_type(t: Either<Variable, Option<Type>>) -> String {
    let (ty, name) = match t {
        Either::Left(var) => (var.ty, Some(var.name)),
        Either::Right(ty) => (ty, None),
    };
    match ty {
        Some(t) => match t {
            Type::Int => "int".into(),
            Type::Str => "char *".into(),
            Type::Bool => "bool".into(),
            Type::Array(_) => match name {
                Some(n) => format!(
                    "{T} {N}[]",
                    T = generate_type(Either::Right(Some(t))),
                    N = n
                ),
                None => format!("{}[]", generate_type(Either::Right(Some(t)))),
            },
        },
        None => "void".into(),
    }
}

fn generate_function(func: Function) -> String {
    let arguments: String = func
        .arguments
        .into_iter()
        .map(|var| format!("{} {}", generate_type(Either::Left(var.clone())), var.name))
        .collect::<Vec<String>>()
        .join(", ");
    let t = generate_type(Either::Right(func.ret_type));
    let mut raw = format!("{T} {N}({A})", T = t, N = func.name, A = arguments);

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
    let state = match statement {
        Statement::Return(ret) => generate_return(ret),
        Statement::Declare(var, val) => generate_declare(var, val),
        Statement::Exp(val) => generate_expression(val) + ";\n",
        Statement::If(expr, if_state, else_state) => {
            generate_conditional(expr, *if_state, else_state.map(|x| *x))
        }
        Statement::Assign(name, state) => generate_assign(*name, *state),
        Statement::Block(_) => generate_block(statement),
        Statement::While(expr, body) => generate_while_loop(expr, *body),
    };

    format!("{}\n", state)
}

fn generate_expression(expr: Expression) -> String {
    let st = match expr {
        Expression::Int(val) => val.to_string(),
        Expression::Variable(val) | Expression::Str(val) => val,
        Expression::Bool(b) => b.to_string(),
        Expression::FunctionCall(name, e) => generate_function_call(name, e),
        Expression::Array(els) => generate_array(els),
        Expression::ArrayAccess(name, expr) => generate_array_access(name, *expr),
        Expression::BinOp(left, op, right) => generate_bin_op(*left, op, *right),
    };

    format!("{}", st)
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

fn generate_array_access(name: String, expr: Expression) -> String {
    format!("{n}[{e}]", n = name, e = generate_expression(expr))
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

fn generate_declare(var: Variable, val: Option<Expression>) -> String {
    // var is used here to not collide with scopes.
    // TODO: Can let be used instead?
    match val {
        Some(expr) => format!(
            "{} {} = {};",
            generate_type(Either::Left(var.to_owned())),
            var.to_owned().name,
            generate_expression(expr)
        ),
        None => format!(
            "{} {};",
            generate_type(Either::Left(var.to_owned())),
            var.name
        ),
    }
}

fn generate_function_call(func: String, args: Vec<Expression>) -> String {
    let formatted_args = args
        .into_iter()
        .map(|arg| match arg {
            Expression::Int(i) => i.to_string(),
            Expression::Bool(v) => v.to_string(),
            Expression::ArrayAccess(name, expr) => generate_array_access(name, *expr),
            Expression::FunctionCall(n, a) => generate_function_call(n, a),
            Expression::Str(s) | Expression::Variable(s) => s,
            Expression::Array(_) => todo!(),
            Expression::BinOp(left, op, right) => generate_bin_op(*left, op, *right),
        })
        .collect::<Vec<String>>()
        .join(",");
    format!("{N}({A})", N = func, A = formatted_args)
}

fn generate_return(ret: Option<Expression>) -> String {
    match ret {
        Some(expr) => format!("return {};", generate_expression(expr)),
        None => "return;".to_string(),
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

fn generate_assign(name: Expression, expr: Expression) -> String {
    format!(
        "{} = {};",
        generate_expression(name),
        generate_expression(expr)
    )
}
