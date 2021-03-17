/**
 * Copyright 2021 Garrit Franke
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
use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type {
    Any,
    Int,
    Str,
    Bool,
    Array(Box<Type>, Option<usize>),
    Struct(String),
}

impl TryFrom<String> for Type {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_ref() {
            "int" => Ok(Self::Int),
            "string" => Ok(Self::Str),
            "any" => Ok(Self::Any),
            "bool" => Ok(Self::Bool),
            name => Ok(Self::Struct(name.to_string())),
        }
    }
}
