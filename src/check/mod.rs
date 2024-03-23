use crate::ast::{self, AssignOp, BinOp};
use crate::lexer::{self, Position};
use std::collections::HashMap;

pub mod scope;
pub mod types;

pub type Error = lexer::Error;

pub type Result<T> = lexer::Result<T>;

#[derive(Debug)]
pub struct Module {
    pub type_table: types::Table,
    pub scope_table: scope::Table,
    pub functions: Vec<Function>,
    pub structs: Vec<Struct>,
}

struct Context {
    type_table: types::Table,
    scope_table: scope::Table,
}

impl Module {
    pub fn from_ast(module: ast::Module) -> Result<Self> {
        let type_table = types::Table::new();
        let scope_table = scope::Table::new();

        let mut ctx = Context {
            type_table,
            scope_table,
        };
        let scope = ctx.scope_table.add_root();

        // TODO: dependency resolution (!)
        let structs = module
            .structs
            .into_iter()
            .map(|def| Struct::from_ast(&mut ctx, scope, def))
            .collect::<Result<Vec<_>>>()?;

        for func in &module.func {
            scope.insert(
                func.callable.name.clone(),
                ctx.type_table.insert_ast_callable(&func.callable)?,
                &mut ctx.scope_table,
            );
        }

        let functions: Vec<Function> = module
            .func
            .into_iter()
            .map(|func| Function::from_ast(&mut ctx, scope, func))
            .collect::<Result<Vec<_>>>()?;

        // TODO: globals

        Ok(Self {
            type_table: ctx.type_table,
            scope_table: ctx.scope_table,
            functions,
            structs,
        })
    }
}

#[derive(Debug)]
pub struct Struct {
    pub ty: types::Id,
    pub methods: Vec<Method>,
}

impl Struct {
    fn from_ast(ctx: &mut Context, scope: scope::Id, def: ast::StructDef) -> Result<Self> {
        let ty = ctx.type_table.insert_ast_struct(&def)?;
        let methods = def
            .methods
            .into_iter()
            .map(|method| Method::from_ast(ctx, scope, ty, method))
            .collect::<Result<Vec<_>>>()?;

        Ok(Struct { ty, methods })
    }
}

#[derive(Debug)]
pub struct Callable {
    pub pos: Position,
    pub scope: scope::Id,
    pub name: String,
    pub parameters: Vec<scope::VariableId>,
    pub return_type: types::Id,
}

#[derive(Debug)]
pub struct Function {
    pub callable: Callable,
    pub body: Option<Statement>,
}

impl Function {
    fn from_ast(ctx: &mut Context, scope: scope::Id, func: ast::Function) -> Result<Self> {
        let callable = func.callable;
        let return_type = match callable.ret_type {
            Some(ty) => ctx.type_table.insert_ast_type(&ty)?,
            None => ctx.type_table.void,
        };
        let scope = scope.push_function(return_type, &mut ctx.scope_table);
        let parameters = callable
            .arguments
            .into_iter()
            .map(|param| {
                let ty = ctx.type_table.insert_ast_type(&param.ty)?;
                Ok(scope.insert(param.name, ty, &mut ctx.scope_table))
            })
            .collect::<Result<Vec<_>>>()?;
        let body = func
            .body
            .map(|body| Statement::from_ast(ctx, scope, body))
            .transpose()?;

        Ok(Self {
            callable: Callable {
                pos: callable.pos,
                scope,
                name: callable.name,
                parameters,
                return_type,
            },
            body,
        })
    }
}

#[derive(Debug)]
pub struct Method {
    pub callable: Callable,
    pub self_parameter: scope::VariableId,
    pub body: Statement,
}

impl Method {
    fn from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        struct_type: types::Id,
        method: ast::Method,
    ) -> Result<Self> {
        let callable = method.callable;
        let return_type = match callable.ret_type {
            Some(ty) => ctx.type_table.insert_ast_type(&ty)?,
            None => ctx.type_table.void,
        };
        let scope = scope.push_function(return_type, &mut ctx.scope_table);
        let self_parameter = scope.insert("self".into(), struct_type, &mut ctx.scope_table);
        let parameters = callable
            .arguments
            .into_iter()
            .map(|param| {
                let ty = ctx.type_table.insert_ast_type(&param.ty)?;
                Ok(scope.insert(param.name, ty, &mut ctx.scope_table))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            callable: Callable {
                pos: callable.pos,
                scope,
                name: callable.name,
                parameters,
                return_type,
            },
            self_parameter,
            body: Statement::from_ast(ctx, scope, method.body)?,
        })
    }
}

#[derive(Debug)]
pub struct Statement {
    pub pos: Position,
    pub kind: StatementKind,
}

#[derive(Debug)]
pub enum StatementKind {
    Block {
        statements: Vec<Statement>,
        scope: scope::Id,
    },
    Declare {
        variable: scope::VariableId,
        value: Option<Expression>,
    },
    Assign {
        lhs: Expression,
        rhs: Expression,
    },
    Return(Option<Expression>),
    If {
        condition: Expression,
        body: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        scope: scope::Id,
        condition: Expression,
        body: Box<Statement>,
    },
    For {
        scope: scope::Id,
        variable: scope::VariableId,
        expr: Expression,
        body: Box<Statement>,
    },
    Break(scope::Id),
    Continue(scope::Id),
    Exp(Expression),
}

#[derive(Debug)]
pub struct MatchArm {
    pub condition: Expression,
    pub body: Statement,
}

impl Statement {
    fn from_ast(ctx: &mut Context, scope: scope::Id, stmt: ast::Statement) -> Result<Self> {
        let kind = match stmt.kind {
            ast::StatementKind::Block { statements, .. } => {
                Self::block_from_ast(ctx, scope, statements)
            }
            ast::StatementKind::Declare { variable, value } => {
                Self::declare_from_ast(ctx, scope, variable, value)
            }
            ast::StatementKind::Assign { lhs, op, rhs } => {
                Self::assign_from_ast(ctx, scope, stmt.pos, *lhs, op, *rhs)
            }
            ast::StatementKind::Return(value) => Self::return_from_ast(ctx, scope, stmt.pos, value),
            ast::StatementKind::If {
                condition,
                body,
                else_branch,
            } => Self::if_from_ast(ctx, scope, condition, *body, else_branch),
            ast::StatementKind::Match { subject, arms } => {
                Self::match_from_ast(ctx, scope, stmt.pos, subject, arms)
            }
            ast::StatementKind::While { condition, body } => {
                Self::while_from_ast(ctx, scope, stmt.pos, condition, *body)
            }
            ast::StatementKind::For { ident, expr, body } => {
                Self::for_from_ast(ctx, scope, ident, expr, *body)
            }
            ast::StatementKind::Break => Self::break_from_ast(ctx, scope, stmt.pos),
            ast::StatementKind::Continue => Self::continue_from_ast(ctx, scope, stmt.pos),
            ast::StatementKind::Exp(expr) => {
                let expr = Expression::from_ast(ctx, scope, expr)?;
                Ok(StatementKind::Exp(expr))
            }
        }?;
        Ok(Statement {
            pos: stmt.pos,
            kind,
        })
    }

    fn block_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        statements: Vec<ast::Statement>,
    ) -> Result<StatementKind> {
        let scope = scope.push(&mut ctx.scope_table);
        let statements = statements
            .into_iter()
            .map(|stmt| Statement::from_ast(ctx, scope, stmt))
            .collect::<Result<Vec<_>>>()?;

        Ok(StatementKind::Block { scope, statements })
    }

    fn declare_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        variable: ast::Variable,
        value: Option<ast::Expression>,
    ) -> Result<StatementKind> {
        let value = match value {
            Some(value) => Some(Expression::from_ast(ctx, scope, value)?),
            None => None,
        };
        let ty = match variable.ty {
            Some(ty) => ctx.type_table.insert_ast_type(&ty)?,
            // The parser should ensure that an expression is available for
            // declarations without an explicit type
            None => value.as_ref().unwrap().result,
        };

        if let Some(ref value) = value {
            if !ctx.type_table.assignable(ty, value.result) {
                return Err(Error::new(
                    value.pos,
                    "Initializer is not assignable to variable type".to_owned(),
                ));
            }
        }

        let variable = scope.insert(variable.name, ty, &mut ctx.scope_table);

        Ok(StatementKind::Declare { variable, value })
    }

    fn assign_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        lhs: ast::Expression,
        op: AssignOp,
        rhs: ast::Expression,
    ) -> Result<StatementKind> {
        let lhs = Expression::from_ast(ctx, scope, lhs)?;
        let rhs = Expression::from_ast(ctx, scope, rhs)?;
        match lhs.kind {
            ExpressionKind::Variable(..)
            | ExpressionKind::FieldAccess { .. }
            | ExpressionKind::ArrayAccess { .. } => {}
            _ => {
                return Err(Error::new(
                    lhs.pos,
                    "Left side of assignment must be a variable, field access or array access"
                        .to_owned(),
                ));
            }
        }
        if !ctx.type_table.assignable(lhs.result, rhs.result) {
            return Err(Error::new(
                rhs.pos,
                "Value is not assignable to variable or field type".to_owned(),
            ));
        }
        match op {
            AssignOp::Set => {}
            AssignOp::Add => {
                if lhs.result == ctx.type_table.int {
                    // Addition variant
                    if rhs.result != ctx.type_table.int {
                        return Err(Error::new(
                            pos,
                            "Could not add <lhs type> and int".to_owned(),
                        ));
                    }
                } else if lhs.result == ctx.type_table.string {
                    // Concatenation variant
                    if !(rhs.result == ctx.type_table.int
                        || rhs.result == ctx.type_table.boolean
                        || rhs.result == ctx.type_table.string)
                    {
                        return Err(Error::new(
                            pos,
                            "Could not concatenate 'string' and <rhs type>".to_owned(),
                        ));
                    }
                } else {
                    return Err(Error::new(
                        pos,
                        format!("Could not apply {op:?} to those types"),
                    ));
                }
            }
            AssignOp::Subtract | AssignOp::Multiply | AssignOp::Divide => {
                if rhs.result != ctx.type_table.int {
                    return Err(Error::new(
                        pos,
                        format!("Right side of {op:?} must be an int"),
                    ));
                }
            }
        }

        Ok(StatementKind::Assign { lhs, rhs })
    }

    fn return_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        value: Option<ast::Expression>,
    ) -> Result<StatementKind> {
        let value = match value {
            Some(value) => Some(Expression::from_ast(ctx, scope, value)?),
            None => None,
        };

        let return_type = scope
            .return_type(&ctx.scope_table)
            .ok_or_else(|| Error::new(pos, "'return' used outside of a function?".to_owned()))?;

        match value {
            Some(ref value) => {
                if !ctx.type_table.assignable(return_type, value.result) {
                    return Err(Error::new(
                        value.pos,
                        "Value is not assignable to function return type".to_owned(),
                    ));
                }
            }
            None => {
                if return_type != ctx.type_table.void {
                    return Err(Error::new(pos, "Must return a value".to_owned()));
                }
            }
        }

        Ok(StatementKind::Return(value))
    }

    fn if_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        condition: ast::Expression,
        body: ast::Statement,
        else_branch: Option<Box<ast::Statement>>,
    ) -> Result<StatementKind> {
        let condition = Expression::from_ast(ctx, scope, condition)?;
        if condition.result != ctx.type_table.boolean {
            return Err(Error::new(
                condition.pos,
                "Condition must be a 'bool'".to_owned(),
            ));
        }
        let body = Box::new(Statement::from_ast(ctx, scope, body)?);
        let else_branch = match else_branch {
            Some(stmt) => Some(Box::new(Statement::from_ast(ctx, scope, *stmt)?)),
            None => None,
        };

        Ok(StatementKind::If {
            condition,
            body,
            else_branch,
        })
    }

    fn match_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        subject: ast::Expression,
        ast_arms: Vec<ast::MatchArm>,
    ) -> Result<StatementKind> {
        let subject = Expression::from_ast(ctx, scope, subject)?;
        let mut else_branch: Option<Box<Statement>> = None;
        let mut arms: Vec<MatchArm> = Vec::new();
        for arm in ast_arms {
            match arm {
                ast::MatchArm::Case(condition, body) => {
                    let condition = Expression::from_ast(ctx, scope, condition)?;
                    if subject.result != condition.result {
                        // TODO: location for match arms
                        return Err(Error::new(
                            pos,
                            "Condition does not match subject type".to_owned(),
                        ));
                    }
                    let body = Statement::from_ast(ctx, scope, body)?;
                    arms.push(MatchArm { condition, body });
                }
                ast::MatchArm::Else(body) => {
                    assert!(else_branch.is_none()); // Enforced by the parser
                    let body = Box::new(Statement::from_ast(ctx, scope, body)?);
                    else_branch = Some(body);
                }
            }
        }

        Ok(StatementKind::Match {
            subject,
            arms,
            else_branch,
        })
    }

    fn while_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        condition: ast::Expression,
        body: ast::Statement,
    ) -> Result<StatementKind> {
        let condition = Expression::from_ast(ctx, scope, condition)?;
        if condition.result != ctx.type_table.boolean {
            return Err(Error::new(pos, "Condition must be a 'bool'".to_owned()));
        }
        let scope = scope.push_loop(&mut ctx.scope_table);
        let body = Box::new(Statement::from_ast(ctx, scope, body)?);

        Ok(StatementKind::While {
            scope,
            condition,
            body,
        })
    }

    fn for_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        variable: ast::Variable,
        expr: ast::Expression,
        body: ast::Statement,
    ) -> Result<StatementKind> {
        let expr = Expression::from_ast(ctx, scope, expr)?;
        let scope = scope.push_loop(&mut ctx.scope_table);
        let variable = if let types::Repr::Array { member_type, .. } =
            expr.result.dealiased_repr(&ctx.type_table)
        {
            scope.insert(variable.name, *member_type, &mut ctx.scope_table)
        } else {
            return Err(Error::new(
                expr.pos,
                "'for' must iterate over an array".to_owned(),
            ));
        };
        let body = Box::new(Statement::from_ast(ctx, scope, body)?);

        Ok(StatementKind::For {
            scope,
            variable,
            expr,
            body,
        })
    }

    fn break_from_ast(ctx: &mut Context, scope: scope::Id, pos: Position) -> Result<StatementKind> {
        match scope.nearest_loop(&ctx.scope_table) {
            Some(loop_id) => Ok(StatementKind::Break(loop_id)),
            None => Err(Error::new(
                pos,
                "'break' must be used within a loop".to_owned(),
            )),
        }
    }

    fn continue_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
    ) -> Result<StatementKind> {
        match scope.nearest_loop(&ctx.scope_table) {
            Some(loop_id) => Ok(StatementKind::Continue(loop_id)),
            None => Err(Error::new(
                pos,
                "'continue' must be used within a loop".to_owned(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct Expression {
    pub pos: Position,
    pub kind: ExpressionKind,
    pub result: types::Id,
}

#[derive(Debug)]
pub enum ExpressionKind {
    Int(usize),
    Str(String),
    Bool(bool),
    Array(Vec<Expression>),
    Variable(scope::VariableId),
    FunctionCall {
        fn_name: String,
        args: Vec<Expression>,
    },
    MethodCall {
        method: String, // TODO: some kind of method identifier?
        instance: Box<Expression>,
        args: Vec<Expression>,
    },
    BinOp {
        lhs: Box<Expression>,
        op: BinOp,
        rhs: Box<Expression>,
    },
    ArrayAccess {
        target: Box<Expression>,
        index: Box<Expression>,
    },
    FieldAccess {
        target: Box<Expression>,
        field: String, // TODO: some kind of field identifier?
    },
    StructInitialization {
        ty: types::Id,
        fields: HashMap<String, Expression>,
    },
}

impl Expression {
    fn from_ast(ctx: &mut Context, scope: scope::Id, aexpr: ast::Expression) -> Result<Self> {
        match aexpr.kind {
            ast::ExpressionKind::Int(value) => Ok(Expression {
                pos: aexpr.pos,
                kind: ExpressionKind::Int(value),
                result: ctx.type_table.int,
            }),
            ast::ExpressionKind::Str(value) => Ok(Expression {
                pos: aexpr.pos,
                kind: ExpressionKind::Str(value),
                result: ctx.type_table.string,
            }),
            ast::ExpressionKind::Bool(value) => Ok(Expression {
                pos: aexpr.pos,
                kind: ExpressionKind::Bool(value),
                result: ctx.type_table.boolean,
            }),
            ast::ExpressionKind::Array(elements) => {
                Self::array_from_ast(ctx, scope, aexpr.pos, elements)
            }
            ast::ExpressionKind::Variable(name) => {
                Self::variable_from_ast(ctx, scope, aexpr.pos, name)
            }
            ast::ExpressionKind::Selff => Self::self_from_ast(ctx, scope, aexpr.pos),
            ast::ExpressionKind::FunctionCall { expr, args } => {
                Self::call_from_ast(ctx, scope, aexpr.pos, *expr, args)
            }
            ast::ExpressionKind::BinOp { lhs, op, rhs } => {
                Self::binop_from_ast(ctx, scope, aexpr.pos, *lhs, op, *rhs)
            }
            ast::ExpressionKind::ArrayAccess { expr, index } => {
                Self::array_access_from_ast(ctx, scope, aexpr.pos, *expr, *index)
            }
            ast::ExpressionKind::FieldAccess { expr, field } => {
                Self::field_access_from_ast(ctx, scope, aexpr.pos, *expr, field)
            }
            ast::ExpressionKind::StructInitialization { name, fields } => {
                Self::struct_init_from_ast(ctx, scope, aexpr.pos, name, fields)
            }
        }
    }

    fn variable_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        name: String,
    ) -> Result<Self> {
        let (id, ty) = scope
            .lookup(&name, &ctx.scope_table)
            .ok_or_else(|| Error::new(pos, format!("Undefined variable '{name}'")))?;

        Ok(Expression {
            pos,
            kind: ExpressionKind::Variable(id),
            result: ty,
        })
    }

    fn self_from_ast(ctx: &mut Context, scope: scope::Id, pos: Position) -> Result<Self> {
        let (id, ty) = scope.lookup("self", &ctx.scope_table).ok_or_else(|| {
            Error::new(pos, "'self' is not allowed outside of a method".to_owned())
        })?;

        Ok(Expression {
            pos,
            kind: ExpressionKind::Variable(id),
            result: ty,
        })
    }

    fn call_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        target: ast::Expression,
        args: Vec<ast::Expression>,
    ) -> Result<Self> {
        match target.kind {
            ast::ExpressionKind::Variable(name) => {
                Self::function_call_from_ast(ctx, scope, pos, name, args)
            }
            ast::ExpressionKind::FieldAccess { expr, field } => {
                let instance = Expression::from_ast(ctx, scope, *expr)?;
                Self::method_call_from_ast(ctx, scope, pos, instance, field, args)
            }
            // TODO: better error message
            _ => Err(Error::new(pos, "Invalid call".to_owned())),
        }
    }

    fn function_call_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        fn_name: String,
        args: Vec<ast::Expression>,
    ) -> Result<Self> {
        let (_, fn_type) = scope
            .lookup(&fn_name, &ctx.scope_table)
            .ok_or_else(|| Error::new(pos, format!("Undefined function '{fn_name}'")))?;
        let (args, return_type) = Self::check_call(ctx, scope, pos, fn_type, args)?;

        Ok(Expression {
            pos,
            kind: ExpressionKind::FunctionCall { fn_name, args },
            result: return_type,
        })
    }

    fn method_call_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        instance: Expression,
        method: String,
        args: Vec<ast::Expression>,
    ) -> Result<Self> {
        let ty = instance.result.dealiased_repr(&ctx.type_table);

        let methods = if let types::Repr::Struct { methods, .. } = ty {
            methods
        } else {
            return Err(Error::new(
                pos,
                "Cannot call a method of a non-struct type".to_owned(),
            ));
        };

        let fn_type = methods
            .get(&method)
            .ok_or_else(|| Error::new(pos, format!("No such method '{method}'")))?;

        let (args, return_type) = Self::check_call(ctx, scope, pos, *fn_type, args)?;

        Ok(Expression {
            pos,
            kind: ExpressionKind::MethodCall {
                instance: Box::new(instance),
                method,
                args,
            },
            result: return_type,
        })
    }

    fn check_call(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        fn_type: types::Id,
        args: Vec<ast::Expression>,
    ) -> Result<(Vec<Expression>, types::Id)> {
        let ty = fn_type.repr(&ctx.type_table);

        let (parameters, return_type) = if let types::Repr::Function {
            parameters,
            return_type,
        } = ty
        {
            // FIXME: this clone should not be necessary
            (parameters.clone(), *return_type)
        } else {
            return Err(Error::new(pos, "Cannot call non-function type".to_owned()));
        };

        if args.len() < parameters.len() {
            return Err(Error::new(
                pos,
                format!(
                    "Not enough arguments: expected {}, got {}",
                    parameters.len(),
                    args.len()
                ),
            ));
        }
        if args.len() > parameters.len() {
            return Err(Error::new(
                pos,
                format!(
                    "Too many arguments: expected {}, got {}",
                    parameters.len(),
                    args.len()
                ),
            ));
        }

        let args = std::iter::zip(parameters, args)
            .map(|(param_type, arg)| {
                let arg = Expression::from_ast(ctx, scope, arg)?;
                if !ctx.type_table.assignable(param_type, arg.result) {
                    return Err(Error::new(
                        arg.pos,
                        "Argument is not assignable to parameter type".to_owned(),
                    ));
                }
                Ok(arg)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok((args, return_type))
    }

    fn array_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        elements: Vec<ast::Expression>,
    ) -> Result<Self> {
        let mut member_type: Option<types::Id> = None;
        let elements = elements
            .into_iter()
            .map(|elem| {
                let elem = Expression::from_ast(ctx, scope, elem)?;
                if let Some(previous_type) = member_type {
                    if elem.result != previous_type {
                        return Err(Error::new(
                            elem.pos,
                            "Array elements must have a uniform type".to_owned(),
                        ));
                    }
                } else {
                    member_type = Some(elem.result);
                }
                Ok(elem)
            })
            .collect::<Result<Vec<_>>>()?;
        let member_type =
            member_type.ok_or_else(|| Error::new(pos, "TODO: empty arrays".to_owned()))?;
        let array_type = ctx
            .type_table
            .insert_array(member_type, elements.len())
            .unwrap();

        Ok(Expression {
            pos,
            kind: ExpressionKind::Array(elements),
            result: array_type,
        })
    }

    fn binop_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        lhs: ast::Expression,
        op: BinOp,
        rhs: ast::Expression,
    ) -> Result<Self> {
        let lhs = Expression::from_ast(ctx, scope, lhs)?;
        let rhs = Expression::from_ast(ctx, scope, rhs)?;
        let result = match op {
            BinOp::Equal | BinOp::NotEqual => {
                if lhs.result != rhs.result {
                    return Err(Error::new(
                        pos,
                        "Cannot compare values of different types".to_owned(),
                    ));
                }
                ctx.type_table.boolean
            }
            BinOp::And | BinOp::Or => {
                if lhs.result != ctx.type_table.boolean {
                    return Err(Error::new(
                        pos,
                        format!("Left side of {op:?} must be a bool"),
                    ));
                }
                if rhs.result != ctx.type_table.boolean {
                    return Err(Error::new(
                        pos,
                        format!("Right side of {op:?} must be a bool"),
                    ));
                }
                ctx.type_table.boolean
            }
            BinOp::Addition => {
                if lhs.result == ctx.type_table.int {
                    // Addition variant
                    if rhs.result != ctx.type_table.int {
                        return Err(Error::new(
                            pos,
                            "Could not add <lhs type> and int".to_owned(),
                        ));
                    }
                    ctx.type_table.int
                } else if lhs.result == ctx.type_table.string {
                    // Concatenation variant
                    if !(rhs.result == ctx.type_table.int
                        || rhs.result == ctx.type_table.boolean
                        || rhs.result == ctx.type_table.string)
                    {
                        return Err(Error::new(
                            pos,
                            "Could not concatenate 'string' and <rhs type>".to_owned(),
                        ));
                    }
                    ctx.type_table.string
                } else {
                    return Err(Error::new(
                        pos,
                        format!("Could not apply {op:?} to those types"),
                    ));
                }
            }
            // Comparisons
            _ => {
                if lhs.result != ctx.type_table.int {
                    return Err(Error::new(
                        pos,
                        format!("Left side of {op:?} must be an int"),
                    ));
                }
                if rhs.result != ctx.type_table.int {
                    return Err(Error::new(
                        pos,
                        format!("Right side of {op:?} must be an int"),
                    ));
                }
                ctx.type_table.int
            }
        };
        Ok(Expression {
            pos,
            kind: ExpressionKind::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            },
            result,
        })
    }

    fn array_access_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        target: ast::Expression,
        index: ast::Expression,
    ) -> Result<Self> {
        let target = Box::new(Expression::from_ast(ctx, scope, target)?);
        let index = Box::new(Expression::from_ast(ctx, scope, index)?);
        let member_type = if let types::Repr::Array { member_type, .. } =
            target.result.dealiased_repr(&ctx.type_table)
        {
            *member_type
        } else {
            return Err(Error::new(
                pos,
                "Cannot index a value of non-array type".to_owned(),
            ));
        };
        if index.result != ctx.type_table.int {
            return Err(Error::new(pos, "Index must be an 'int'".to_owned()));
        }

        Ok(Expression {
            pos,
            kind: ExpressionKind::ArrayAccess { target, index },
            result: member_type,
        })
    }

    fn field_access_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        target: ast::Expression,
        field: String,
    ) -> Result<Self> {
        let target = Box::new(Expression::from_ast(ctx, scope, target)?);
        let struct_repr = target.result.dealiased_repr(&ctx.type_table);
        let fields = if let types::Repr::Struct { fields, .. } = &struct_repr {
            fields
        } else {
            return Err(Error::new(
                pos,
                "Cannot get a field of non-struct type".to_owned(),
            ));
        };

        let field_type = fields
            .get(&field)
            .ok_or_else(|| Error::new(pos, format!("No such field '{field}'")))?
            .ty;

        Ok(Expression {
            pos,
            kind: ExpressionKind::FieldAccess { target, field },
            result: field_type,
        })
    }

    fn struct_init_from_ast(
        ctx: &mut Context,
        scope: scope::Id,
        pos: Position,
        struct_name: String,
        fields: HashMap<String, Box<ast::Expression>>,
    ) -> Result<Self> {
        let ty = ctx
            .type_table
            .by_name(&struct_name)
            .ok_or_else(|| Error::new(pos, format!("No such struct '{struct_name}'")))?;
        let repr = ty.dealiased_repr(&ctx.type_table);
        let available_fields = if let types::Repr::Struct { fields, .. } = &repr {
            fields
        } else {
            panic!("Unexpected non-struct named type")
        };

        // Borrow checker shenanigan
        let types = fields
            .keys()
            .map(|name| {
                let types::StructField { ty: field_type, .. } = available_fields
                    .get(name)
                    .ok_or_else(|| Error::new(pos, format!("Unknown field '{name}'")))?;
                Ok((name.clone(), *field_type))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let fields = fields
            .into_iter()
            .map(|(name, value)| {
                let field_type = types.get(&name).unwrap();
                let value = Expression::from_ast(ctx, scope, *value)?;
                if !ctx.type_table.assignable(*field_type, value.result) {
                    return Err(Error::new(
                        value.pos,
                        "Initializer is not assignable to field type".to_owned(),
                    ));
                }
                Ok((name, value))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(Expression {
            pos,
            kind: ExpressionKind::StructInitialization { ty, fields },
            result: ty,
        })
    }
}
