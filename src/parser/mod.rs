mod infer;
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
pub mod node_type;
// TODO: Resolve this lint by renaming the module
#[allow(clippy::module_inception)]
mod parser;
mod rules;
use crate::lexer::Token;
use node_type::Program;
#[cfg(test)]
mod tests;

pub fn parse(tokens: Vec<Token>, raw: Option<String>) -> Result<Program, String> {
    let mut parser = parser::Parser::new(tokens, raw);
    parser.parse()
}
