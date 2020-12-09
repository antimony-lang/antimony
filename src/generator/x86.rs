use crate::generator::Generator;
use crate::parser::node_type::{Function, Program, Statement};

struct Assembly {
    asm: Vec<String>,
}

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

    fn add_all<S: Into<String>>(&mut self, strings: Vec<S>) {
        for string in strings {
            self.asm.push(string.into())
        }
    }

    fn build(&self) -> String {
        self.asm.join("\n")
    }
}

pub struct X86Generator;

impl Generator for X86Generator {
    fn generate(prog: Program) -> String {
        Self::new().gen_program(prog).build()
    }
}

impl X86Generator {
    fn new() -> Self {
        X86Generator {}
    }

    fn gen_program(&mut self, prog: Program) -> Assembly {
        let mut asm = Assembly::new();
        match prog {
            Program { func, globals } => {
                asm.add(".intel_syntax noprefix");
                asm.add(".text");

                for f in func {
                    asm.add(self.gen_function(f));
                }
                asm.add(".data");
                for g in globals {
                    asm.add(format!("_{0}: .word 0", g));
                }
            }
        };

        asm
    }

    fn gen_function(&mut self, func: Function) -> Assembly {
        let mut asm = Assembly::new();

        let has_return: bool = match &func.body {
            Statement::Block(statements) => statements.iter().any(|s| {
                if let Statement::Return(_) = *s {
                    true
                } else {
                    false
                }
            }),
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
