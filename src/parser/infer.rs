use crate::ast::hast::{HBinOp, HExpression, HMatchArm, HModule, HStatement};
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
pub fn infer(program: &mut HModule) {
    let table = &program.get_symbol_table();
    for func in &mut program.func {
        let mut var_map: HashMap<String, Type> = HashMap::new();
        // Seed with parameter types
        for arg in &func.arguments {
            if let Some(ty) = &arg.ty {
                var_map.insert(arg.name.clone(), ty.clone());
            }
        }
        infer_statement(&mut func.body, table, &mut var_map);
    }
}

fn infer_statement(
    stmt: &mut HStatement,
    table: &SymbolTable,
    var_map: &mut HashMap<String, Type>,
) {
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
                        println!("Type of {} could not be infered: {:?}", &variable.name, e);
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
        HStatement::For { ident, expr, body } => {
            if ident.ty.is_none() {
                if let Some(Type::Array(elem_ty, _)) = infer_expression(expr, table, var_map) {
                    ident.ty = Some(*elem_ty);
                }
            }
            if let Some(ty) = &ident.ty {
                var_map.insert(ident.name.clone(), ty.clone());
            }
            infer_statement(body, table, var_map);
        }
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
fn infer_expression(
    expr: &HExpression,
    table: &SymbolTable,
    var_map: &HashMap<String, Type>,
) -> Option<Type> {
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
        HExpression::Variable(name) => var_map.get(name).cloned(),
        HExpression::ArrayAccess { name, .. } => {
            // Infer element type from the array variable's type
            match var_map.get(name) {
                Some(Type::Array(elem_ty, _)) => Some(*elem_ty.clone()),
                _ => None,
            }
        }
        HExpression::BinOp { lhs, op, rhs } => match op {
            HBinOp::Equal
            | HBinOp::NotEqual
            | HBinOp::LessThan
            | HBinOp::LessThanOrEqual
            | HBinOp::GreaterThan
            | HBinOp::GreaterThanOrEqual
            | HBinOp::And
            | HBinOp::Or => Some(Type::Bool),
            _ => infer_expression(lhs, table, var_map)
                .or_else(|| infer_expression(rhs, table, var_map))
                .or(Some(Type::Int)),
        },
        _ => None,
    }
}

fn infer_array(
    elements: &[HExpression],
    table: &SymbolTable,
    var_map: &HashMap<String, Type>,
) -> Option<Type> {
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
    match table.get(name) {
        Some(t) => t.to_owned(),
        None => infer_builtin(name),
    }
}

fn infer_builtin(name: &str) -> Option<Type> {
    match name {
        "len" => Some(Type::Int),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::hast::{HFunction, HVariable};
    use std::collections::HashSet;

    // ─── Helpers ────────────────────────────────────────────────────────────────

    /// Wraps a single function in a minimal module.
    fn make_module(func: HFunction) -> HModule {
        HModule {
            imports: HashSet::new(),
            func: vec![func],
            structs: vec![],
            globals: vec![],
        }
    }

    /// Shorthand for BinOp construction; both operands require Box::new.
    fn binop(lhs: HExpression, op: HBinOp, rhs: HExpression) -> HExpression {
        HExpression::BinOp {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }

    /// Extract the inferred type of the nth Declare statement inside a Block.
    fn declared_type(body: &HStatement, index: usize) -> Option<Type> {
        if let HStatement::Block { statements, .. } = body {
            if let Some(HStatement::Declare { variable, .. }) = statements.get(index) {
                return variable.ty.clone();
            }
            panic!("Statement at index {} is not a Declare", index);
        }
        panic!("Body is not a Block");
    }

    /// Extract the inferred type of the For loop identifier from the function body.
    fn for_ident_type(body: &HStatement) -> Option<Type> {
        if let HStatement::Block { statements, .. } = body {
            if let Some(HStatement::For { ident, .. }) = statements.first() {
                return ident.ty.clone();
            }
        }
        panic!("Expected For statement as first statement in block");
    }

    // ─── Group 1: infer_expression — Literal types ──────────────────────────────

    #[test]
    fn int_literal_infers_int() {
        let ty = infer_expression(&HExpression::Int(42), &HashMap::new(), &HashMap::new());
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn bool_literal_infers_bool() {
        let ty = infer_expression(&HExpression::Bool(true), &HashMap::new(), &HashMap::new());
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn str_literal_infers_str() {
        let ty = infer_expression(
            &HExpression::Str("hello".to_string()),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Str));
    }

    // ─── Group 2: infer_expression — Struct initialization ──────────────────────

    #[test]
    fn struct_init_infers_struct_type() {
        let expr = HExpression::StructInitialization {
            name: "Point".to_string(),
            fields: HashMap::new(),
        };
        let ty = infer_expression(&expr, &HashMap::new(), &HashMap::new());
        assert_eq!(ty, Some(Type::Struct("Point".to_string())));
    }

    #[test]
    fn struct_init_uses_the_struct_name_as_type() {
        let expr = HExpression::StructInitialization {
            name: "MyCustomStruct".to_string(),
            fields: HashMap::new(),
        };
        let ty = infer_expression(&expr, &HashMap::new(), &HashMap::new());
        assert_eq!(ty, Some(Type::Struct("MyCustomStruct".to_string())));
    }

    // ─── Group 3: infer_expression — Variables ───────────────────────────────────

    #[test]
    fn variable_in_var_map_infers_correct_type() {
        let var_map = HashMap::from([("x".to_string(), Type::Int)]);
        let ty = infer_expression(&HExpression::Variable("x".to_string()), &HashMap::new(), &var_map);
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn variable_not_in_var_map_returns_none() {
        let ty = infer_expression(
            &HExpression::Variable("x".to_string()),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, None);
    }

    #[test]
    fn variable_type_matches_what_was_inserted() {
        let var_map = HashMap::from([("flag".to_string(), Type::Bool)]);
        let ty = infer_expression(
            &HExpression::Variable("flag".to_string()),
            &HashMap::new(),
            &var_map,
        );
        assert_eq!(ty, Some(Type::Bool));
    }

    // ─── Group 4: infer_expression — Arrays ─────────────────────────────────────

    #[test]
    fn array_of_ints_infers_array_int_type() {
        let expr = HExpression::Array {
            capacity: 3,
            elements: vec![HExpression::Int(1), HExpression::Int(2), HExpression::Int(3)],
        };
        let ty = infer_expression(&expr, &HashMap::new(), &HashMap::new());
        assert_eq!(ty, Some(Type::Array(Box::new(Type::Int), Some(3))));
    }

    #[test]
    fn array_of_bools_infers_array_bool_type() {
        let expr = HExpression::Array {
            capacity: 2,
            elements: vec![HExpression::Bool(true), HExpression::Bool(false)],
        };
        let ty = infer_expression(&expr, &HashMap::new(), &HashMap::new());
        assert_eq!(ty, Some(Type::Array(Box::new(Type::Bool), Some(2))));
    }

    #[test]
    fn array_of_strings_infers_array_str_type() {
        let expr = HExpression::Array {
            capacity: 2,
            elements: vec![HExpression::Str("a".to_string()), HExpression::Str("b".to_string())],
        };
        let ty = infer_expression(&expr, &HashMap::new(), &HashMap::new());
        assert_eq!(ty, Some(Type::Array(Box::new(Type::Str), Some(2))));
    }

    #[test]
    fn empty_array_returns_none() {
        let expr = HExpression::Array { capacity: 0, elements: vec![] };
        let ty = infer_expression(&expr, &HashMap::new(), &HashMap::new());
        assert_eq!(ty, None);
    }

    #[test]
    fn array_capacity_matches_element_count() {
        let expr = HExpression::Array {
            capacity: 4,
            elements: vec![
                HExpression::Int(1),
                HExpression::Int(2),
                HExpression::Int(3),
                HExpression::Int(4),
            ],
        };
        let ty = infer_expression(&expr, &HashMap::new(), &HashMap::new());
        assert_eq!(ty, Some(Type::Array(Box::new(Type::Int), Some(4))));
    }

    // ─── Group 5: infer_expression — Array access ───────────────────────────────

    #[test]
    fn array_access_known_array_var_returns_element_type() {
        let var_map = HashMap::from([("arr".to_string(), Type::Array(Box::new(Type::Int), Some(3)))]);
        let expr = HExpression::ArrayAccess {
            name: "arr".to_string(),
            index: Box::new(HExpression::Int(0)),
        };
        let ty = infer_expression(&expr, &HashMap::new(), &var_map);
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn array_access_known_bool_array_returns_bool_element_type() {
        let var_map = HashMap::from([("flags".to_string(), Type::Array(Box::new(Type::Bool), None))]);
        let expr = HExpression::ArrayAccess {
            name: "flags".to_string(),
            index: Box::new(HExpression::Int(1)),
        };
        let ty = infer_expression(&expr, &HashMap::new(), &var_map);
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn array_access_unknown_var_returns_none() {
        let expr = HExpression::ArrayAccess {
            name: "arr".to_string(),
            index: Box::new(HExpression::Int(0)),
        };
        let ty = infer_expression(&expr, &HashMap::new(), &HashMap::new());
        assert_eq!(ty, None);
    }

    #[test]
    fn array_access_on_non_array_var_returns_none() {
        let var_map = HashMap::from([("x".to_string(), Type::Int)]);
        let expr = HExpression::ArrayAccess {
            name: "x".to_string(),
            index: Box::new(HExpression::Int(0)),
        };
        let ty = infer_expression(&expr, &HashMap::new(), &var_map);
        assert_eq!(ty, None);
    }

    // ─── Group 6: infer_expression — Binary operations ──────────────────────────

    #[test]
    fn binop_equal_returns_bool() {
        let ty = infer_expression(
            &binop(HExpression::Int(1), HBinOp::Equal, HExpression::Int(1)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_not_equal_returns_bool() {
        let ty = infer_expression(
            &binop(HExpression::Int(1), HBinOp::NotEqual, HExpression::Int(2)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_less_than_returns_bool() {
        let ty = infer_expression(
            &binop(HExpression::Int(1), HBinOp::LessThan, HExpression::Int(2)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_less_than_or_equal_returns_bool() {
        let ty = infer_expression(
            &binop(HExpression::Int(2), HBinOp::LessThanOrEqual, HExpression::Int(2)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_greater_than_returns_bool() {
        let ty = infer_expression(
            &binop(HExpression::Int(3), HBinOp::GreaterThan, HExpression::Int(1)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_greater_than_or_equal_returns_bool() {
        let ty = infer_expression(
            &binop(HExpression::Int(3), HBinOp::GreaterThanOrEqual, HExpression::Int(3)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_and_returns_bool() {
        let ty = infer_expression(
            &binop(HExpression::Bool(true), HBinOp::And, HExpression::Bool(false)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_or_returns_bool() {
        let ty = infer_expression(
            &binop(HExpression::Bool(false), HBinOp::Or, HExpression::Bool(true)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_addition_infers_int_from_int_operands() {
        let ty = infer_expression(
            &binop(HExpression::Int(1), HBinOp::Addition, HExpression::Int(2)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn binop_subtraction_infers_int_from_int_operands() {
        let ty = infer_expression(
            &binop(HExpression::Int(5), HBinOp::Subtraction, HExpression::Int(3)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn binop_multiplication_infers_int_from_int_operands() {
        let ty = infer_expression(
            &binop(HExpression::Int(2), HBinOp::Multiplication, HExpression::Int(3)),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn binop_arithmetic_on_unknown_variables_defaults_to_int() {
        // Both variables absent from var_map → fallback to Some(Type::Int)
        let ty = infer_expression(
            &binop(
                HExpression::Variable("x".to_string()),
                HBinOp::Addition,
                HExpression::Variable("y".to_string()),
            ),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn binop_arithmetic_infers_from_rhs_when_lhs_unknown() {
        // lhs variable absent, rhs is an int literal → infer Int from rhs
        let ty = infer_expression(
            &binop(
                HExpression::Variable("unknown".to_string()),
                HBinOp::Addition,
                HExpression::Int(5),
            ),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(ty, Some(Type::Int));
    }

    // ─── Group 7: infer_function_call ───────────────────────────────────────────

    #[test]
    fn function_call_found_in_table_returns_its_return_type() {
        let table = HashMap::from([("foo".to_string(), Some(Type::Int))]);
        assert_eq!(infer_function_call("foo", &table), Some(Type::Int));
    }

    #[test]
    fn function_call_with_none_return_type_in_table_returns_none() {
        let table = HashMap::from([("void_fn".to_string(), None)]);
        assert_eq!(infer_function_call("void_fn", &table), None);
    }

    #[test]
    fn function_call_not_in_table_falls_back_to_builtin() {
        // "len" is a builtin → returns Some(Type::Int) even with empty table
        assert_eq!(infer_function_call("len", &HashMap::new()), Some(Type::Int));
    }

    #[test]
    fn function_call_unknown_fn_not_in_table_or_builtins_returns_none() {
        assert_eq!(infer_function_call("unknown_function", &HashMap::new()), None);
    }

    // ─── Group 8: infer_builtin ──────────────────────────────────────────────────

    #[test]
    fn builtin_len_returns_int() {
        assert_eq!(infer_builtin("len"), Some(Type::Int));
    }

    #[test]
    fn builtin_unknown_name_returns_none() {
        assert_eq!(infer_builtin("println"), None);
    }

    #[test]
    fn builtin_empty_name_returns_none() {
        assert_eq!(infer_builtin(""), None);
    }

    // ─── Group 9: infer — Declaration type inference ────────────────────────────

    #[test]
    fn declare_infers_int_from_int_literal() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Declare {
                    variable: HVariable { name: "x".to_string(), ty: None },
                    value: Some(HExpression::Int(42)),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Int));
    }

    #[test]
    fn declare_infers_bool_from_bool_literal() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Declare {
                    variable: HVariable { name: "b".to_string(), ty: None },
                    value: Some(HExpression::Bool(true)),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Bool));
    }

    #[test]
    fn declare_infers_str_from_str_literal() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Declare {
                    variable: HVariable { name: "s".to_string(), ty: None },
                    value: Some(HExpression::Str("hello".to_string())),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Str));
    }

    #[test]
    fn declare_does_not_overwrite_explicit_type() {
        // Variable has explicit type Bool but value is an int literal; Bool must be preserved.
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Declare {
                    variable: HVariable { name: "x".to_string(), ty: Some(Type::Bool) },
                    value: Some(HExpression::Int(42)),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Bool));
    }

    #[test]
    fn declare_uninitialized_without_type_stays_none() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Declare {
                    variable: HVariable { name: "x".to_string(), ty: None },
                    value: None,
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), None);
    }

    #[test]
    fn declare_propagates_type_to_subsequent_variable() {
        // let x = 1; let y = x  →  y gets Type::Int
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![
                    HStatement::Declare {
                        variable: HVariable { name: "x".to_string(), ty: None },
                        value: Some(HExpression::Int(1)),
                    },
                    HStatement::Declare {
                        variable: HVariable { name: "y".to_string(), ty: None },
                        value: Some(HExpression::Variable("x".to_string())),
                    },
                ],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 1), Some(Type::Int));
    }

    // ─── Group 10: infer — Function parameters seed var_map ─────────────────────

    #[test]
    fn function_param_type_propagates_to_body_variable() {
        // fn foo(x: int) { let y = x }  →  y gets Type::Int
        let mut module = make_module(HFunction {
            name: "foo".to_string(),
            arguments: vec![HVariable { name: "x".to_string(), ty: Some(Type::Int) }],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Declare {
                    variable: HVariable { name: "y".to_string(), ty: None },
                    value: Some(HExpression::Variable("x".to_string())),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Int));
    }

    #[test]
    fn function_param_without_type_does_not_propagate() {
        // fn foo(x) { let y = x }  →  y stays None (param has no type annotation)
        let mut module = make_module(HFunction {
            name: "foo".to_string(),
            arguments: vec![HVariable { name: "x".to_string(), ty: None }],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Declare {
                    variable: HVariable { name: "y".to_string(), ty: None },
                    value: Some(HExpression::Variable("x".to_string())),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), None);
    }

    #[test]
    fn multiple_function_params_each_propagate_their_type() {
        let mut module = make_module(HFunction {
            name: "foo".to_string(),
            arguments: vec![
                HVariable { name: "a".to_string(), ty: Some(Type::Int) },
                HVariable { name: "b".to_string(), ty: Some(Type::Bool) },
            ],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![
                    HStatement::Declare {
                        variable: HVariable { name: "x".to_string(), ty: None },
                        value: Some(HExpression::Variable("a".to_string())),
                    },
                    HStatement::Declare {
                        variable: HVariable { name: "y".to_string(), ty: None },
                        value: Some(HExpression::Variable("b".to_string())),
                    },
                ],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Int));
        assert_eq!(declared_type(&module.func[0].body, 1), Some(Type::Bool));
    }

    // ─── Group 11: infer — For loop identifier inference ────────────────────────

    #[test]
    fn for_loop_ident_infers_element_type_from_int_array() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::For {
                    ident: HVariable { name: "i".to_string(), ty: None },
                    expr: HExpression::Array {
                        capacity: 3,
                        elements: vec![HExpression::Int(1), HExpression::Int(2), HExpression::Int(3)],
                    },
                    body: Box::new(HStatement::Block { statements: vec![], scope: vec![] }),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(for_ident_type(&module.func[0].body), Some(Type::Int));
    }

    #[test]
    fn for_loop_ident_infers_element_type_from_bool_array() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::For {
                    ident: HVariable { name: "flag".to_string(), ty: None },
                    expr: HExpression::Array {
                        capacity: 2,
                        elements: vec![HExpression::Bool(true), HExpression::Bool(false)],
                    },
                    body: Box::new(HStatement::Block { statements: vec![], scope: vec![] }),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(for_ident_type(&module.func[0].body), Some(Type::Bool));
    }

    #[test]
    fn for_loop_ident_with_explicit_type_is_not_overwritten() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::For {
                    ident: HVariable { name: "i".to_string(), ty: Some(Type::Bool) },
                    expr: HExpression::Array {
                        capacity: 2,
                        elements: vec![HExpression::Int(1), HExpression::Int(2)],
                    },
                    body: Box::new(HStatement::Block { statements: vec![], scope: vec![] }),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(for_ident_type(&module.func[0].body), Some(Type::Bool));
    }

    #[test]
    fn for_loop_body_variable_gets_element_type_via_loop_ident() {
        // for i in [1,2,3] { let x = i }  →  x gets Type::Int
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::For {
                    ident: HVariable { name: "i".to_string(), ty: None },
                    expr: HExpression::Array {
                        capacity: 3,
                        elements: vec![HExpression::Int(1), HExpression::Int(2), HExpression::Int(3)],
                    },
                    body: Box::new(HStatement::Block {
                        statements: vec![HStatement::Declare {
                            variable: HVariable { name: "x".to_string(), ty: None },
                            value: Some(HExpression::Variable("i".to_string())),
                        }],
                        scope: vec![],
                    }),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);

        if let HStatement::Block { statements, .. } = &module.func[0].body {
            if let HStatement::For { body: for_body, .. } = &statements[0] {
                if let HStatement::Block { statements: inner, .. } = for_body.as_ref() {
                    if let HStatement::Declare { variable, .. } = &inner[0] {
                        assert_eq!(variable.ty, Some(Type::Int));
                        return;
                    }
                }
            }
        }
        panic!("Could not navigate to inner declare statement");
    }

    // ─── Group 12: infer — Control flow bodies are processed ────────────────────

    #[test]
    fn if_body_variables_are_inferred() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::If {
                    condition: HExpression::Bool(true),
                    body: Box::new(HStatement::Block {
                        statements: vec![HStatement::Declare {
                            variable: HVariable { name: "x".to_string(), ty: None },
                            value: Some(HExpression::Int(5)),
                        }],
                        scope: vec![],
                    }),
                    else_branch: None,
                }],
                scope: vec![],
            },
        });
        infer(&mut module);

        if let HStatement::Block { statements, .. } = &module.func[0].body {
            if let HStatement::If { body: if_body, .. } = &statements[0] {
                assert_eq!(declared_type(if_body, 0), Some(Type::Int));
                return;
            }
        }
        panic!("Could not navigate to if body");
    }

    #[test]
    fn else_body_variables_are_inferred() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::If {
                    condition: HExpression::Bool(true),
                    body: Box::new(HStatement::Block { statements: vec![], scope: vec![] }),
                    else_branch: Some(Box::new(HStatement::Block {
                        statements: vec![HStatement::Declare {
                            variable: HVariable { name: "x".to_string(), ty: None },
                            value: Some(HExpression::Str("hi".to_string())),
                        }],
                        scope: vec![],
                    })),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);

        if let HStatement::Block { statements, .. } = &module.func[0].body {
            if let HStatement::If { else_branch: Some(else_body), .. } = &statements[0] {
                assert_eq!(declared_type(else_body, 0), Some(Type::Str));
                return;
            }
        }
        panic!("Could not navigate to else body");
    }

    #[test]
    fn while_body_variables_are_inferred() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::While {
                    condition: HExpression::Bool(true),
                    body: Box::new(HStatement::Block {
                        statements: vec![HStatement::Declare {
                            variable: HVariable { name: "x".to_string(), ty: None },
                            value: Some(HExpression::Bool(false)),
                        }],
                        scope: vec![],
                    }),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);

        if let HStatement::Block { statements, .. } = &module.func[0].body {
            if let HStatement::While { body: while_body, .. } = &statements[0] {
                assert_eq!(declared_type(while_body, 0), Some(Type::Bool));
                return;
            }
        }
        panic!("Could not navigate to while body");
    }

    #[test]
    fn match_case_arm_variables_are_inferred() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Match {
                    subject: HExpression::Int(1),
                    arms: vec![HMatchArm::Case(
                        HExpression::Int(1),
                        HStatement::Block {
                            statements: vec![HStatement::Declare {
                                variable: HVariable { name: "x".to_string(), ty: None },
                                value: Some(HExpression::Int(99)),
                            }],
                            scope: vec![],
                        },
                    )],
                }],
                scope: vec![],
            },
        });
        infer(&mut module);

        if let HStatement::Block { statements, .. } = &module.func[0].body {
            if let HStatement::Match { arms, .. } = &statements[0] {
                if let HMatchArm::Case(_, arm_body) = &arms[0] {
                    assert_eq!(declared_type(arm_body, 0), Some(Type::Int));
                    return;
                }
            }
        }
        panic!("Could not navigate to match case arm body");
    }

    #[test]
    fn match_else_arm_variables_are_inferred() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Match {
                    subject: HExpression::Int(1),
                    arms: vec![HMatchArm::Else(HStatement::Block {
                        statements: vec![HStatement::Declare {
                            variable: HVariable { name: "x".to_string(), ty: None },
                            value: Some(HExpression::Str("default".to_string())),
                        }],
                        scope: vec![],
                    })],
                }],
                scope: vec![],
            },
        });
        infer(&mut module);

        if let HStatement::Block { statements, .. } = &module.func[0].body {
            if let HStatement::Match { arms, .. } = &statements[0] {
                if let HMatchArm::Else(arm_body) = &arms[0] {
                    assert_eq!(declared_type(arm_body, 0), Some(Type::Str));
                    return;
                }
            }
        }
        panic!("Could not navigate to match else arm body");
    }

    #[test]
    fn nested_blocks_are_fully_inferred() {
        // { { let x = 42 } }  →  x gets Type::Int
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Block {
                    statements: vec![HStatement::Declare {
                        variable: HVariable { name: "x".to_string(), ty: None },
                        value: Some(HExpression::Int(42)),
                    }],
                    scope: vec![],
                }],
                scope: vec![],
            },
        });
        infer(&mut module);

        if let HStatement::Block { statements, .. } = &module.func[0].body {
            if let HStatement::Block { statements: inner_stmts, .. } = &statements[0] {
                if let HStatement::Declare { variable, .. } = &inner_stmts[0] {
                    assert_eq!(variable.ty, Some(Type::Int));
                    return;
                }
            }
        }
        panic!("Could not navigate to nested block declare");
    }

    // ─── Group 13: infer — Function call type inference via symbol table ─────────

    #[test]
    fn function_call_expr_is_inferred_from_symbol_table() {
        // fn helper() -> int { return 1 }
        // fn main() { let x = helper() }  →  x gets Type::Int
        let mut module = HModule {
            imports: HashSet::new(),
            func: vec![
                HFunction {
                    name: "helper".to_string(),
                    arguments: vec![],
                    ret_type: Some(Type::Int),
                    body: HStatement::Block {
                        statements: vec![HStatement::Return(Some(HExpression::Int(1)))],
                        scope: vec![],
                    },
                },
                HFunction {
                    name: "main".to_string(),
                    arguments: vec![],
                    ret_type: None,
                    body: HStatement::Block {
                        statements: vec![HStatement::Declare {
                            variable: HVariable { name: "x".to_string(), ty: None },
                            value: Some(HExpression::FunctionCall {
                                fn_name: "helper".to_string(),
                                args: vec![],
                            }),
                        }],
                        scope: vec![],
                    },
                },
            ],
            structs: vec![],
            globals: vec![],
        };
        infer(&mut module);
        assert_eq!(declared_type(&module.func[1].body, 0), Some(Type::Int));
    }

    #[test]
    fn builtin_len_call_is_inferred_as_int() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Declare {
                    variable: HVariable { name: "n".to_string(), ty: None },
                    value: Some(HExpression::FunctionCall {
                        fn_name: "len".to_string(),
                        args: vec![HExpression::Variable("arr".to_string())],
                    }),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Int));
    }

    #[test]
    fn unknown_function_call_leaves_type_as_none() {
        let mut module = make_module(HFunction {
            name: "main".to_string(),
            arguments: vec![],
            ret_type: None,
            body: HStatement::Block {
                statements: vec![HStatement::Declare {
                    variable: HVariable { name: "x".to_string(), ty: None },
                    value: Some(HExpression::FunctionCall {
                        fn_name: "totally_unknown_fn".to_string(),
                        args: vec![],
                    }),
                }],
                scope: vec![],
            },
        });
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), None);
    }
}
