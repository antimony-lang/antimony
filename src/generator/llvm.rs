use crate::generator::Generator;
use crate::parser::node_type::*;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target};

pub struct LLVMGenerator<'ctx> {
    ctx: &'ctx Context,
    module: Module<'ctx>,
}

impl<'ctx> Generator for LLVMGenerator<'ctx> {
    fn generate(prog: Program) -> String {
        let ctx = Context::create();
        let module = ctx.create_module("main");
        let mut generator = LLVMGenerator {
            ctx: &ctx,
            module: module,
        };
        for func in prog.func {
            generator.generate_function(func);
        }
        generator.module.print_to_string().to_string()
    }
}

impl<'ctx> LLVMGenerator<'ctx> {
    fn generate_function(&mut self, func: Function) {
        self.module
            .add_function(&func.name, self.ctx.void_type().fn_type(&[], false), None);
    }
}
