use crate::generator::Generator;
use crate::parser::node_type::*;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::types::*;

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
        let arg_types: Vec<BasicTypeEnum> = func
            .arguments
            .iter()
            .map(|arg| match arg.ty {
                Some(Type::Int) => self.ctx.i32_type().as_basic_type_enum(),
                Some(Type::Bool) => self.ctx.bool_type().as_basic_type_enum(),
                Some(Type::Any) => todo!(),
                Some(Type::Str) => todo!(),
                Some(Type::Array(_)) => todo!(),
                None => panic!("Function argument has no type"),
            })
            .collect();

        let func_type = match func.ret_type {
            Some(Type::Int) => self.ctx.i32_type().fn_type(&arg_types, false),
            Some(Type::Bool) => self.ctx.bool_type().fn_type(&arg_types, false),
            None => self.ctx.void_type().fn_type(&arg_types, false),
            _ => todo!(),
        };
        self.module.add_function(&func.name, func_type, None);
    }
}
