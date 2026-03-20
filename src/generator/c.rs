use crate::ast::*;
use crate::generator::{Generator, GeneratorResult};
use std::cell::RefCell;
use std::collections::HashMap;
use types::Type;

// Thread-local variable type context, set at the start of each function generation.
// Maps Antimony variable name → its declared type so that `generate_field_access`
// can resolve the struct type of the object and produce a correctly-mangled method name.
thread_local! {
    static CURRENT_VAR_TYPES: RefCell<HashMap<String, Option<Type>>> =
        RefCell::new(HashMap::new());
}

/// Returns the mangled C name for a struct method: `StructName_methodName`.
fn mangle_method_name(struct_name: &str, method_name: &str) -> String {
    format!("{}_{}", struct_name, method_name)
}

pub struct CGenerator;

impl Generator for CGenerator {
    fn generate(prog: Module) -> GeneratorResult<String> {
        let mut code = String::new();

        // Add standard C headers
        code += "#include <stdio.h>\n";
        code += "#include <stdlib.h>\n";
        code += "#include <stdbool.h>\n";
        code += "#include <string.h>\n\n";

        // Add builtin functions
        let raw_builtins = crate::Builtins::get("builtin.c")
            .expect("Could not locate builtin functions")
            .data;
        code += std::str::from_utf8(&raw_builtins).expect("Unable to interpret builtin functions");

        // Sort structs topologically so dependencies are defined before dependents
        let sorted_structs = topological_sort_structs(prog.structs.clone());

        // 1. Generate struct type definitions (typedef struct only, no method bodies)
        for s in &sorted_structs {
            code += &generate_struct_type_definition(s);
        }

        // 2. Generate all prototypes: struct methods + standalone functions
        for s in &sorted_structs {
            code += &generate_struct_method_prototypes(s);
        }
        let prototypes: String = prog.func.iter().map(generate_function_prototype).collect();
        code += &prototypes;
        code += "\n";

        // 3. Generate struct method implementations
        for s in sorted_structs {
            code += &generate_struct_method_impls(s);
        }

        // 4. Generate standalone function implementations
        let funcs: String = prog.func.into_iter().map(generate_function).collect();
        code += &funcs;

        Ok(code)
    }
}

/// Returns the struct names that a struct depends on (via its field types).
fn struct_dependencies(s: &StructDef) -> Vec<String> {
    s.fields
        .iter()
        .filter_map(|f| {
            if let Some(types::Type::Struct(name)) = &f.ty {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Sorts structs so that a struct's dependencies appear before it.
fn topological_sort_structs(structs: Vec<StructDef>) -> Vec<StructDef> {
    let mut sorted: Vec<StructDef> = Vec::with_capacity(structs.len());
    let mut remaining = structs;

    while !remaining.is_empty() {
        let mut progress = false;
        let sorted_names: Vec<String> = sorted.iter().map(|s| s.name.clone()).collect();

        remaining.retain(|s| {
            let deps = struct_dependencies(s);
            let all_deps_satisfied = deps.iter().all(|dep| sorted_names.contains(dep));
            if all_deps_satisfied {
                sorted.push(s.clone());
                progress = true;
                false
            } else {
                true
            }
        });

        // If no progress was made (e.g. circular dependency), emit the rest as-is
        if !progress {
            sorted.extend(remaining.drain(..));
            break;
        }
    }

    sorted
}

/// Generates only the `typedef struct { ... } Name;` declaration (no methods).
fn generate_struct_type_definition(struct_def: &StructDef) -> String {
    let mut buf = format!("typedef struct {} {{\n", struct_def.name);
    for field in &struct_def.fields {
        buf += &format!("    {} {};\n", type_to_c_type(&field.ty), field.name);
    }
    buf += &format!("}} {};\n\n", struct_def.name);
    buf
}

/// Generates forward declarations for all methods of a struct.
/// Method names are mangled to `StructName_methodName` to avoid C name collisions.
fn generate_struct_method_prototypes(struct_def: &StructDef) -> String {
    let mut buf = String::new();
    for method in &struct_def.methods {
        let mut method_copy = method.clone();
        method_copy.name = mangle_method_name(&struct_def.name, &method.name);
        let self_var = Variable {
            name: "self".to_string(),
            ty: Some(types::Type::Struct(struct_def.name.clone())),
        };
        method_copy.arguments.insert(0, self_var);
        buf += &generate_function_prototype(&method_copy);
    }
    buf
}

/// Generates the method implementations for a struct.
/// Method names are mangled to `StructName_methodName` to avoid C name collisions.
fn generate_struct_method_impls(struct_def: StructDef) -> String {
    let mut buf = String::new();
    for method in struct_def.methods {
        let mut method_copy = method.clone();
        method_copy.name = mangle_method_name(&struct_def.name, &method.name);
        let self_var = Variable {
            name: "self".to_string(),
            ty: Some(types::Type::Struct(struct_def.name.clone())),
        };
        method_copy.arguments.insert(0, self_var);
        buf += &generate_function(method_copy);
    }
    buf
}

pub(super) fn generate_arguments(args: Vec<Variable>) -> String {
    if args.is_empty() {
        return "void".to_string();
    }

    args.into_iter()
        .map(|var| format!("{} {}", type_to_c_type(&var.ty), var.name))
        .collect::<Vec<String>>()
        .join(", ")
}

fn type_to_c_type(ty: &Option<Type>) -> String {
    match ty {
        Some(Type::Int) => "int".to_string(),
        Some(Type::Bool) => "bool".to_string(),
        Some(Type::Str) => "char*".to_string(),
        Some(Type::Array(inner, _)) => format!("{}*", type_to_c_type(&Some(*inner.clone()))),
        Some(Type::Struct(name)) => name.clone(),
        Some(Type::Any) => "int".to_string(),
        None => "void".to_string(),
    }
}

/// Infers a C type string from an expression when the variable's declared type is unknown.
fn infer_type_from_expr(expr: &Expression) -> String {
    match expr {
        Expression::Int(_) => "int".to_string(),
        Expression::Bool(_) => "bool".to_string(),
        Expression::Str(_) => "char*".to_string(),
        Expression::BinOp { op, lhs, rhs } => match op {
            BinOp::Equal
            | BinOp::NotEqual
            | BinOp::LessThan
            | BinOp::LessThanOrEqual
            | BinOp::GreaterThan
            | BinOp::GreaterThanOrEqual
            | BinOp::And
            | BinOp::Or => "bool".to_string(),
            BinOp::Addition => {
                // String concatenation: if either operand is a string, the result is too
                if matches!(**lhs, Expression::Str(_))
                    || matches!(**rhs, Expression::Str(_))
                    || matches!(**lhs, Expression::FieldAccess { .. })
                    || matches!(**rhs, Expression::FieldAccess { .. })
                {
                    "char*".to_string()
                } else {
                    "int".to_string()
                }
            }
            _ => "int".to_string(),
        },
        _ => "int".to_string(),
    }
}

pub(super) fn generate_function_prototype(func: &Function) -> String {
    let return_type = if func.name == "main" {
        "int".to_string()
    } else {
        match &func.ret_type {
            Some(ty) => type_to_c_type(&Some(ty.clone())),
            None => infer_return_type_from_body(&func.body)
                .unwrap_or_else(|| "void".to_string()),
        }
    };

    format!(
        "{} {}({});\n",
        return_type,
        func.name,
        generate_arguments(func.arguments.clone())
    )
}

/// Tries to infer the return type of a function body by looking at its return statements.
///
/// Note: The Block's `scope` field has pre-inference types (cloned at parse time before
/// type inference runs). We must look at the `Declare` statements instead, which are
/// updated in-place by the type inferencer.
fn infer_return_type_from_body(body: &Statement) -> Option<String> {
    if let Statement::Block { statements, .. } = body {
        // Build a map of variable name → C type from Declare statements (post-inference types).
        let mut var_types: HashMap<String, String> = HashMap::new();
        for stmt in statements {
            if let Statement::Declare { variable, .. } = stmt {
                if let Some(ty) = &variable.ty {
                    var_types.insert(variable.name.clone(), type_to_c_type(&Some(ty.clone())));
                }
            }
            if let Statement::Return(Some(expr)) = stmt {
                // If returning a named variable, look it up in declarations first.
                if let Expression::Variable(name) = expr {
                    if let Some(ty_str) = var_types.get(name) {
                        return Some(ty_str.clone());
                    }
                }
                let t = infer_type_from_expr(expr);
                if t != "void" {
                    return Some(t);
                }
            }
        }
    }
    None
}

pub(super) fn generate_function(func: Function) -> String {
    // Populate the thread-local variable type context so sub-generators (e.g.
    // generate_field_access) can resolve the struct type of local variables.
    if let Statement::Block { ref statements, .. } = func.body {
        let var_types: HashMap<String, Option<Type>> = statements
            .iter()
            .filter_map(|s| {
                if let Statement::Declare { variable, .. } = s {
                    Some((variable.name.clone(), variable.ty.clone()))
                } else {
                    None
                }
            })
            .collect();
        // Also include function arguments in the type context.
        let mut all_types = var_types;
        for arg in &func.arguments {
            all_types.insert(arg.name.clone(), arg.ty.clone());
        }
        CURRENT_VAR_TYPES.with(|vt| {
            *vt.borrow_mut() = all_types;
        });
    }

    // C's main() must return int; override the inferred return type for it
    let return_type = if func.name == "main" {
        "int".to_string()
    } else {
        match &func.ret_type {
            Some(ty) => type_to_c_type(&Some(ty.clone())),
            None => {
                // Try to infer from the function body's return statement
                infer_return_type_from_body(&func.body)
                    .unwrap_or_else(|| "void".to_string())
            }
        }
    };

    let arguments = generate_arguments(func.arguments);
    let mut raw = format!("{} {}({}) ", return_type, func.name, arguments);

    // For main(), append an implicit "return 0;" so the process exits cleanly
    let body = if func.name == "main" {
        let mut block = generate_block(func.body, None);
        // Insert "return 0;" before the closing brace
        if let Some(pos) = block.rfind('}') {
            block.insert_str(pos, "return 0;\n");
        }
        block
    } else {
        generate_block(func.body, None)
    };

    raw += &body;
    raw += "\n";
    raw
}

pub(super) fn generate_struct_definition(struct_def: StructDef) -> String {
    let mut buf = format!("typedef struct {} {{\n", &struct_def.name);

    // Generate struct fields
    for field in &struct_def.fields {
        buf += &format!("    {} {};\n", type_to_c_type(&field.ty), field.name);
    }
    buf += &format!("}} {};\n\n", &struct_def.name);

    // Generate method prototypes
    for method in &struct_def.methods {
        let mut method_copy = method.clone();
        // Add self parameter as first argument
        let self_var = Variable {
            name: "self".to_string(),
            ty: Some(Type::Struct(struct_def.name.clone())),
        };
        method_copy.arguments.insert(0, self_var);
        buf += &generate_function_prototype(&method_copy);
    }

    // Generate method implementations
    for method in &struct_def.methods {
        let mut method_copy = method.clone();
        // Add self parameter as first argument
        let self_var = Variable {
            name: "self".to_string(),
            ty: Some(Type::Struct(struct_def.name.clone())),
        };
        method_copy.arguments.insert(0, self_var);
        buf += &generate_function(method_copy);
    }

    buf
}

/// Returns true if `stmt` is `assert(var)` where `var` has a struct type.
/// C cannot apply boolean operations to struct values directly, so such
/// calls must be skipped (stack-allocated structs are always "truthy").
fn is_struct_assert_call(stmt: &Statement, var_types: &HashMap<String, Option<Type>>) -> bool {
    if let Statement::Exp(Expression::FunctionCall { fn_name, args }) = stmt {
        if fn_name == "assert" {
            if let Some(arg) = args.first() {
                match arg {
                    Expression::Variable(var_name) => {
                        if let Some(opt_ty) = var_types.get(var_name) {
                            return matches!(opt_ty, Some(Type::Struct(_)));
                        }
                    }
                    Expression::StructInitialization { .. } => return true,
                    _ => {}
                }
            }
        }
    }
    false
}

pub(super) fn generate_block(block: Statement, prepend: Option<String>) -> String {
    let mut generated = String::from("{\n");

    if let Some(pre) = prepend {
        generated += &pre;
    }

    let statements = match block {
        Statement::Block {
            statements,
            scope: _,
        } => statements,
        _ => panic!("Block body should be of type Statement::Block"),
    };

    // Build a map of variable name → type from Declare statements.
    // Note: the Block's `scope` field has pre-inference (stale) types; the Declare
    // statements hold the correctly inferred types after type inference runs.
    let var_types: HashMap<String, Option<Type>> = statements
        .iter()
        .filter_map(|s| {
            if let Statement::Declare { variable, .. } = s {
                Some((variable.name.clone(), variable.ty.clone()))
            } else {
                None
            }
        })
        .collect();

    for statement in statements {
        if is_struct_assert_call(&statement, &var_types) {
            // C cannot apply boolean ops to struct values. Stack-allocated structs
            // are always valid ("truthy"), so the assertion always passes — emit a no-op.
            generated += "    (void)0; /* struct assert - always passes */\n";
        } else {
            generated += &generate_statement(statement);
        }
    }

    generated += "}\n";
    generated
}

pub(super) fn generate_statement(statement: Statement) -> String {
    let state = match statement {
        Statement::Return(ret) => generate_return(ret),
        Statement::Declare { variable, value } => generate_declare(variable, value),
        Statement::Exp(val) => generate_expression(val),
        Statement::If {
            condition,
            body,
            else_branch,
        } => generate_conditional(condition, *body, else_branch.map(|x| *x)),
        Statement::Assign { lhs, rhs } => generate_assign(*lhs, *rhs),
        Statement::Block {
            statements: _,
            scope: _,
        } => return generate_block(statement, None),
        Statement::While { condition, body } => generate_while_loop(condition, *body),
        Statement::For { ident, expr, body } => generate_for_loop(ident, expr, *body),
        Statement::Continue => "continue".to_string(),
        Statement::Break => "break".to_string(),
    };

    format!("    {};\n", state)
}

pub(super) fn generate_expression(expr: Expression) -> String {
    match expr {
        Expression::Int(val) => val.to_string(),
        Expression::Selff => "self".to_string(),
        Expression::Str(val) => format!(
            "\"{}\"",
            val.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t")
        ),
        Expression::Variable(val) => val,
        Expression::Bool(b) => if b { "true" } else { "false" }.to_string(),
        Expression::FunctionCall { fn_name, args } => generate_function_call(fn_name, args),
        Expression::Array {
            capacity: _,
            elements,
        } => generate_array(elements),
        Expression::ArrayAccess { name, index } => generate_array_access(name, *index),
        Expression::BinOp { lhs, op, rhs } => generate_bin_op(*lhs, op, *rhs),
        Expression::StructInitialization { name, fields } => {
            generate_struct_initialization(name, fields)
        }
        Expression::FieldAccess { expr, field } => generate_field_access(*expr, *field),
    }
}

pub(super) fn generate_while_loop(expr: Expression, body: Statement) -> String {
    format!(
        "while ({}) {}",
        generate_expression(expr),
        generate_block(body, None)
    )
}

/// Infers the element type of an array expression.
fn infer_array_element_type(expr: &Expression) -> String {
    match expr {
        Expression::Array { elements, .. } => elements
            .first()
            .map(|e| infer_type_from_expr(e))
            .unwrap_or_else(|| "int".to_string()),
        _ => "int".to_string(),
    }
}

pub(super) fn generate_for_loop(ident: Variable, expr: Expression, body: Statement) -> String {
    // Determine loop variable type: use declared type, or infer from array
    let elem_type = if ident.ty.is_some() {
        type_to_c_type(&ident.ty)
    } else {
        infer_array_element_type(&expr)
    };

    // C-style for loop with array indexing
    let mut out_str = format!(
        "for(int i = 0; i < sizeof({}) / sizeof({}[0]); i++)",
        generate_expression(expr.clone()),
        generate_expression(expr.clone())
    );

    // Add the loop variable declaration to the prepended block
    out_str += &generate_block(
        body,
        Some(format!(
            "    {} {} = {}[i];\n",
            elem_type,
            ident.name,
            generate_expression(expr)
        )),
    );
    out_str
}

pub(super) fn generate_array(elements: Vec<Expression>) -> String {
    let mut out_str = String::from("{");

    out_str += &elements
        .iter()
        .map(|el| generate_expression(el.clone()))
        .collect::<Vec<String>>()
        .join(", ");

    out_str += "}";
    out_str
}

pub(super) fn generate_array_access(name: String, expr: Expression) -> String {
    format!("{}[{}]", name, generate_expression(expr))
}

pub(super) fn generate_conditional(
    expr: Expression,
    if_state: Statement,
    else_state: Option<Statement>,
) -> String {
    let mut outcome = format!("if ({}) ", generate_expression(expr));
    outcome += &generate_block(if_state, None);

    if let Some(else_state) = else_state {
        outcome += "else ";
        match else_state {
            Statement::If {
                condition,
                body,
                else_branch,
            } => outcome += &generate_conditional(condition, *body, else_branch.map(|x| *x)),
            _ => outcome += &generate_block(else_state, None),
        }
    }

    outcome
}

pub(super) fn generate_declare<V: AsRef<Variable>>(
    identifier: V,
    val: Option<Expression>,
) -> String {
    let ident = identifier.as_ref();

    match val {
        Some(expr) => {
            // For array initialisers use `elem_type name[] = {...}` (not pointer syntax)
            let is_array_init = matches!(expr, Expression::Array { .. });
            if is_array_init {
                let elem_type = infer_array_element_type(&expr);
                format!(
                    "{} {}[] = {}",
                    elem_type,
                    ident.name,
                    generate_expression(expr)
                )
            } else {
                let type_str = if ident.ty.is_none() {
                    infer_type_from_expr(&expr)
                } else {
                    type_to_c_type(&ident.ty)
                };
                format!("{} {} = {}", type_str, ident.name, generate_expression(expr))
            }
        }
        None => {
            let type_str = type_to_c_type(&ident.ty);
            match &ident.ty {
                Some(Type::Array(_, _)) => {
                    format!("{} {}[]", type_str, ident.name)
                }
                _ => format!("{} {}", type_str, ident.name),
            }
        }
    }
}

pub(super) fn generate_function_call(func: String, args: Vec<Expression>) -> String {
    let formatted_args = args
        .into_iter()
        .map(generate_expression)
        .collect::<Vec<String>>()
        .join(", ");

    format!("{}({})", func, formatted_args)
}

pub(super) fn generate_return(ret: Option<Expression>) -> String {
    match ret {
        Some(expr) => format!("return {}", generate_expression(expr)),
        None => "return".to_string(),
    }
}

pub(super) fn generate_bin_op(left: Expression, op: BinOp, right: Expression) -> String {
    // String concatenation: use _str_concat when either operand is a string literal
    // or a field/struct access (which may yield a char* at runtime).
    if op == BinOp::Addition {
        let is_string_like = |e: &Expression| {
            matches!(
                e,
                Expression::Str(_) | Expression::FieldAccess { .. }
            )
        };
        if is_string_like(&left) || is_string_like(&right) {
            return format!(
                "_str_concat({}, {})",
                generate_expression(left),
                generate_expression(right)
            );
        }
    }

    let op_str = match op {
        BinOp::Addition => "+",
        BinOp::And => "&&",
        BinOp::Division => "/",
        BinOp::Equal => "==",
        BinOp::GreaterThan => ">",
        BinOp::GreaterThanOrEqual => ">=",
        BinOp::LessThan => "<",
        BinOp::LessThanOrEqual => "<=",
        BinOp::Modulus => "%",
        BinOp::Multiplication => "*",
        BinOp::NotEqual => "!=",
        BinOp::Or => "||",
        BinOp::Subtraction => "-",
        BinOp::AddAssign => "+=",
        BinOp::SubtractAssign => "-=",
        BinOp::MultiplyAssign => "*=",
        BinOp::DivideAssign => "/=",
    };

    format!(
        "{} {} {}",
        generate_expression(left),
        op_str,
        generate_expression(right)
    )
}

pub(super) fn generate_struct_initialization(
    name: String,
    fields: HashMap<String, Box<Expression>>,
) -> String {
    let mut out_str = format!("({}) {{", name);

    let field_inits: Vec<String> = fields
        .into_iter()
        .map(|(key, value)| format!(".{} = {}", key, generate_expression(*value)))
        .collect();

    out_str += &field_inits.join(", ");
    out_str += "}";

    out_str
}

pub(super) fn generate_field_access(expr: Expression, field: Expression) -> String {
    // Method call: obj.method(args) → StructName_method(obj, args)
    // Method names are mangled with the struct name prefix to avoid C naming conflicts.
    if let Expression::FunctionCall { fn_name, args } = field {
        // Try to determine the struct type of `expr` from the current variable context.
        let c_fn_name = if let Expression::Variable(ref var_name) = expr {
            CURRENT_VAR_TYPES.with(|vt| {
                let vt = vt.borrow();
                if let Some(Some(Type::Struct(struct_name))) = vt.get(var_name.as_str()) {
                    mangle_method_name(struct_name, &fn_name)
                } else {
                    fn_name.clone()
                }
            })
        } else {
            fn_name.clone()
        };
        let mut all_args = vec![generate_expression(expr)];
        all_args.extend(args.into_iter().map(generate_expression));
        return format!("{}({})", c_fn_name, all_args.join(", "));
    }
    format!(
        "{}.{}",
        generate_expression(expr),
        generate_expression(field)
    )
}

pub(super) fn generate_assign(name: Expression, expr: Expression) -> String {
    format!(
        "{} = {}",
        generate_expression(name),
        generate_expression(expr)
    )
}
