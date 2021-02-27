use crate::ast::*;
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
use std::collections::HashMap;

pub struct JsGenerator;

impl Generator for JsGenerator {
    fn generate(prog: Module) -> String {
        let mut code = String::new();

        let raw_builtins =
            crate::Builtins::get("builtin.js").expect("Could not locate builtin functions");
        code += std::str::from_utf8(raw_builtins.as_ref())
            .expect("Unable to interpret builtin functions");

        let structs: String = prog
            .structs
            .clone()
            .into_iter()
            .map(|struct_def| {
                let methods: Vec<Function> = prog
                    .func
                    .iter()
                    .filter(|f| {
                        if let FunctionType::Method(subject) = &f.function_type {
                            *subject == struct_def.name
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();

                generate_struct_definition(struct_def, methods)
            })
            .collect();

        code += &structs;

        let funcs: String = prog
            .func
            .into_iter()
            .filter(|f| matches!(f.function_type, FunctionType::Function))
            .map(generate_function)
            .collect();

        code += &funcs;

        code += "main();";

        code
    }
}

fn generate_arguments(args: Vec<Variable>) -> String {
    args.into_iter()
        .map(|var| var.name)
        .collect::<Vec<String>>()
        .join(", ")
}

fn generate_function(func: Function) -> String {
    let arguments: String = generate_arguments(func.arguments);

    let mut raw = match func.function_type {
        FunctionType::Function => format!("function {N}({A})", N = func.name, A = arguments),
        FunctionType::Method(subject) => format!(
            "{}.prototype.{} = function({}) ",
            subject, func.name, arguments
        ),
    };

    raw += &generate_block(func.body, None);
    raw += "\n";
    raw
}

fn generate_struct_definition(struct_def: StructDef, methods: Vec<Function>) -> String {
    // JS doesn't care about field declaration

    // Constructor signature
    let mut buf = format!("function {}(args) {{\n", struct_def.name);

    // Field constructor fields
    for field in struct_def.fields {
        buf += &format!("this.{N} = args.{N};\n", N = field.name);
    }
    // Constructor end
    buf += "}\n";

    // Methods
    for method in methods {
        buf += &generate_function(method.to_owned());
    }

    buf
}

/// prepend is used to pass optional statements, that will be put in front of the regular block
/// Currently used in for statements, to declare local variables
fn generate_block(block: Statement, prepend: Option<String>) -> String {
    let mut generated = String::from("{\n");

    if let Some(pre) = prepend {
        generated += &pre;
    }

    // TODO: Prepend statements
    let statements = match block {
        Statement::Block(blk, _) => blk,
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
        Statement::Declare(name, val) => generate_declare(name.name, val),
        Statement::Exp(val) => generate_expression(val),
        Statement::If(expr, if_state, else_state) => {
            generate_conditional(expr, *if_state, else_state.map(|x| *x))
        }
        Statement::Assign(name, state) => generate_assign(*name, *state),
        Statement::Block(_, _) => generate_block(statement, None),
        Statement::While(expr, body) => generate_while_loop(expr, *body),
        Statement::For(ident, expr, body) => generate_for_loop(ident, expr, *body),
        Statement::Continue => generate_continue(),
        Statement::Break => generate_break(),
        Statement::Match(subject, arms) => generate_match(subject, arms),
    };

    format!("{};\n", state)
}

fn generate_expression(expr: Expression) -> String {
    match expr {
        Expression::Int(val) => val.to_string(),
        Expression::Selff => "this".to_string(),
        Expression::Variable(val) | Expression::Str(val) => val,
        Expression::Bool(b) => b.to_string(),
        Expression::FunctionCall(name, e) => generate_function_call(name, e),
        Expression::Array(els) => generate_array(els),
        Expression::ArrayAccess(name, expr) => generate_array_access(name, *expr),
        Expression::BinOp(left, op, right) => generate_bin_op(*left, op, *right),
        Expression::StructInitialization(name, fields) => {
            generate_struct_initialization(name, fields)
        }
        Expression::FieldAccess(expr, field) => generate_field_access(*expr, *field),
    }
}

fn generate_while_loop(expr: Expression, body: Statement) -> String {
    let mut out_str = String::from("while (");

    out_str += &generate_expression(expr);
    out_str += ") ";
    out_str += &generate_block(body, None);
    out_str
}

fn generate_for_loop(ident: Variable, expr: Expression, body: Statement) -> String {
    // Assign expression to variable to access it from within the loop
    let expr_name = format!("loop_orig_{}", ident.name);
    let mut out_str = format!("{};\n", generate_declare(expr_name.clone(), Some(expr)));

    // Loop signature
    out_str += &format!(
        "for (let iter_{I} = 0; iter_{I} < {E}.length; iter_{I}++)",
        I = ident.name,
        E = expr_name
    );

    // Block with prepended declaration of the actual variable
    out_str += &generate_block(
        body,
        Some(format!(
            "let {I} = {E}[iter_{I}];\n",
            I = ident.name,
            E = expr_name
        )),
    );
    out_str
}

fn generate_break() -> String {
    "break;\n".into()
}

fn generate_continue() -> String {
    "continue;\n".into()
}

fn generate_match(subject: Expression, arms: Vec<MatchArm>) -> String {
    let mut out_str = format!("switch ({E}) {{\n", E = generate_expression(subject));
    for arm in arms {
        match arm {
            MatchArm::Case(expr, statement) => {
                out_str += &format!("case {}:\n", generate_expression(expr));
                out_str += &format!("{}\n", &generate_statement(statement));
                out_str += "break;";
            }
            MatchArm::Else(statement) => {
                out_str += "default:\n";
                out_str += &format!("{}\n", &generate_statement(statement));
            }
        }
    }

    out_str += "}";

    out_str
}

fn generate_array(elements: Vec<Expression>) -> String {
    let mut out_str = String::from("[");

    out_str += &elements
        .iter()
        .map(|el| generate_expression(el.clone()))
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
        Statement::Block(blk, _) => blk,
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
        Some(expr) => format!("var {} = {}", name, generate_expression(expr)),
        None => format!("var {}", name),
    }
}

fn generate_function_call(func: String, args: Vec<Expression>) -> String {
    let formatted_args = args
        .into_iter()
        .map(|arg| match arg {
            Expression::Int(i) => i.to_string(),
            Expression::Bool(v) => v.to_string(),
            Expression::Selff => "this".to_string(),
            Expression::ArrayAccess(name, expr) => generate_array_access(name, *expr),
            Expression::FunctionCall(n, a) => generate_function_call(n, a),
            Expression::Str(s) | Expression::Variable(s) => s,
            Expression::Array(elements) => generate_array(elements),
            Expression::BinOp(left, op, right) => generate_bin_op(*left, op, *right),
            Expression::StructInitialization(name, fields) => {
                generate_struct_initialization(name, fields)
            }
            Expression::FieldAccess(expr, field) => generate_field_access(*expr, *field),
        })
        .collect::<Vec<String>>()
        .join(",");
    format!("{N}({A})", N = func, A = formatted_args)
}

fn generate_return(ret: Option<Expression>) -> String {
    match ret {
        Some(expr) => format!("return {}", generate_expression(expr)),
        None => "return".to_string(),
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
        BinOp::AddAssign => "+=",
        BinOp::SubtractAssign => "-=",
        BinOp::MultiplyAssign => "*=",
        BinOp::DivideAssign => "/=",
    };
    format!(
        "{l} {op} {r}",
        l = generate_expression(left),
        op = op_str,
        r = generate_expression(right)
    )
}

fn generate_struct_initialization(
    name: String,
    fields: HashMap<String, Box<Expression>>,
) -> String {
    let mut out_str = format!("new {}({{", name);
    for (key, value) in fields {
        out_str += &format!("{}: {},", key, generate_expression(*value));
    }

    out_str += "})";

    out_str
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
        "{} = {}",
        generate_expression(name),
        generate_expression(expr)
    )
}
