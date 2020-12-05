use crate::generator::Generator;
use crate::parser::node_type::Function;
use crate::parser::node_type::Program;

pub struct X86Generator;

impl Generator for X86Generator {
    fn generate(prog: Program) -> String {
        return prog
            .func
            .into_iter()
            .map(|f| generate_function(f))
            .collect();
    }
}

fn generate_function(func: Function) -> String {
    format!(
        "
        .globl {F}
        {F}:
        ",
        F = func.name
    )
}
