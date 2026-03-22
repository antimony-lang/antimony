use crate::ast::hast::{HExpression, HMatchArm, HModule, HStatement};
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
use crate::ast::types::Type;
use crate::ast::SymbolTable;
use std::collections::HashMap;

/// Try to infer types of variables
///
/// TODO: Global symbol table is passed around randomly.
/// This could probably be cleaned up.
pub(super) fn infer(program: &mut HModule) {
    let table = &program.get_symbol_table();
    for func in &mut program.func {
        let mut var_map: HashMap<String, Type> = HashMap::new();
        infer_statement(&mut func.body, table, &mut var_map);
    }
}

fn infer_statement(stmt: &mut HStatement, table: &SymbolTable, var_map: &mut HashMap<String, Type>) {
    match stmt {
        HStatement::Block { statements, .. } => {
            for s in statements {
                infer_statement(s, table, var_map);
            }
        }
        HStatement::Declare { variable, value } => {
            if variable.ty.is_none() {
                if let Some(e) = value {
                    variable.ty = infer_expression(e, table, var_map);
                    #[cfg(debug_assertions)]
                    if variable.ty.is_none() {
                        println!(
                            "Type of {} could not be infered: {:?}",
                            &variable.name, e
                        );
                    }
                }
            }
            if let Some(ty) = &variable.ty {
                var_map.insert(variable.name.clone(), ty.clone());
            }
        }
        HStatement::If {
            body, else_branch, ..
        } => {
            infer_statement(body, table, var_map);
            if let Some(else_stmt) = else_branch {
                infer_statement(else_stmt, table, var_map);
            }
        }
        HStatement::While { body, .. } => infer_statement(body, table, var_map),
        HStatement::For { body, .. } => infer_statement(body, table, var_map),
        HStatement::Match { arms, .. } => {
            for arm in arms {
                match arm {
                    HMatchArm::Case(_, s) | HMatchArm::Else(s) => {
                        infer_statement(s, table, var_map);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Function table is needed to infer possible function calls
fn infer_expression(expr: &HExpression, table: &SymbolTable, var_map: &HashMap<String, Type>) -> Option<Type> {
    match expr {
        HExpression::Int(_) => Some(Type::Int),
        HExpression::Bool(_) => Some(Type::Bool),
        HExpression::Str(_) => Some(Type::Str),
        HExpression::StructInitialization { name, fields: _ } => {
            Some(Type::Struct(name.to_string()))
        }
        HExpression::FunctionCall { fn_name, args: _ } => infer_function_call(fn_name, table),
        HExpression::Array {
            capacity: _,
            elements,
        } => infer_array(elements, table, var_map),
        HExpression::ArrayAccess { name, .. } => {
            // Infer element type from the array variable's type
            match var_map.get(name) {
                Some(Type::Array(elem_ty, _)) => Some(*elem_ty.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn infer_array(elements: &[HExpression], table: &SymbolTable, var_map: &HashMap<String, Type>) -> Option<Type> {
    let types: Vec<Option<Type>> = elements
        .iter()
        .map(|el| infer_expression(el, table, var_map))
        .collect();

    // TODO: This approach only relies on the first element.
    // It will not catch that types are possibly inconsistent.
    types
        .first()
        .and_then(|ty| ty.to_owned())
        .map(|ty| Type::Array(Box::new(ty), Some(types.len())))
}

fn infer_function_call(name: &str, table: &SymbolTable) -> Option<Type> {
    // Built-in functions with known return types
    match name {
        "len" => return Some(Type::Int),
        _ => {}
    }
    match table.get(name) {
        Some(t) => t.to_owned(),
        None => None,
    }
}
