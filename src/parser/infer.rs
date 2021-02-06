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
                                dbg!(&var);
                                var.ty = infer_expression(&e);
                                dbg!(&var);
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
        _ => None,
    }
}
