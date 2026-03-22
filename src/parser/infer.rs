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

    fn make_module(func: HFunction) -> HModule {
        HModule {
            imports: HashSet::new(),
            func: vec![func],
            structs: vec![],
            globals: vec![],
        }
    }

    fn make_func(
        name: &str,
        args: Vec<HVariable>,
        body: HStatement,
        ret_type: Option<Type>,
    ) -> HFunction {
        HFunction {
            name: name.to_string(),
            arguments: args,
            body,
            ret_type,
        }
    }

    fn make_var(name: &str, ty: Option<Type>) -> HVariable {
        HVariable {
            name: name.to_string(),
            ty,
        }
    }

    /// Block statement with no scoped variables.
    fn block(stmts: Vec<HStatement>) -> HStatement {
        HStatement::Block {
            statements: stmts,
            scope: vec![],
        }
    }

    fn declare(name: &str, ty: Option<Type>, value: Option<HExpression>) -> HStatement {
        HStatement::Declare {
            variable: make_var(name, ty),
            value,
        }
    }

    fn int_expr(n: usize) -> HExpression {
        HExpression::Int(n)
    }

    fn bool_expr(b: bool) -> HExpression {
        HExpression::Bool(b)
    }

    fn str_expr(s: &str) -> HExpression {
        HExpression::Str(s.to_string())
    }

    fn var_expr(name: &str) -> HExpression {
        HExpression::Variable(name.to_string())
    }

    fn array_expr(elements: Vec<HExpression>) -> HExpression {
        HExpression::Array {
            capacity: elements.len(),
            elements,
        }
    }

    fn call_expr(fn_name: &str, args: Vec<HExpression>) -> HExpression {
        HExpression::FunctionCall {
            fn_name: fn_name.to_string(),
            args,
        }
    }

    fn binop(lhs: HExpression, op: HBinOp, rhs: HExpression) -> HExpression {
        HExpression::BinOp {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }

    fn empty_table() -> SymbolTable {
        HashMap::new()
    }

    fn empty_var_map() -> HashMap<String, Type> {
        HashMap::new()
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
        let ty = infer_expression(&int_expr(42), &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn bool_literal_infers_bool() {
        let ty = infer_expression(&bool_expr(true), &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn str_literal_infers_str() {
        let ty = infer_expression(&str_expr("hello"), &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Str));
    }

    // ─── Group 2: infer_expression — Struct initialization ──────────────────────

    #[test]
    fn struct_init_infers_struct_type() {
        let expr = HExpression::StructInitialization {
            name: "Point".to_string(),
            fields: HashMap::new(),
        };
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Struct("Point".to_string())));
    }

    #[test]
    fn struct_init_uses_the_struct_name_as_type() {
        let expr = HExpression::StructInitialization {
            name: "MyCustomStruct".to_string(),
            fields: HashMap::new(),
        };
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Struct("MyCustomStruct".to_string())));
    }

    // ─── Group 3: infer_expression — Variables ───────────────────────────────────

    #[test]
    fn variable_in_var_map_infers_correct_type() {
        let mut var_map = empty_var_map();
        var_map.insert("x".to_string(), Type::Int);
        let ty = infer_expression(&var_expr("x"), &empty_table(), &var_map);
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn variable_not_in_var_map_returns_none() {
        let ty = infer_expression(&var_expr("x"), &empty_table(), &empty_var_map());
        assert_eq!(ty, None);
    }

    #[test]
    fn variable_type_matches_what_was_inserted() {
        let mut var_map = empty_var_map();
        var_map.insert("flag".to_string(), Type::Bool);
        let ty = infer_expression(&var_expr("flag"), &empty_table(), &var_map);
        assert_eq!(ty, Some(Type::Bool));
    }

    // ─── Group 4: infer_expression — Arrays ─────────────────────────────────────

    #[test]
    fn array_of_ints_infers_array_int_type() {
        let expr = array_expr(vec![int_expr(1), int_expr(2), int_expr(3)]);
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Array(Box::new(Type::Int), Some(3))));
    }

    #[test]
    fn array_of_bools_infers_array_bool_type() {
        let expr = array_expr(vec![bool_expr(true), bool_expr(false)]);
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Array(Box::new(Type::Bool), Some(2))));
    }

    #[test]
    fn array_of_strings_infers_array_str_type() {
        let expr = array_expr(vec![str_expr("a"), str_expr("b")]);
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Array(Box::new(Type::Str), Some(2))));
    }

    #[test]
    fn empty_array_returns_none() {
        let expr = array_expr(vec![]);
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, None);
    }

    #[test]
    fn array_capacity_matches_element_count() {
        let expr = array_expr(vec![int_expr(1), int_expr(2), int_expr(3), int_expr(4)]);
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Array(Box::new(Type::Int), Some(4))));
    }

    // ─── Group 5: infer_expression — Array access ───────────────────────────────

    #[test]
    fn array_access_known_array_var_returns_element_type() {
        let mut var_map = empty_var_map();
        var_map.insert("arr".to_string(), Type::Array(Box::new(Type::Int), Some(3)));
        let expr = HExpression::ArrayAccess {
            name: "arr".to_string(),
            index: Box::new(int_expr(0)),
        };
        let ty = infer_expression(&expr, &empty_table(), &var_map);
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn array_access_known_bool_array_returns_bool_element_type() {
        let mut var_map = empty_var_map();
        var_map.insert("flags".to_string(), Type::Array(Box::new(Type::Bool), None));
        let expr = HExpression::ArrayAccess {
            name: "flags".to_string(),
            index: Box::new(int_expr(1)),
        };
        let ty = infer_expression(&expr, &empty_table(), &var_map);
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn array_access_unknown_var_returns_none() {
        let expr = HExpression::ArrayAccess {
            name: "arr".to_string(),
            index: Box::new(int_expr(0)),
        };
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, None);
    }

    #[test]
    fn array_access_on_non_array_var_returns_none() {
        let mut var_map = empty_var_map();
        var_map.insert("x".to_string(), Type::Int);
        let expr = HExpression::ArrayAccess {
            name: "x".to_string(),
            index: Box::new(int_expr(0)),
        };
        let ty = infer_expression(&expr, &empty_table(), &var_map);
        assert_eq!(ty, None);
    }

    // ─── Group 6: infer_expression — Binary operations ──────────────────────────

    #[test]
    fn binop_equal_returns_bool() {
        let expr = binop(int_expr(1), HBinOp::Equal, int_expr(1));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_not_equal_returns_bool() {
        let expr = binop(int_expr(1), HBinOp::NotEqual, int_expr(2));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_less_than_returns_bool() {
        let expr = binop(int_expr(1), HBinOp::LessThan, int_expr(2));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_less_than_or_equal_returns_bool() {
        let expr = binop(int_expr(2), HBinOp::LessThanOrEqual, int_expr(2));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_greater_than_returns_bool() {
        let expr = binop(int_expr(3), HBinOp::GreaterThan, int_expr(1));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_greater_than_or_equal_returns_bool() {
        let expr = binop(int_expr(3), HBinOp::GreaterThanOrEqual, int_expr(3));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_and_returns_bool() {
        let expr = binop(bool_expr(true), HBinOp::And, bool_expr(false));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_or_returns_bool() {
        let expr = binop(bool_expr(false), HBinOp::Or, bool_expr(true));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Bool));
    }

    #[test]
    fn binop_addition_infers_int_from_int_operands() {
        let expr = binop(int_expr(1), HBinOp::Addition, int_expr(2));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn binop_subtraction_infers_int_from_int_operands() {
        let expr = binop(int_expr(5), HBinOp::Subtraction, int_expr(3));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn binop_multiplication_infers_int_from_int_operands() {
        let expr = binop(int_expr(2), HBinOp::Multiplication, int_expr(3));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn binop_arithmetic_on_unknown_variables_defaults_to_int() {
        // Both variables unknown → fallback to Some(Type::Int)
        let expr = binop(var_expr("x"), HBinOp::Addition, var_expr("y"));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn binop_arithmetic_infers_from_rhs_when_lhs_unknown() {
        // lhs unknown, rhs is an int literal → infer Int from rhs
        let expr = binop(var_expr("unknown"), HBinOp::Addition, int_expr(5));
        let ty = infer_expression(&expr, &empty_table(), &empty_var_map());
        assert_eq!(ty, Some(Type::Int));
    }

    // ─── Group 7: infer_function_call ───────────────────────────────────────────

    #[test]
    fn function_call_found_in_table_returns_its_return_type() {
        let mut table = empty_table();
        table.insert("foo".to_string(), Some(Type::Int));
        let ty = infer_function_call("foo", &table);
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn function_call_with_none_return_type_in_table_returns_none() {
        let mut table = empty_table();
        table.insert("void_fn".to_string(), None);
        let ty = infer_function_call("void_fn", &table);
        assert_eq!(ty, None);
    }

    #[test]
    fn function_call_not_in_table_falls_back_to_builtin() {
        // "len" is a builtin → should return Some(Type::Int) even with empty table
        let ty = infer_function_call("len", &empty_table());
        assert_eq!(ty, Some(Type::Int));
    }

    #[test]
    fn function_call_unknown_fn_not_in_table_or_builtins_returns_none() {
        let ty = infer_function_call("unknown_function", &empty_table());
        assert_eq!(ty, None);
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
        let body = block(vec![declare("x", None, Some(int_expr(42)))]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Int));
    }

    #[test]
    fn declare_infers_bool_from_bool_literal() {
        let body = block(vec![declare("b", None, Some(bool_expr(true)))]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Bool));
    }

    #[test]
    fn declare_infers_str_from_str_literal() {
        let body = block(vec![declare("s", None, Some(str_expr("hello")))]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Str));
    }

    #[test]
    fn declare_does_not_overwrite_explicit_type() {
        // Variable has explicit type Bool but value is an int literal
        let body = block(vec![declare("x", Some(Type::Bool), Some(int_expr(42)))]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        // The explicit Bool type must be preserved
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Bool));
    }

    #[test]
    fn declare_uninitialized_without_type_stays_none() {
        let body = block(vec![declare("x", None, None)]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), None);
    }

    #[test]
    fn declare_propagates_type_to_subsequent_variable() {
        // let x = 1; let y = x  →  y should get Type::Int
        let body = block(vec![
            declare("x", None, Some(int_expr(1))),
            declare("y", None, Some(var_expr("x"))),
        ]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 1), Some(Type::Int));
    }

    // ─── Group 10: infer — Function parameters seed var_map ─────────────────────

    #[test]
    fn function_param_type_propagates_to_body_variable() {
        // fn foo(x: int) { let y = x }  →  y gets Type::Int
        let param = make_var("x", Some(Type::Int));
        let body = block(vec![declare("y", None, Some(var_expr("x")))]);
        let mut module = make_module(make_func("foo", vec![param], body, None));
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Int));
    }

    #[test]
    fn function_param_without_type_does_not_propagate() {
        // fn foo(x) { let y = x }  →  y stays None (no type on param)
        let param = make_var("x", None);
        let body = block(vec![declare("y", None, Some(var_expr("x")))]);
        let mut module = make_module(make_func("foo", vec![param], body, None));
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), None);
    }

    #[test]
    fn multiple_function_params_each_propagate_their_type() {
        let params = vec![
            make_var("a", Some(Type::Int)),
            make_var("b", Some(Type::Bool)),
        ];
        let body = block(vec![
            declare("x", None, Some(var_expr("a"))),
            declare("y", None, Some(var_expr("b"))),
        ]);
        let mut module = make_module(make_func("foo", params, body, None));
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Int));
        assert_eq!(declared_type(&module.func[0].body, 1), Some(Type::Bool));
    }

    // ─── Group 11: infer — For loop identifier inference ────────────────────────

    #[test]
    fn for_loop_ident_infers_element_type_from_int_array() {
        let for_stmt = HStatement::For {
            ident: make_var("i", None),
            expr: array_expr(vec![int_expr(1), int_expr(2), int_expr(3)]),
            body: Box::new(block(vec![])),
        };
        let body = block(vec![for_stmt]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        assert_eq!(for_ident_type(&module.func[0].body), Some(Type::Int));
    }

    #[test]
    fn for_loop_ident_infers_element_type_from_bool_array() {
        let for_stmt = HStatement::For {
            ident: make_var("flag", None),
            expr: array_expr(vec![bool_expr(true), bool_expr(false)]),
            body: Box::new(block(vec![])),
        };
        let body = block(vec![for_stmt]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        assert_eq!(for_ident_type(&module.func[0].body), Some(Type::Bool));
    }

    #[test]
    fn for_loop_ident_with_explicit_type_is_not_overwritten() {
        let for_stmt = HStatement::For {
            ident: make_var("i", Some(Type::Bool)), // explicit type
            expr: array_expr(vec![int_expr(1), int_expr(2)]),
            body: Box::new(block(vec![])),
        };
        let body = block(vec![for_stmt]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        // The explicit type Bool must be preserved, not overwritten by array element type
        assert_eq!(for_ident_type(&module.func[0].body), Some(Type::Bool));
    }

    #[test]
    fn for_loop_body_variable_gets_element_type_via_loop_ident() {
        // for i in [1,2,3] { let x = i }  →  x gets Type::Int
        let for_stmt = HStatement::For {
            ident: make_var("i", None),
            expr: array_expr(vec![int_expr(1), int_expr(2), int_expr(3)]),
            body: Box::new(block(vec![declare("x", None, Some(var_expr("i")))])),
        };
        let body = block(vec![for_stmt]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);

        // Navigate: body (Block) → For → body (Block) → Declare
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
        let if_stmt = HStatement::If {
            condition: bool_expr(true),
            body: Box::new(block(vec![declare("x", None, Some(int_expr(5)))])),
            else_branch: None,
        };
        let body = block(vec![if_stmt]);
        let mut module = make_module(make_func("main", vec![], body, None));
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
        let if_stmt = HStatement::If {
            condition: bool_expr(true),
            body: Box::new(block(vec![])),
            else_branch: Some(Box::new(block(vec![declare(
                "x",
                None,
                Some(str_expr("hi")),
            )]))),
        };
        let body = block(vec![if_stmt]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);

        if let HStatement::Block { statements, .. } = &module.func[0].body {
            if let HStatement::If {
                else_branch: Some(else_body),
                ..
            } = &statements[0]
            {
                assert_eq!(declared_type(else_body, 0), Some(Type::Str));
                return;
            }
        }
        panic!("Could not navigate to else body");
    }

    #[test]
    fn while_body_variables_are_inferred() {
        let while_stmt = HStatement::While {
            condition: bool_expr(true),
            body: Box::new(block(vec![declare("x", None, Some(bool_expr(false)))])),
        };
        let body = block(vec![while_stmt]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);

        if let HStatement::Block { statements, .. } = &module.func[0].body {
            if let HStatement::While {
                body: while_body, ..
            } = &statements[0]
            {
                assert_eq!(declared_type(while_body, 0), Some(Type::Bool));
                return;
            }
        }
        panic!("Could not navigate to while body");
    }

    #[test]
    fn match_case_arm_variables_are_inferred() {
        let match_stmt = HStatement::Match {
            subject: int_expr(1),
            arms: vec![HMatchArm::Case(
                int_expr(1),
                block(vec![declare("x", None, Some(int_expr(99)))]),
            )],
        };
        let body = block(vec![match_stmt]);
        let mut module = make_module(make_func("main", vec![], body, None));
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
        let match_stmt = HStatement::Match {
            subject: int_expr(1),
            arms: vec![HMatchArm::Else(block(vec![declare(
                "x",
                None,
                Some(str_expr("default")),
            )]))],
        };
        let body = block(vec![match_stmt]);
        let mut module = make_module(make_func("main", vec![], body, None));
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
        let inner = block(vec![declare("x", None, Some(int_expr(42)))]);
        let outer = block(vec![inner]);
        let mut module = make_module(make_func("main", vec![], outer, None));
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
        let helper = make_func(
            "helper",
            vec![],
            block(vec![HStatement::Return(Some(int_expr(1)))]),
            Some(Type::Int),
        );
        let main_body = block(vec![declare(
            "x",
            None,
            Some(call_expr("helper", vec![])),
        )]);
        let main_fn = make_func("main", vec![], main_body, None);

        let mut module = HModule {
            imports: HashSet::new(),
            func: vec![helper, main_fn],
            structs: vec![],
            globals: vec![],
        };
        infer(&mut module);

        // main is at index 1
        assert_eq!(declared_type(&module.func[1].body, 0), Some(Type::Int));
    }

    #[test]
    fn builtin_len_call_is_inferred_as_int() {
        let body = block(vec![declare(
            "n",
            None,
            Some(call_expr("len", vec![var_expr("arr")])),
        )]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), Some(Type::Int));
    }

    #[test]
    fn unknown_function_call_leaves_type_as_none() {
        let body = block(vec![declare(
            "x",
            None,
            Some(call_expr("totally_unknown_fn", vec![])),
        )]);
        let mut module = make_module(make_func("main", vec![], body, None));
        infer(&mut module);
        assert_eq!(declared_type(&module.func[0].body, 0), None);
    }
}
