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

    fn module(funcs: Vec<HFunction>) -> HModule {
        HModule {
            func: funcs,
            structs: vec![],
            globals: vec![],
            imports: HashSet::new(),
        }
    }

    fn func(
        name: &str,
        arguments: Vec<HVariable>,
        body: HStatement,
        ret_type: Option<Type>,
    ) -> HFunction {
        HFunction {
            name: name.to_string(),
            arguments,
            body,
            ret_type,
        }
    }

    fn var(name: &str, ty: Option<Type>) -> HVariable {
        HVariable {
            name: name.to_string(),
            ty,
        }
    }

    fn block(stmts: Vec<HStatement>) -> HStatement {
        HStatement::Block {
            statements: stmts,
            scope: vec![],
        }
    }

    fn declare(name: &str, ty: Option<Type>, value: Option<HExpression>) -> HStatement {
        HStatement::Declare {
            variable: var(name, ty),
            value,
        }
    }

    /// Helper: run inference and return the body statements of the first function
    fn infer_and_get_stmts(m: &mut HModule) -> Vec<HStatement> {
        infer(m);
        match &m.func[0].body {
            HStatement::Block { statements, .. } => statements.clone(),
            other => vec![other.clone()],
        }
    }

    fn get_declared_type(stmt: &HStatement) -> Option<Type> {
        match stmt {
            HStatement::Declare { variable, .. } => variable.ty.clone(),
            _ => panic!("expected Declare statement"),
        }
    }

    #[test]
    fn test_infer_int_literal() {
        let body = block(vec![declare("x", None, Some(HExpression::Int(42)))]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        let stmts = infer_and_get_stmts(&mut m);
        assert_eq!(get_declared_type(&stmts[0]), Some(Type::Int));
    }

    #[test]
    fn test_infer_bool_literal() {
        let body = block(vec![declare("x", None, Some(HExpression::Bool(true)))]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        let stmts = infer_and_get_stmts(&mut m);
        assert_eq!(get_declared_type(&stmts[0]), Some(Type::Bool));
    }

    #[test]
    fn test_infer_str_literal() {
        let body = block(vec![declare(
            "x",
            None,
            Some(HExpression::Str("hello".into())),
        )]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        let stmts = infer_and_get_stmts(&mut m);
        assert_eq!(get_declared_type(&stmts[0]), Some(Type::Str));
    }

    #[test]
    fn test_infer_variable_from_param() {
        let body = block(vec![declare(
            "y",
            None,
            Some(HExpression::Variable("x".into())),
        )]);
        let args = vec![var("x", Some(Type::Int))];
        let mut m = module(vec![func("main", args, body, None)]);
        let stmts = infer_and_get_stmts(&mut m);
        assert_eq!(get_declared_type(&stmts[0]), Some(Type::Int));
    }

    #[test]
    fn test_infer_array_type() {
        let body = block(vec![declare(
            "a",
            None,
            Some(HExpression::Array {
                capacity: 3,
                elements: vec![
                    HExpression::Int(1),
                    HExpression::Int(2),
                    HExpression::Int(3),
                ],
            }),
        )]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        let stmts = infer_and_get_stmts(&mut m);
        assert_eq!(
            get_declared_type(&stmts[0]),
            Some(Type::Array(Box::new(Type::Int), Some(3)))
        );
    }

    #[test]
    fn test_infer_function_call_return_type() {
        let body = block(vec![declare(
            "x",
            None,
            Some(HExpression::FunctionCall {
                fn_name: "foo".into(),
                args: vec![],
            }),
        )]);
        let mut m = module(vec![
            func("foo", vec![], block(vec![]), Some(Type::Int)),
            func("main", vec![], body, None),
        ]);
        infer(&mut m);
        // main is func[1] after adding foo
        let stmts = match &m.func[1].body {
            HStatement::Block { statements, .. } => statements.clone(),
            _ => panic!(),
        };
        assert_eq!(get_declared_type(&stmts[0]), Some(Type::Int));
    }

    #[test]
    fn test_infer_builtin_len() {
        let body = block(vec![declare(
            "x",
            None,
            Some(HExpression::FunctionCall {
                fn_name: "len".into(),
                args: vec![HExpression::Variable("a".into())],
            }),
        )]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        let stmts = infer_and_get_stmts(&mut m);
        assert_eq!(get_declared_type(&stmts[0]), Some(Type::Int));
    }

    #[test]
    fn test_infer_comparison_returns_bool() {
        let body = block(vec![declare(
            "x",
            None,
            Some(HExpression::BinOp {
                lhs: Box::new(HExpression::Int(1)),
                op: HBinOp::Equal,
                rhs: Box::new(HExpression::Int(2)),
            }),
        )]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        let stmts = infer_and_get_stmts(&mut m);
        assert_eq!(get_declared_type(&stmts[0]), Some(Type::Bool));
    }

    #[test]
    fn test_infer_arithmetic_returns_int() {
        let body = block(vec![declare(
            "x",
            None,
            Some(HExpression::BinOp {
                lhs: Box::new(HExpression::Int(1)),
                op: HBinOp::Addition,
                rhs: Box::new(HExpression::Int(2)),
            }),
        )]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        let stmts = infer_and_get_stmts(&mut m);
        assert_eq!(get_declared_type(&stmts[0]), Some(Type::Int));
    }

    #[test]
    fn test_infer_struct_init() {
        let body = block(vec![declare(
            "s",
            None,
            Some(HExpression::StructInitialization {
                name: "Point".into(),
                fields: HashMap::from([("x".to_string(), Box::new(HExpression::Int(1)))]),
            }),
        )]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        let stmts = infer_and_get_stmts(&mut m);
        assert_eq!(
            get_declared_type(&stmts[0]),
            Some(Type::Struct("Point".into()))
        );
    }

    #[test]
    fn test_infer_for_loop_element() {
        let arr_ty = Type::Array(Box::new(Type::Int), Some(3));
        let for_stmt = HStatement::For {
            ident: var("i", None),
            expr: HExpression::Variable("arr".into()),
            body: Box::new(block(vec![])),
        };
        let body = block(vec![declare("arr", Some(arr_ty), None), for_stmt]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        infer(&mut m);
        let stmts = match &m.func[0].body {
            HStatement::Block { statements, .. } => statements.clone(),
            _ => panic!(),
        };
        match &stmts[1] {
            HStatement::For { ident, .. } => {
                assert_eq!(ident.ty, Some(Type::Int));
            }
            _ => panic!("expected For statement"),
        }
    }

    #[test]
    fn test_infer_array_access() {
        let arr_ty = Type::Array(Box::new(Type::Int), Some(3));
        let body = block(vec![
            declare("arr", Some(arr_ty), None),
            declare(
                "x",
                None,
                Some(HExpression::ArrayAccess {
                    name: "arr".into(),
                    index: Box::new(HExpression::Int(0)),
                }),
            ),
        ]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        infer(&mut m);
        let stmts = match &m.func[0].body {
            HStatement::Block { statements, .. } => statements.clone(),
            _ => panic!(),
        };
        assert_eq!(get_declared_type(&stmts[1]), Some(Type::Int));
    }

    #[test]
    fn test_infer_nested_if() {
        let if_body = block(vec![declare("x", None, Some(HExpression::Int(1)))]);
        let else_body = block(vec![declare(
            "y",
            None,
            Some(HExpression::Str("hi".into())),
        )]);
        let if_stmt = HStatement::If {
            condition: HExpression::Bool(true),
            body: Box::new(if_body),
            else_branch: Some(Box::new(else_body)),
        };
        let body = block(vec![if_stmt]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        infer(&mut m);
        let stmts = match &m.func[0].body {
            HStatement::Block { statements, .. } => statements.clone(),
            _ => panic!(),
        };
        match &stmts[0] {
            HStatement::If {
                body, else_branch, ..
            } => {
                let if_stmts = match body.as_ref() {
                    HStatement::Block { statements, .. } => statements,
                    _ => panic!(),
                };
                assert_eq!(get_declared_type(&if_stmts[0]), Some(Type::Int));
                let else_stmts = match else_branch.as_ref().unwrap().as_ref() {
                    HStatement::Block { statements, .. } => statements,
                    _ => panic!(),
                };
                assert_eq!(get_declared_type(&else_stmts[0]), Some(Type::Str));
            }
            _ => panic!("expected If statement"),
        }
    }

    #[test]
    fn test_infer_match_arms() {
        let arm_body = block(vec![declare("x", None, Some(HExpression::Int(1)))]);
        let else_body = block(vec![declare("y", None, Some(HExpression::Bool(false)))]);
        let match_stmt = HStatement::Match {
            subject: HExpression::Int(1),
            arms: vec![
                HMatchArm::Case(HExpression::Int(1), arm_body),
                HMatchArm::Else(else_body),
            ],
        };
        let body = block(vec![match_stmt]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        infer(&mut m);
        let stmts = match &m.func[0].body {
            HStatement::Block { statements, .. } => statements.clone(),
            _ => panic!(),
        };
        match &stmts[0] {
            HStatement::Match { arms, .. } => {
                match &arms[0] {
                    HMatchArm::Case(_, HStatement::Block { statements, .. }) => {
                        assert_eq!(get_declared_type(&statements[0]), Some(Type::Int));
                    }
                    _ => panic!(),
                }
                match &arms[1] {
                    HMatchArm::Else(HStatement::Block { statements, .. }) => {
                        assert_eq!(get_declared_type(&statements[0]), Some(Type::Bool));
                    }
                    _ => panic!(),
                }
            }
            _ => panic!("expected Match statement"),
        }
    }

    #[test]
    fn test_explicit_type_not_overwritten() {
        let body = block(vec![declare(
            "x",
            Some(Type::Str),
            Some(HExpression::Int(42)),
        )]);
        let mut m = module(vec![func("main", vec![], body, None)]);
        let stmts = infer_and_get_stmts(&mut m);
        assert_eq!(get_declared_type(&stmts[0]), Some(Type::Str));
    }
}
