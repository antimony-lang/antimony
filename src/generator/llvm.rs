use crate::ast::types::Type;
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
use inkwell::context::Context;
use inkwell::module;
use inkwell::types::*;

pub struct LLVMGenerator<'ctx> {
    ctx: &'ctx Context,
    module: module::Module<'ctx>,
}

impl<'ctx> Generator for LLVMGenerator<'ctx> {
    fn generate(prog: Module) -> String {
        let ctx = Context::create();
        let module = ctx.create_module("main");
        let mut generator = LLVMGenerator { ctx: &ctx, module };
        for func in prog.func {
            generator.generate_function(func);
        }
        generator.module.print_to_string().to_string()
    }
}

impl<'ctx> LLVMGenerator<'ctx> {
    fn convert_to_llvm_args(&mut self, args: Vec<Variable>) -> Vec<BasicTypeEnum<'ctx>> {
        let arg_types: Vec<BasicTypeEnum> = args
            .iter()
            .map(|arg| match arg.ty {
                Some(Type::Int) => self.ctx.i32_type().as_basic_type_enum(),
                Some(Type::Bool) => self.ctx.bool_type().as_basic_type_enum(),
                Some(Type::Any) => todo!(),
                Some(Type::Str) => todo!(),
                Some(Type::Array(_)) => todo!(),
                Some(Type::Struct(_)) => todo!(),
                None => panic!("Function argument has no type"),
            })
            .collect();
        arg_types
    }

    fn generate_function(&mut self, func: Function) {
        let arg_types: Vec<BasicTypeEnum> = self.convert_to_llvm_args(func.arguments);

        let func_type = match func.ret_type {
            Some(Type::Int) => self.ctx.i32_type().fn_type(&arg_types, false),
            Some(Type::Bool) => self.ctx.bool_type().fn_type(&arg_types, false),
            None => self.ctx.void_type().fn_type(&arg_types, false),
            _ => todo!(),
        };
        let function = self.module.add_function(&func.name, func_type, None);
        let _basic_block = self.ctx.append_basic_block(function, "entry");
        self.generate_statement(func.body);
    }

    fn generate_statement(&mut self, statement: Statement) {
        match statement {
            Statement::Block(statements, _) => {
                for s in statements {
                    self.generate_statement(s);
                }
            }
            Statement::Exp(expression) => self.generate_expression(expression),
            _ => todo!(),
        };
    }

    fn generate_expression(&mut self, _expr: Expression) {
        todo!()
    }
}
