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
pub mod qbe;
pub mod js;
#[cfg(feature = "llvm")]
pub mod llvm;
#[cfg(test)]
mod tests;
pub mod x86;

#[derive(Debug, Clone, Copy)]
pub enum Target {
    C,
    JS,
    QBE,
    LLVM,
}

impl Target {
    /// Constructs target based on provided output filename, returns
    /// None if target can't be detected
    pub fn from_extension(file: &path::Path) -> Option<Self> {
        let ext = file.extension()?;

        match &*ext.to_string_lossy() {
            "c" => Some(Self::C),
            "js" => Some(Self::JS),
            "ssa" => Some(Self::QBE),
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
            "qbe" => Ok(Target::QBE),
            "llvm" => Ok(Target::LLVM),

            _ => Err(format!("no target {} found", s)),
        }
    }
}

pub trait Generator {
    fn generate(prog: Module) -> String;
}
