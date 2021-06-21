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
use crate::ast::*;
use std::path;
use std::str::FromStr;

pub mod c;
pub mod js;
#[cfg(feature = "llvm")]
pub mod llvm;
pub mod qbe;
#[cfg(test)]
mod tests;
pub mod x86;

#[derive(Debug)]
pub enum Target {
    C,
    JS,
    Llvm,
    Qbe,
    X86,
}

impl Target {
    /// Constructs target based on provided output filename, returns
    /// None if target can't be detected
    pub fn from_extension(file: &path::Path) -> Option<Self> {
        let ext = file.extension()?;

        match &*ext.to_string_lossy() {
            "c" => Some(Self::C),
            "js" => Some(Self::JS),
            "ssa" => Some(Self::Qbe),
            "s" => Some(Self::X86),
            _ => None,
        }
    }
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        match s.as_str() {
            "c" => Ok(Target::C),
            "js" => Ok(Target::JS),
            "llvm" => Ok(Target::Llvm),
            "qbe" => Ok(Target::Qbe),
            "x86" => Ok(Target::X86),
            _ => Err(format!("no target {} found", s)),
        }
    }
}

pub trait Generator {
    fn generate(prog: Module) -> String;
}

/// Returns C syntax representation of a raw string
pub fn string_syntax(raw: String) -> String {
    format!(
        "\"{}\"",
        raw.chars()
            .map(|c| match c {
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\u{000C}' => "\\f".to_string(),
                '\u{0008}' => "\\b".to_string(),
                '\\' => "\\\\".to_string(),
                '"' => "\"".to_string(),
                other => other.to_string(),
            })
            .collect::<String>(),
    )
}
