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
use crate::parser::node_type::{Function, Module, Statement};

struct Assembly {
    asm: Vec<String>,
}

// We don't need "From", so we can ignore the lint here
#[allow(clippy::from_over_into)]
impl Into<String> for Assembly {
    fn into(self) -> String {
        self.build()
    }
}

impl Assembly {
    fn new() -> Assembly {
        Assembly { asm: vec![] }
    }

    fn add<S: Into<String>>(&mut self, string: S) {
        self.asm.push(string.into())
    }

    fn build(&self) -> String {
        self.asm.join("\n")
    }
}

pub struct X86Generator;

impl Generator for X86Generator {
    fn generate(prog: Module) -> String {
        Self::new().gen_program(prog).build()
    }
}

impl X86Generator {
    fn new() -> Self {
        X86Generator {}
    }

    fn gen_program(&mut self, prog: Module) -> Assembly {
        let mut asm = Assembly::new();
        let Module {
            func,
            globals,
            structs: _,
            path: _,
            imports: _,
        } = prog;

        asm.add(".intel_syntax noprefix");
        asm.add(".text");

        for f in func {
            asm.add(self.gen_function(f));
        }
        asm.add(".data");
        for g in globals {
            asm.add(format!("_{0}: .word 0", g));
        }

        asm
    }

    fn gen_function(&mut self, func: Function) -> Assembly {
        let mut asm = Assembly::new();

        let has_return: bool = match &func.body {
            Statement::Block(statements, _) => statements
                .iter()
                .any(|s| matches!(*s, Statement::Return(_))),
            _ => panic!("Function body should be of type Block"),
        };

        asm.add(format!(".globl _{}", func.name));
        asm.add(format!("_{}:", func.name));
        asm.add("push rbp");
        asm.add("mov rbp, rsp");

        if !has_return {
            asm.add("mov	rsp, rbp");
            asm.add("pop rbp");
            asm.add("ret\n");
        }

        asm
    }
}
