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
use crate::ast::types::Type;
use crate::ast::*;
use crate::generator::{Generator, GeneratorResult};
use crate::util::Either;
use std::collections::HashMap;

pub struct CGenerator;

impl Generator for CGenerator {
    fn generate(prog: Module) -> GeneratorResult<String> {
        let mut code = String::new();

        let raw_builtins =
            crate::Builtins::get("builtin.c").expect("Could not locate builtin functions");
        code += std::str::from_utf8(raw_builtins.as_ref())
            .expect("Unable to interpret builtin functions");

        let structs: String = prog.structs.into_iter().map(generate_struct).collect();

        code += &structs;

        for func in &prog.func {
            code += &format!("{};\n", &generate_function_signature(func.clone()));
        }

        let funcs: String = prog.func.into_iter().map(generate_function).collect();

        code += &funcs;

        Ok(code)
    }
}

pub fn generate_struct(def: StructDef) -> String {
    // struct name {
    let mut buf = format!("struct {} {{\n", def.name);

    def.fields.iter().for_each(|f| {
        // int counter;
        buf += &format!("{} {};\n", generate_type(Either::Left(f.clone())), f.name,);
    });

    // };
    buf += "};\n";

    buf
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
            Type::Any => "void *".into(),
            Type::Bool => "bool".into(),
            Type::Struct(name) => format!("struct {}", name),
            Type::Array(t, capacity) => match name {
                Some(n) => format!(
                    "{T} {N}[{C}]",
                    T = generate_type(Either::Right(Some(*t))),
                    N = n,
                    C = capacity
                        .map(|val| val.to_string())
                        .unwrap_or_else(|| "".to_string()),
                ),
                None => format!("{}[]", generate_type(Either::Right(Some(*t)))),
            },
        },
        None => "void".into(),
    }
}

fn generate_function(func: Function) -> String {
    let mut buf = String::new();
    buf += &format!("{} ", &generate_function_signature(func.clone()));
    if let Statement::Block { statements, scope } = func.body {
        buf += &generate_block(statements, scope);
    }

    buf
}

fn generate_function_signature(func: Function) -> String {
    let arguments: String = func
        .arguments
        .into_iter()
        .map(|var| format!("{} {}", generate_type(Either::Left(var.clone())), var.name))
        .collect::<Vec<String>>()
        .join(", ");
    let t = generate_type(Either::Right(func.ret_type));
    format!("{T} {N}({A})", T = t, N = func.name, A = arguments)
}

fn generate_block(block: Vec<Statement>, _scope: Vec<Variable>) -> String {
    let mut generated = String::from("{\n");

    for statement in block {
        generated += &generate_statement(statement);
    }

    generated += "}\n";

    generated
}

fn generate_statement(statement: Statement) -> String {
    let state = match statement {
        Statement::Return(ret) => generate_return(ret),
        Statement::Declare { variable, value } => generate_declare(variable, value),
        Statement::Exp(val) => generate_expression(val) + ";\n",
        Statement::If {
            condition,
            body,
            else_branch,
        } => generate_conditional(condition, *body, else_branch.map(|x| *x)),
        Statement::Assign { lhs, rhs } => generate_assign(*lhs, *rhs),
        Statement::Block { statements, scope } => generate_block(statements, scope),
        Statement::While { condition, body } => generate_while_loop(condition, *body),
        Statement::For {
            ident: _,
            expr: _,
            body: _,
        } => todo!(),
        Statement::Continue => todo!(),
        Statement::Break => todo!(),
        Statement::Match {
            subject: _,
            arms: _,
        } => todo!(),
    };

    format!("{}\n", state)
}

fn generate_expression(expr: Expression) -> String {
    match expr {
        Expression::Int(val) => val.to_string(),
        Expression::Variable(val) => val,
        Expression::Str(val) => super::string_syntax(val),
        Expression::Bool(b) => b.to_string(),
        Expression::FunctionCall { fn_name, args } => generate_function_call(fn_name, args),
        Expression::Array { capacity, elements } => generate_array(capacity, elements),
        Expression::ArrayAccess { name, index } => generate_array_access(name, *index),
        Expression::BinOp { lhs, op, rhs } => generate_bin_op(*lhs, op, *rhs),
        Expression::StructInitialization(_, fields) => generate_struct_initialization(fields),
        Expression::FieldAccess(expr, field) => generate_field_access(*expr, *field),
        Expression::Selff => todo!(),
    }
}

fn generate_while_loop(expr: Expression, body: Statement) -> String {
    let mut out_str = String::from("while (");

    out_str += &generate_expression(expr);
    out_str += ") ";

    if let Statement::Block { statements, scope } = body {
        out_str += &generate_block(statements, scope);
    }
    out_str
}

fn generate_array(_size: usize, elements: Vec<Expression>) -> String {
    let mut out_str = String::from("[");

    out_str += &elements
        .iter()
        .map(|el| match el {
            Expression::Int(i) => i.to_string(),
            Expression::Str(s) => super::string_syntax(s.to_owned()),
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
        Statement::Block {
            statements,
            scope: _,
        } => statements,
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
            var.name,
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
            Expression::ArrayAccess { name, index } => generate_array_access(name, *index),
            Expression::FunctionCall { fn_name, args } => generate_function_call(fn_name, args),
            Expression::Str(s) => super::string_syntax(s),
            Expression::Variable(s) => s,
            Expression::Array {
                capacity: _,
                elements: _,
            } => todo!(),
            Expression::BinOp { lhs, op, rhs } => generate_bin_op(*lhs, op, *rhs),
            Expression::StructInitialization(_, fields) => generate_struct_initialization(fields),
            Expression::FieldAccess(expr, field) => generate_field_access(*expr, *field),
            Expression::Selff => todo!(),
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
        BinOp::Equal => "==",
        BinOp::GreaterThan => ">",
        BinOp::GreaterThanOrEqual => ">=",
        BinOp::LessThan => "<",
        BinOp::LessThanOrEqual => "<=",
        BinOp::AddAssign => "+=",
        BinOp::SubtractAssign => "-=",
        BinOp::MultiplyAssign => "*=",
        BinOp::DivideAssign => "/=",
        BinOp::Modulus => "%",
        BinOp::Multiplication => "*",
        BinOp::NotEqual => "!=",
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

fn generate_struct_initialization(fields: HashMap<String, Box<Expression>>) -> String {
    let mut buf: String = String::from("{");

    fields.iter().for_each(|(k, v)| {
        buf += &format!(".{} = {},", k, generate_expression(*v.clone()));
    });

    buf += "}";

    buf
}

fn generate_field_access(expr: Expression, field: Expression) -> String {
    format!(
        "{}.{}",
        generate_expression(expr),
        generate_expression(field)
    )
}

fn generate_assign(name: Expression, expr: Expression) -> String {
    format!(
        "{} = {};",
        generate_expression(name),
        generate_expression(expr)
    )
}
