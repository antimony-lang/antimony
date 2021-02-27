use super::Generator;
use crate::ast::types::Type;
use crate::ast::{Function, Module, Variable};

pub struct QBEGenerator {
    pub code: String,
}

impl Generator for QBEGenerator {
    fn generate(prog: Module) -> String {
        let mut gen = QBEGenerator::new();

        for func in prog.func.iter() {
            gen.add_function(func)
        }

        gen.code
    }
}

impl QBEGenerator {
    pub(crate) fn new() -> Self {
        Self {
            code: String::new(),
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
        arguments.
            into_iter().
            map(|arg| {
                // w %apple
                format!(
                    "{type} %{name}",
                    type = Self::generate_type(&arg.ty.as_ref().unwrap()),
                    name = arg.name,
                )
            }).
            collect::<Vec<String>>().
            join(", ")
    }

    /// Adds a block label to the generated code
    fn add_block_label(&mut self, name: &str) {
        self.code.push_str(&format!("@{}\n", name));
    }

    fn add_function(&mut self, func: &Function) {
        let return_type = Self::generate_type(&func.ret_type.as_ref().unwrap_or(&Type::Int));
        let params = Self::generate_function_params(&func.arguments);

        // XXX: Do we need to export all functions? Or when noted as so?
        // TODO: We might get a function collision. Or not?

        // export function w $myfunc(w %age) {
        self.code.push_str(&format!(
            "export function {return_type} ${name}({params}) {{\n",
            return_type = return_type,
            name = func.name,
            params = params,
        ));

        // @start
        self.add_block_label("start");

        // }
        self.code.push_str("}\n");
    }
}
