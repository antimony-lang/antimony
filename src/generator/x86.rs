use crate::generator::Generator;
use crate::parser::node_type::Program;

pub struct X86Generator;

impl Generator for X86Generator {
    fn generate(prog: Program) -> String {
        return prog.func.into_iter().map(|f| format!("{:#?}", f)).collect();
    }
}
