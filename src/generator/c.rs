use crate::ast::*;
use crate::generator::{Generator, GeneratorResult};
use std::collections::HashMap;
use types::Type;

pub struct CGenerator;

impl Generator for CGenerator {
    fn generate(prog: Module) -> GeneratorResult<String> {
        let mut code = String::new();

        // Add standard C headers
        code += "#include <stdio.h>\n";
        code += "#include <stdlib.h>\n";
        code += "#include <stdbool.h>\n";
        code += "#include <string.h>\n\n";

        // Add builtin functions
        let raw_builtins = crate::Builtins::get("builtin.c")
            .expect("Could not locate builtin functions")
            .data;
        code += std::str::from_utf8(&raw_builtins).expect("Unable to interpret builtin functions");

        // Generate struct definitions first
        let structs: String = prog
            .structs
            .clone()
            .into_iter()
            .map(generate_struct_definition)
            .collect();

        code += &structs;

        // Generate function prototypes
        let prototypes: String = prog.func.iter().map(generate_function_prototype).collect();

        code += &prototypes;
        code += "\n";

        // Generate function implementations
        let funcs: String = prog.func.into_iter().map(generate_function).collect();

        code += &funcs;

        Ok(code)
    }
}

pub(super) fn generate_arguments(args: Vec<Variable>) -> String {
    if args.is_empty() {
        return "void".to_string();
    }

    args.into_iter()
        .map(|var| format!("{} {}", type_to_c_type(&var.ty), var.name))
        .collect::<Vec<String>>()
        .join(", ")
}

fn type_to_c_type(ty: &Option<Type>) -> String {
    match ty {
        Some(Type::Int) => "int".to_string(),
        Some(Type::Bool) => "bool".to_string(),
        Some(Type::Str) => "char*".to_string(),
        Some(Type::Array(inner, _)) => format!("{}*", type_to_c_type(&Some(*inner.clone()))),
        Some(Type::Struct(name)) => name.clone(),
        Some(Type::Any) => "void*".to_string(),
        None => "void".to_string(),
    }
}

pub(super) fn generate_function_prototype(func: &Function) -> String {
    let return_type = match &func.ret_type {
        Some(ty) => type_to_c_type(&Some(ty.clone())),
        None => "void".to_string(),
    };

    format!(
        "{} {}({});\n",
        return_type,
        func.name,
        generate_arguments(func.arguments.clone())
    )
}

pub(super) fn generate_function(func: Function) -> String {
    let return_type = match &func.ret_type {
        Some(ty) => type_to_c_type(&Some(ty.clone())),
        None => "void".to_string(),
    };

    let arguments = generate_arguments(func.arguments);
    let mut raw = format!("{} {}({}) ", return_type, func.name, arguments);

    raw += &generate_block(func.body, None);
    raw += "\n";
    raw
}

pub(super) fn generate_struct_definition(struct_def: StructDef) -> String {
    let mut buf = format!("typedef struct {} {{\n", &struct_def.name);

    // Generate struct fields
    for field in &struct_def.fields {
        buf += &format!("    {} {};\n", type_to_c_type(&field.ty), field.name);
    }
    buf += &format!("}} {};\n\n", &struct_def.name);

    // Generate method prototypes
    for method in &struct_def.methods {
        let mut method_copy = method.clone();
        // Add self parameter as first argument
        let self_var = Variable {
            name: "self".to_string(),
            ty: Some(Type::Struct(struct_def.name.clone())),
        };
        method_copy.arguments.insert(0, self_var);
        buf += &generate_function_prototype(&method_copy);
    }

    // Generate method implementations
    for method in &struct_def.methods {
        let mut method_copy = method.clone();
        // Add self parameter as first argument
        let self_var = Variable {
            name: "self".to_string(),
            ty: Some(Type::Struct(struct_def.name.clone())),
        };
        method_copy.arguments.insert(0, self_var);
        buf += &generate_function(method_copy);
    }

    buf
}

pub(super) fn generate_block(block: Statement, prepend: Option<String>) -> String {
    let mut generated = String::from("{\n");

    if let Some(pre) = prepend {
        generated += &pre;
    }

    let statements = match block {
        Statement::Block {
            statements,
            scope: _,
        } => statements,
        _ => panic!("Block body should be of type Statement::Block"),
    };

    for statement in statements {
        generated += &generate_statement(statement);
    }

    generated += "}\n";
    generated
}

pub(super) fn generate_statement(statement: Statement) -> String {
    let state = match statement {
        Statement::Return(ret) => generate_return(ret),
        Statement::Declare { variable, value } => generate_declare(variable, value),
        Statement::Exp(val) => generate_expression(val),
        Statement::If {
            condition,
            body,
            else_branch,
        } => generate_conditional(condition, *body, else_branch.map(|x| *x)),
        Statement::Assign { lhs, rhs } => generate_assign(*lhs, *rhs),
        Statement::Block {
            statements: _,
            scope: _,
        } => return generate_block(statement, None),
        Statement::While { condition, body } => generate_while_loop(condition, *body),
        Statement::For { ident, expr, body } => generate_for_loop(ident, expr, *body),
        Statement::Continue => "continue".to_string(),
        Statement::Break => "break".to_string(),
    };

    format!("    {};\n", state)
}

pub(super) fn generate_expression(expr: Expression) -> String {
    match expr {
        Expression::Int(val) => val.to_string(),
        Expression::Selff => "self".to_string(),
        Expression::Str(val) => format!("\"{}\"", val.replace("\"", "\\\"")),
        Expression::Variable(val) => val,
        Expression::Bool(b) => if b { "true" } else { "false" }.to_string(),
        Expression::FunctionCall { fn_name, args } => generate_function_call(fn_name, args),
        Expression::Array {
            capacity: _,
            elements,
        } => generate_array(elements),
        Expression::ArrayAccess { name, index } => generate_array_access(name, *index),
        Expression::BinOp { lhs, op, rhs } => generate_bin_op(*lhs, op, *rhs),
        Expression::StructInitialization { name, fields } => {
            generate_struct_initialization(name, fields)
        }
        Expression::FieldAccess { expr, field } => generate_field_access(*expr, *field),
    }
}

pub(super) fn generate_while_loop(expr: Expression, body: Statement) -> String {
    format!(
        "while ({}) {}",
        generate_expression(expr),
        generate_block(body, None)
    )
}

pub(super) fn generate_for_loop(ident: Variable, expr: Expression, body: Statement) -> String {
    // C-style for loop with array indexing
    let mut out_str = format!(
        "for(int i = 0; i < sizeof({}) / sizeof({}[0]); i++)",
        generate_expression(expr.clone()),
        generate_expression(expr.clone())
    );

    // Add the loop variable declaration to the prepended block
    out_str += &generate_block(
        body,
        Some(format!(
            "    {} {} = {}[i];\n",
            type_to_c_type(&ident.ty),
            ident.name,
            generate_expression(expr)
        )),
    );
    out_str
}

pub(super) fn generate_array(elements: Vec<Expression>) -> String {
    let mut out_str = String::from("{");

    out_str += &elements
        .iter()
        .map(|el| generate_expression(el.clone()))
        .collect::<Vec<String>>()
        .join(", ");

    out_str += "}";
    out_str
}

pub(super) fn generate_array_access(name: String, expr: Expression) -> String {
    format!("{}[{}]", name, generate_expression(expr))
}

pub(super) fn generate_conditional(
    expr: Expression,
    if_state: Statement,
    else_state: Option<Statement>,
) -> String {
    let mut outcome = format!("if ({}) ", generate_expression(expr));
    outcome += &generate_block(if_state, None);

    if let Some(else_state) = else_state {
        outcome += "else ";
        outcome += &generate_block(else_state, None);
    }

    outcome
}

pub(super) fn generate_declare<V: AsRef<Variable>>(
    identifier: V,
    val: Option<Expression>,
) -> String {
    let ident = identifier.as_ref();
    let type_str = type_to_c_type(&ident.ty);

    match val {
        Some(expr) => format!(
            "{} {} = {}",
            type_str,
            ident.name,
            generate_expression(expr)
        ),
        None => match &ident.ty {
            Some(Type::Array(_, _)) => {
                format!("{} {}[]", type_str, ident.name)
            }
            _ => format!("{} {}", type_str, ident.name),
        },
    }
}

pub(super) fn generate_function_call(func: String, args: Vec<Expression>) -> String {
    let formatted_args = args
        .into_iter()
        .map(generate_expression)
        .collect::<Vec<String>>()
        .join(", ");

    format!("{}({})", func, formatted_args)
}

pub(super) fn generate_return(ret: Option<Expression>) -> String {
    match ret {
        Some(expr) => format!("return {}", generate_expression(expr)),
        None => "return".to_string(),
    }
}

pub(super) fn generate_bin_op(left: Expression, op: BinOp, right: Expression) -> String {
    let op_str = match op {
        BinOp::Addition => "+",
        BinOp::And => "&&",
        BinOp::Division => "/",
        BinOp::Equal => "==",
        BinOp::GreaterThan => ">",
        BinOp::GreaterThanOrEqual => ">=",
        BinOp::LessThan => "<",
        BinOp::LessThanOrEqual => "<=",
        BinOp::Modulus => "%",
        BinOp::Multiplication => "*",
        BinOp::NotEqual => "!=",
        BinOp::Or => "||",
        BinOp::Subtraction => "-",
        BinOp::AddAssign => "+=",
        BinOp::SubtractAssign => "-=",
        BinOp::MultiplyAssign => "*=",
        BinOp::DivideAssign => "/=",
    };

    format!(
        "{} {} {}",
        generate_expression(left),
        op_str,
        generate_expression(right)
    )
}

pub(super) fn generate_struct_initialization(
    name: String,
    fields: HashMap<String, Box<Expression>>,
) -> String {
    let mut out_str = format!("({}) {{", name);

    let field_inits: Vec<String> = fields
        .into_iter()
        .map(|(key, value)| format!(".{} = {}", key, generate_expression(*value)))
        .collect();

    out_str += &field_inits.join(", ");
    out_str += "}";

    out_str
}

pub(super) fn generate_field_access(expr: Expression, field: Expression) -> String {
    // In C, we use -> for pointer access and . for direct access
    // For simplicity, we'll use . here, but in a real implementation
    // you'd need to check if expr is a pointer
    format!(
        "{}.{}",
        generate_expression(expr),
        generate_expression(field)
    )
}

pub(super) fn generate_assign(name: Expression, expr: Expression) -> String {
    format!(
        "{} = {}",
        generate_expression(name),
        generate_expression(expr)
    )
}
