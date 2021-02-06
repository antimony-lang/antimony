use super::node_type::*;

/// Try to infer types of variables
pub(super) fn infer(program: &mut Program) -> Result<(), String> {
    // TODO: Fix aweful nesting
    for func in &mut program.func {
        if let Statement::Block(statements) = &mut func.body {
            for statement in statements {
                match statement {
                    Statement::Declare(var, expr) => {
                        if let None = &var.ty {
                            if let Some(e) = expr {
                                var.ty = infer_expression(&e);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn infer_expression(expr: &Expression) -> Option<Type> {
    match expr {
        Expression::Int(_) => Some(Type::Int),
        Expression::Bool(_) => Some(Type::Bool),
        Expression::Str(_) => Some(Type::Str),
        Expression::Array(els) => infer_array(els),
        _ => None,
    }
}

fn infer_array(elements: &Vec<Expression>) -> Option<Type> {
    let types: Vec<Option<Type>> = elements.iter().map(|el| infer_expression(el)).collect();

    // TODO: This approach only relies on the first element.
    // It will not catch that types are possibly inconsistent.
    match types.first().and_then(|ty| ty.to_owned()) {
        Some(ty) => Some(Type::Array(Box::new(ty))),
        None => None,
    }
}
