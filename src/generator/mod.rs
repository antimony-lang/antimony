use crate::parser::node_type::Program;

pub mod x86;

pub trait Generator {
    fn generate(prog: Program) -> String;
}
