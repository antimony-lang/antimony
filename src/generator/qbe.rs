use super::Generator;
use crate::ast::{
    Module,
    Variable,
    Function,
};
use crate::ast::types::Type;

pub struct QBEGenerator;

impl Generator for QBEGenerator {
    fn generate(prog: Module) -> String {
        let mut code = String::new();

        for func in prog.func.iter() {
            code.push_str(&generate_function(func))
        }

        code
    }
}

fn generate_type(ty: &Type) -> String {
    match ty {
        // TODO: differentiate between 32- and 64-bit values
        Type::Int => "w".into(),
        _ => todo!(),
    }
}

fn generate_function_params(arguments: &Vec<Variable>) -> String {
    let mut buf = String::new();

    let len = arguments.len();
    for (i, arg) in arguments.into_iter().enumerate() {
        buf.push_str(
            &format!(
                "{type} %{ident}",
                // Types for parameters are required
                type = generate_type(arg.ty.as_ref().unwrap()),
                ident = arg.name,
            )
        );

        if i < len - 1 {
            buf.push_str(", ");
        }
    }

    buf
}

fn generate_function(func: &Function) -> String {
    let return_type = generate_type(&func.ret_type.as_ref().unwrap_or(&Type::Int));
    let params = generate_function_params(&func.arguments);

    // XXX: Do we need to export all functions? Or when noted as so?
    // TODO: We might get a function collision. Or not?
    format!("export function {return_type} ${name}({params}) {{
@start
{instructions}
}}
",
    return_type = return_type,
    name = func.name,
    params = params,
    instructions = ""
    )
}
