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
use crate::parser::node_type::*;

#[cfg(feature = "backend_c")]
pub mod c;
#[cfg(feature = "backend_node")]
pub mod js;
#[cfg(feature = "backend_llvm")]
pub mod llvm;
#[cfg(test)]
mod tests;
pub mod x86;

pub trait Generator {
    fn generate(prog: Module) -> String;
}

// Since we're using multiple features,
// "unreachable" statements are okay
#[allow(unreachable_code)]
pub fn generate(prog: Module) -> String {
    #[cfg(feature = "backend_llvm")]
    return llvm::LLVMGenerator::generate(prog);
    #[cfg(feature = "backend_c")]
    return c::CGenerator::generate(prog);

    #[cfg(feature = "backend_node")]
    return js::JsGenerator::generate(prog);

    panic!("No backend specified");
}
