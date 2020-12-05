use crate::generator::Generator;
use crate::parser::node_type::*;

pub struct JsGenerator;

impl Generator for JsGenerator {
    fn generate(prog: Program) -> String {
        let mut code = prog
            .func
            .into_iter()
            .map(|f| generate_function(f))
            .collect();

        code += "main()";

        return code;
    }
}

fn generate_function(func: Function) -> String {
    let arguments: String = func
        .arguments
        .into_iter()
        .map(|arg: Variable| format!("{},", arg.name))
        .collect();
    let mut raw = format!("function {N}({A}) ", N = func.name, A = arguments);

    raw += "{}\n";

    raw
}
