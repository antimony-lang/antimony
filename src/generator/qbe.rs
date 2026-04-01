/**
 * Copyright 2021 Alexey Yerin
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
use super::{Generator, GeneratorResult};
use crate::ast::types::Type;
use crate::ast::*;
use std::cmp;
use std::collections::HashMap;
use std::sync::Arc;

/// SSA implementations of the four runtime helpers that previously lived in
/// `builtin_qbe.c`.  Emitted verbatim at the top of every QBE output file so
/// no C compilation step is needed for them.
///
/// * `_printf`    – write a string to stdout via `write(1, …)`
/// * `_exit`      – flush all stdio streams then terminate via `_Exit`
/// * `_strlen`    – thin word-width wrapper around libc `strlen`
/// * `_parse_int` – thin wrapper around libc `atoi`
///
/// `_str_concat`, `_int_to_str`, and `_read_line` still live in
/// `builtin_qbe.c` because they need `malloc`/`snprintf`/`fgets` logic that
/// is awkward to express in QBE IL (variadic calls, multi-step allocation).
const RUNTIME_PREAMBLE: &str = r#"
# _printf(msg: l) — write string to stdout using write(2)
function $_printf(l %msg) {
@start
    %len =l call $strlen(l %msg)
    %_ =l call $write(w 1, l %msg, l %len)
    ret
}

# _exit(code: w) — flush all stdio streams then hard-exit
function $_exit(w %code) {
@start
    call $fflush(l 0)
    call $_Exit(w %code)
    ret
}

# _strlen(s: l): w — word-width strlen for Antimony's int return type
export function w $_strlen(l %s) {
@start
    %n =l call $strlen(l %s)
    %nw =w copy %n
    ret %nw
}

# _parse_int(s: l): w — atoi wrapper
export function w $_parse_int(l %s) {
@start
    %n =w call $atoi(l %s)
    ret %n
}

# _strcmp(a: l, b: l): w — compare two C strings via libc strcmp
export function w $_strcmp(l %a, l %b) {
@start
    %r =w call $strcmp(l %a, l %b)
    ret %r
}
"#;

/// Information stored for each variable in scope
type VarInfo = (qbe::Type, qbe::Value, Option<Type>);

/// Built-in functions that the QBE backend handles as inline intrinsics
/// rather than emitting a regular function call.
#[derive(Clone)]
enum Intrinsic {
    /// Load the array length from the header at offset 0
    ArrayLen,
}

pub struct QbeGenerator {
    /// Counter for unique temporary names
    tmp_counter: u32,
    /// Block-scoped variable -> (qbe_type, temporary, ast_type) mappings
    scopes: Vec<HashMap<String, VarInfo>>,
    /// Structure -> (type, meta data, size) mappings
    struct_map: HashMap<String, (qbe::Type, StructMeta, u64)>,
    /// Label prefix of loop scopes
    loop_labels: Vec<String>,
    /// Data defintions collected during generation
    datadefs: Vec<qbe::DataDef>,
    /// Type defintions collected during generation
    typedefs: Vec<Arc<qbe::TypeDef>>,
    /// Function name -> return type (populated by pre-pass before codegen)
    fn_signatures: HashMap<String, Option<qbe::Type>>,
    /// Function name -> parameter types (populated by pre-pass before codegen)
    fn_param_types: HashMap<String, Vec<qbe::Type>>,
    /// Function name -> AST return type (populated by pre-pass before codegen)
    fn_ast_signatures: HashMap<String, Option<Type>>,
    /// Function name -> AST parameter types (populated by pre-pass before codegen)
    fn_param_ast_types: HashMap<String, Vec<Option<Type>>>,
    /// Functions replaced by inline intrinsics
    intrinsics: HashMap<String, Intrinsic>,
    /// Module being built
    module: qbe::Module,
}

/// Mapping of field -> (type, offset, ast_type)
type StructMeta = HashMap<String, (qbe::Type, u64, Option<Type>)>;

/// Infer the return type of a function from its body when no annotation is present.
/// Scans declarations and parameters for typed variables, then looks for a matching return statement.
///
/// TODO: This logic belongs in `src/parser/infer.rs`, which should populate
/// `HFunction::ret_type` so all backends get inferred return types for free.
/// Once that's done, remove these helpers and the `fn_ast_signatures` pre-pass.
fn infer_fn_return_type(body: &Statement, args: &[Variable]) -> Option<Type> {
    let mut var_types: HashMap<String, Type> = HashMap::new();
    for arg in args {
        if let Some(ty) = &arg.ty {
            var_types.insert(arg.name.clone(), ty.clone());
        }
    }
    collect_decl_types(body, &mut var_types);
    find_return_type(body, &var_types)
}

fn collect_decl_types(stmt: &Statement, out: &mut HashMap<String, Type>) {
    match stmt {
        Statement::Block { statements, .. } => {
            for s in statements {
                collect_decl_types(s, out);
            }
        }
        Statement::Declare { variable, .. } => {
            if let Some(ty) = &variable.ty {
                out.insert(variable.name.clone(), ty.clone());
            }
        }
        _ => {}
    }
}

fn find_return_type(stmt: &Statement, var_types: &HashMap<String, Type>) -> Option<Type> {
    match stmt {
        Statement::Block { statements, .. } => {
            for s in statements {
                if let Some(ty) = find_return_type(s, var_types) {
                    return Some(ty);
                }
            }
            None
        }
        Statement::Return(Some(expr)) => infer_expr_type(expr, var_types),
        _ => None,
    }
}

fn infer_expr_type(expr: &Expression, var_types: &HashMap<String, Type>) -> Option<Type> {
    match expr {
        Expression::Int(_) => Some(Type::Int),
        Expression::Str(_) => Some(Type::Str),
        Expression::Bool(_) => Some(Type::Bool),
        Expression::Variable(name) => var_types.get(name).cloned(),
        Expression::StructInitialization { name, .. } => Some(Type::Struct(name.clone())),
        Expression::BinOp { lhs, op, .. } => {
            if matches!(op, BinOp::Addition) && is_string_expr_static(lhs, var_types) {
                return Some(Type::Str);
            }
            match op {
                BinOp::Equal
                | BinOp::NotEqual
                | BinOp::LessThan
                | BinOp::LessThanOrEqual
                | BinOp::GreaterThan
                | BinOp::GreaterThanOrEqual
                | BinOp::And
                | BinOp::Or => Some(Type::Bool),
                _ => Some(Type::Int),
            }
        }
        Expression::FunctionCall { .. } => None,
        _ => None,
    }
}

fn is_string_expr_static(expr: &Expression, var_types: &HashMap<String, Type>) -> bool {
    match expr {
        Expression::Str(_) => true,
        Expression::Variable(name) => matches!(var_types.get(name), Some(Type::Str)),
        Expression::BinOp { lhs, op, .. } => {
            matches!(op, BinOp::Addition) && is_string_expr_static(lhs, var_types)
        }
        _ => false,
    }
}

impl Generator for QbeGenerator {
    fn generate(prog: Module) -> GeneratorResult<String> {
        let mut intrinsics = HashMap::new();
        intrinsics.insert("len".to_string(), Intrinsic::ArrayLen);

        let mut generator = QbeGenerator {
            tmp_counter: 0,
            scopes: Vec::new(),
            struct_map: HashMap::new(),
            loop_labels: Vec::new(),
            datadefs: Vec::new(),
            typedefs: Vec::new(),
            fn_signatures: HashMap::new(),
            fn_param_types: HashMap::new(),
            fn_ast_signatures: HashMap::new(),
            fn_param_ast_types: HashMap::new(),
            intrinsics,
            module: qbe::Module::new(),
        };

        // Pre-pass: register all struct names so forward references work
        for def in &prog.structs {
            generator
                .struct_map
                .insert(def.name.clone(), (qbe::Type::Word, StructMeta::new(), 0));
        }

        for def in &prog.structs {
            let structure = generator.generate_struct(def)?;

            #[cfg(debug_assertions)]
            {
                // Just in case it incorrectly calculates offsets
                let (ty, meta, size) = generator.struct_map.get(&def.name).unwrap();
                eprintln!("Struct: {}", def.name);
                eprintln!("Type: {:?}", ty);
                eprintln!("Meta: {:?}", meta);
                eprintln!("Size: {}", size);
                // We can add debug comments to the module if needed
            }

            let typedef_arc = Arc::new(structure);
            generator.module.add_type(Arc::clone(&typedef_arc));
            generator.typedefs.push(typedef_arc);

            // Replace the Word placeholder in struct_map with the proper Aggregate type
            let struct_type = qbe::Type::aggregate(generator.typedefs.last().unwrap());
            if let Some(entry) = generator.struct_map.get_mut(&def.name) {
                entry.0 = struct_type;
            }
        }

        // Pre-pass: infer return types for external C builtins (underscore-prefixed names)
        // by examining thin wrapper functions whose body is a single `return _builtin(...)`.
        // This lets the call-site emit the correct QBE type without hardcoding builtin names.
        for func in &prog.func {
            if let Some(ret_ty) = &func.ret_type {
                let qbe_ret_ty = generator.get_type(ret_ty.to_owned())?.into_abi();
                // Word is already the fallback, so only register non-Word return types.
                if qbe_ret_ty == qbe::Type::Word {
                    continue;
                }
                if let Statement::Block { statements, .. } = &func.body {
                    if let [Statement::Return(Some(Expression::FunctionCall { fn_name, .. }))] =
                        statements.as_slice()
                    {
                        if fn_name.starts_with('_') {
                            generator
                                .fn_signatures
                                .insert(fn_name.clone(), Some(qbe_ret_ty));
                        }
                    }
                }
            }
        }

        // Pre-pass: collect function return types so callers know what type to expect
        for func in &prog.func {
            let ast_ret = func
                .ret_type
                .clone()
                .or_else(|| infer_fn_return_type(&func.body, &func.arguments));
            let ret_type = match &ast_ret {
                Some(Type::Struct(_)) => Some(qbe::Type::Long),
                Some(ty) => Some(generator.get_type(ty.to_owned())?.into_abi()),
                None => None,
            };
            generator.fn_signatures.insert(func.name.clone(), ret_type);
            generator
                .fn_ast_signatures
                .insert(func.name.clone(), ast_ret);

            let param_types: Vec<qbe::Type> = func
                .arguments
                .iter()
                .filter_map(|arg| arg.ty.as_ref())
                .map(|ty| generator.get_type(ty.to_owned()).map(|t| t.into_abi()))
                .collect::<Result<Vec<_>, _>>()?;
            generator
                .fn_param_types
                .insert(func.name.clone(), param_types);

            let param_ast_types: Vec<Option<Type>> =
                func.arguments.iter().map(|arg| arg.ty.clone()).collect();
            generator
                .fn_param_ast_types
                .insert(func.name.clone(), param_ast_types);
        }

        // Pre-pass: collect method return types
        for def in &prog.structs {
            for method in &def.methods {
                let mangled = format!("{}_{}", def.name, method.name);
                let ret_type = if let Some(ty) = &method.ret_type {
                    Some(generator.get_type(ty.to_owned())?.into_abi())
                } else {
                    None
                };
                generator.fn_signatures.insert(mangled, ret_type);
            }
        }

        for func in &prog.func {
            let func = generator.generate_function(func)?;
            generator.module.add_function(func);
        }

        // Generate methods as standalone functions: StructName__methodName(self: Long, ...)
        for def in &prog.structs {
            for method in &def.methods {
                let qfunc = generator.generate_method(def, method)?;
                generator.module.add_function(qfunc);
            }
        }

        for def in &generator.datadefs {
            generator.module.add_data(def.clone());
        }

        Ok(format!(
            "{}# --- user code ---\n{}",
            RUNTIME_PREAMBLE, generator.module
        ))
    }
}

impl QbeGenerator {
    /// Calculate the alignment requirement for a type
    fn type_alignment(&self, ty: &qbe::Type) -> u64 {
        // Helper function that doesn't use self to avoid the recursive self parameter warning
        fn alignment_of(ty: &qbe::Type) -> u64 {
            match ty {
                qbe::Type::Byte | qbe::Type::SignedByte | qbe::Type::UnsignedByte => 1,
                qbe::Type::Halfword | qbe::Type::SignedHalfword | qbe::Type::UnsignedHalfword => 2,
                qbe::Type::Word | qbe::Type::Single => 4,
                qbe::Type::Long | qbe::Type::Double => 8,
                qbe::Type::Aggregate(td) => match &**td {
                    qbe::TypeDef::Regular { items, .. } => items
                        .iter()
                        .map(|(item_ty, _)| alignment_of(item_ty))
                        .max()
                        .unwrap_or(1),
                    _ => 1,
                },
                qbe::Type::Zero => 1,
            }
        }

        alignment_of(ty)
    }

    /// Returns the wider of two QBE types (Byte < Word < Long).
    fn wider_type(a: &qbe::Type, b: &qbe::Type) -> qbe::Type {
        fn rank(ty: &qbe::Type) -> u8 {
            match ty {
                qbe::Type::Byte | qbe::Type::SignedByte | qbe::Type::UnsignedByte => 1,
                qbe::Type::Halfword | qbe::Type::SignedHalfword | qbe::Type::UnsignedHalfword => 2,
                qbe::Type::Word | qbe::Type::Single => 3,
                qbe::Type::Long | qbe::Type::Double => 4,
                _ => 3, // default to Word-sized
            }
        }
        if rank(a) >= rank(b) {
            a.clone()
        } else {
            b.clone()
        }
    }

    /// Calculate the aligned offset for a field
    fn align_offset(&self, offset: u64, alignment: u64) -> u64 {
        (offset + alignment - 1) & !(alignment - 1)
    }

    /// Returns an aggregate type for a structure (note: has side effects)
    fn generate_struct(&mut self, def: &StructDef) -> GeneratorResult<qbe::TypeDef> {
        self.tmp_counter += 1;
        let ident = format!("struct.{}", self.tmp_counter);
        let mut items: Vec<(qbe::Type, usize)> = Vec::new();
        let mut meta = StructMeta::new();
        let mut offset = 0_u64;
        let mut max_align = 1_u64;

        for field in &def.fields {
            let ty = self.get_type(
                field
                    .ty
                    .as_ref()
                    .ok_or_else(|| "Structure field must have a type".to_owned())?
                    .to_owned(),
            )?;

            let field_align = self.type_alignment(&ty);
            max_align = cmp::max(max_align, field_align);

            // Align the current offset for this field
            offset = self.align_offset(offset, field_align);

            meta.insert(field.name.clone(), (ty.clone(), offset, field.ty.clone()));
            items.push((ty.clone(), 1));

            offset += self.type_size(&ty);
        }

        // Final size needs to be aligned to the struct's alignment
        offset = self.align_offset(offset, max_align);

        self.struct_map.insert(
            def.name.clone(),
            (
                qbe::Type::Word, /* temporary placeholder */
                meta,
                offset,
            ),
        );

        Ok(qbe::TypeDef::Regular {
            ident,
            align: Some(max_align),
            items,
        })
    }

    fn generate_function(&mut self, func: &Function) -> GeneratorResult<qbe::Function> {
        // Function argument scope
        self.scopes.push(HashMap::new());

        let mut arguments: Vec<(qbe::Type, qbe::Value)> = Vec::new();
        for arg in &func.arguments {
            let ast_type = arg
                .ty
                .as_ref()
                .ok_or("Function arguments must have a type")?
                .to_owned();
            let ty = self.get_type(ast_type.clone())?;
            let tmp = self.new_var(&ty, &arg.name, Some(ast_type))?;

            arguments.push((ty.into_abi(), tmp));
        }

        let effective_ret = func
            .ret_type
            .clone()
            .or_else(|| infer_fn_return_type(&func.body, &func.arguments));
        let return_ty = match &effective_ret {
            Some(Type::Struct(_)) => Some(qbe::Type::Long),
            Some(ty) => Some(self.get_type(ty.to_owned())?.into_abi()),
            None => None,
        };

        let mut qfunc = qbe::Function::new(
            qbe::Linkage::public(),
            func.name.clone(),
            arguments,
            return_ty,
        );

        qfunc.add_block("start".to_owned());

        self.generate_statement(&mut qfunc, &func.body)?;

        // Check if the function properly returns
        let last_block_empty = qfunc.blocks.last().is_some_and(|b| b.items.is_empty());
        let returns = qfunc.blocks.last().is_some_and(|b| {
            b.items.last().is_some_and(|item| {
                matches!(
                    item,
                    qbe::BlockItem::Statement(qbe::Statement::Volatile(qbe::Instr::Ret(_)))
                )
            })
        });

        if !returns {
            if func.ret_type.is_none() || last_block_empty {
                // For void functions, add an implicit return.
                // For typed functions where the last block is empty, all
                // reachable paths already return — add a dummy ret for QBE.
                qfunc.add_instr(qbe::Instr::Ret(None));
            } else {
                return Err(format!(
                    "Function '{}' does not return in all code paths",
                    &func.name
                ));
            }
        }

        self.scopes.pop();

        Ok(qfunc)
    }

    /// Generates a struct method as a standalone function `StructName__methodName`
    /// with an implicit `self: Long` first argument (pointer to struct)
    fn generate_method(
        &mut self,
        struct_def: &StructDef,
        method: &Function,
    ) -> GeneratorResult<qbe::Function> {
        self.scopes.push(HashMap::new());

        // Prepend self as first argument (Long pointer to struct)
        let self_tmp = self.new_var(
            &qbe::Type::Long,
            "self",
            Some(Type::Struct(struct_def.name.clone())),
        )?;
        let mut arguments: Vec<(qbe::Type, qbe::Value)> = vec![(qbe::Type::Long, self_tmp)];

        for arg in &method.arguments {
            let ast_type = arg
                .ty
                .as_ref()
                .ok_or("Method arguments must have a type")?
                .to_owned();
            let ty = self.get_type(ast_type.clone())?;
            let tmp = self.new_var(&ty, &arg.name, Some(ast_type))?;
            arguments.push((ty.into_abi(), tmp));
        }

        let return_ty = if let Some(ty) = &method.ret_type {
            Some(self.get_type(ty.to_owned())?.into_abi())
        } else {
            None
        };

        let mangled_name = format!("{}_{}", struct_def.name, method.name);
        let mut qfunc = qbe::Function::new(
            qbe::Linkage::public(),
            mangled_name.clone(),
            arguments,
            return_ty,
        );

        qfunc.add_block("start".to_owned());
        self.generate_statement(&mut qfunc, &method.body)?;

        let last_block_empty = qfunc.blocks.last().is_some_and(|b| b.items.is_empty());
        let returns = qfunc.blocks.last().is_some_and(|b| {
            b.items.last().is_some_and(|item| {
                matches!(
                    item,
                    qbe::BlockItem::Statement(qbe::Statement::Volatile(qbe::Instr::Ret(_)))
                )
            })
        });

        if !returns {
            if method.ret_type.is_none() || last_block_empty {
                qfunc.add_instr(qbe::Instr::Ret(None));
            } else {
                return Err(format!(
                    "Method '{}' does not return in all code paths",
                    mangled_name
                ));
            }
        }

        self.scopes.pop();
        Ok(qfunc)
    }

    /// Generates a method call: `obj.method(args)` → `call StructName__methodName(obj_ptr, args...)`
    fn generate_method_call(
        &mut self,
        func: &mut qbe::Function,
        obj: &Expression,
        method_name: &str,
        args: &[Expression],
    ) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        let struct_name = self.get_struct_name_of(obj)?;
        let (_, obj_ptr) = self.generate_expression(func, obj)?;

        let mut call_args: Vec<(qbe::Type, qbe::Value)> = vec![(qbe::Type::Long, obj_ptr)];
        for arg in args {
            let result = self.generate_expression(func, arg)?;
            call_args.push(result);
        }

        let mangled_name = format!("{}_{}", struct_name, method_name);
        let is_void = self
            .fn_signatures
            .get(&mangled_name)
            .map(|v| v.is_none())
            .unwrap_or(false);
        let ret_type_opt = self.fn_signatures.get(&mangled_name).cloned().flatten();

        if is_void {
            func.add_instr(qbe::Instr::Call(mangled_name, call_args, None));
            Ok((qbe::Type::Word, qbe::Value::Const(0)))
        } else {
            let ret_type = ret_type_opt.unwrap_or(qbe::Type::Word);
            let tmp = self.new_temporary();
            func.assign_instr(
                tmp.clone(),
                ret_type.clone(),
                qbe::Instr::Call(mangled_name, call_args, None),
            );
            Ok((ret_type, tmp))
        }
    }

    /// Generates a statement
    fn generate_statement(
        &mut self,
        func: &mut qbe::Function,
        stmt: &Statement,
    ) -> GeneratorResult<()> {
        match stmt {
            Statement::Block {
                statements,
                scope: _,
            } => {
                self.scopes.push(HashMap::new());
                for stmt in statements.iter() {
                    self.generate_statement(func, stmt)?;
                }
                self.scopes.pop();
            }
            Statement::Declare { variable, value } => {
                let ast_type = variable
                    .ty
                    .as_ref()
                    .ok_or_else(|| format!("Missing type for variable '{}'", &variable.name))?
                    .to_owned();
                let ty = self.get_type(ast_type.clone())?;
                let tmp = self.new_var(&ty, &variable.name, Some(ast_type.clone()))?;

                if let Some(expr) = value {
                    let (expr_type, expr_value) = self.generate_expression(func, expr)?;
                    func.assign_instr(tmp, expr_type, qbe::Instr::Copy(expr_value));
                } else if let Type::Array(ref elem_ast_type, Some(size)) = variable
                    .ty
                    .as_ref()
                    .ok_or_else(|| format!("Missing type for variable '{}'", &variable.name))?
                {
                    // Uninitialized sized arrays need memory allocated upfront,
                    // since subsequent index assignments use the variable as a base pointer.
                    let elem_qbe_type = self.get_type(*elem_ast_type.clone())?;
                    let elem_size = self.type_size(&elem_qbe_type);
                    let total_size = 8 + (*size as u64) * elem_size;

                    func.assign_instr(tmp.clone(), qbe::Type::Long, qbe::Instr::Alloc8(total_size));
                    func.add_instr(qbe::Instr::Store(
                        qbe::Type::Long,
                        tmp,
                        qbe::Value::Const(*size as u64),
                    ));

                    // Register a typedef for the array
                    self.tmp_counter += 1;
                    let name = format!("array.{}", self.tmp_counter);
                    let typedef = qbe::TypeDef::Regular {
                        ident: name.clone(),
                        align: None,
                        items: vec![(qbe::Type::Long, 1), (elem_qbe_type, *size)],
                    };
                    let typedef_arc = Arc::new(typedef);
                    self.module.add_type(Arc::clone(&typedef_arc));
                    self.typedefs.push(typedef_arc);
                }
            }
            Statement::Assign { lhs, rhs } => {
                let (_, rhs_value) = self.generate_expression(func, rhs)?;
                self.generate_assignment(func, lhs, rhs_value)?;
            }
            Statement::Return(val) => match val {
                Some(expr) => {
                    let (_, result) = self.generate_expression(func, expr)?;
                    func.add_instr(qbe::Instr::Ret(Some(result)));
                }
                None => func.add_instr(qbe::Instr::Ret(None)),
            },
            Statement::If {
                condition,
                body,
                else_branch,
            } => {
                self.generate_if(func, condition, body, else_branch)?;
            }
            Statement::While { condition, body } => {
                self.generate_while(func, condition, body)?;
            }
            Statement::Break => {
                if let Some(label) = &self.loop_labels.last() {
                    func.add_instr(qbe::Instr::Jmp(format!("{}.end", label)));
                } else {
                    return Err("break used outside of a loop".to_owned());
                }
            }
            Statement::Continue => {
                if let Some(label) = &self.loop_labels.last() {
                    func.add_instr(qbe::Instr::Jmp(format!("{}.cond", label)));
                } else {
                    return Err("continue used outside of a loop".to_owned());
                }
            }
            Statement::For { ident, expr, body } => {
                self.generate_for_loop(func, ident, expr, body)?;
            }
            Statement::Exp(expr) => {
                self.generate_expression(func, expr)?;
            }
        }
        Ok(())
    }

    /// Generates code for a built-in intrinsic, replacing the normal function call.
    fn generate_intrinsic(
        &mut self,
        func: &mut qbe::Function,
        intrinsic: &Intrinsic,
        args: &[Expression],
    ) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        match intrinsic {
            Intrinsic::ArrayLen => {
                let arr_arg = args
                    .first()
                    .ok_or_else(|| "len() requires one argument".to_string())?;
                let (_, arr_ptr) = self.generate_expression(func, arr_arg)?;

                // Load length (Long) from array header at offset 0
                let len_tmp = self.new_temporary();
                func.assign_instr(
                    len_tmp.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Load(qbe::Type::Long, arr_ptr),
                );

                // Truncate Long to Word (len() returns int)
                let word_tmp = self.new_temporary();
                func.assign_instr(word_tmp.clone(), qbe::Type::Word, qbe::Instr::Copy(len_tmp));

                Ok((qbe::Type::Word, word_tmp))
            }
        }
    }

    /// Generates an expression
    fn generate_expression(
        &mut self,
        func: &mut qbe::Function,
        expr: &Expression,
    ) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        match expr {
            Expression::Int(literal) => {
                let tmp = self.new_temporary();
                func.assign_instr(
                    tmp.clone(),
                    qbe::Type::Word,
                    qbe::Instr::Copy(qbe::Value::Const(*literal as u64)),
                );

                Ok((qbe::Type::Word, tmp))
            }
            Expression::Str(string) => self.generate_string(string),
            Expression::Bool(literal) => {
                let tmp = self.new_temporary();
                func.assign_instr(
                    tmp.clone(),
                    qbe::Type::Word,
                    qbe::Instr::Copy(qbe::Value::Const(u64::from(*literal))),
                );

                Ok((qbe::Type::Word, tmp))
            }
            Expression::Array { capacity, elements } => {
                self.generate_array(func, *capacity, elements)
            }
            Expression::FunctionCall { fn_name, args } => {
                if let Some(intrinsic) = self.intrinsics.get(fn_name).cloned() {
                    return self.generate_intrinsic(func, &intrinsic, args);
                }

                // Collect arguments first to avoid multiple mutable borrows
                let mut arg_results = Vec::new();
                for arg in args.iter() {
                    let result = self.generate_expression(func, arg)?;
                    arg_results.push(result);
                }

                // Widen arguments if the callee expects a larger type (e.g. Type::Any → Long)
                let param_types_opt = self.fn_param_types.get(fn_name).cloned();
                let param_ast_types_opt = self.fn_param_ast_types.get(fn_name).cloned();
                let mut new_args: Vec<(qbe::Type, qbe::Value)> = Vec::new();
                for (i, (arg_ty, arg_val)) in arg_results.into_iter().enumerate() {
                    // int → string coercion: call _int_to_str when passing an int to a string param
                    if arg_ty == qbe::Type::Word {
                        let param_ast_ty = param_ast_types_opt
                            .as_ref()
                            .and_then(|v| v.get(i))
                            .and_then(|t| t.as_ref());
                        if matches!(param_ast_ty, Some(Type::Str)) {
                            let str_tmp = self.new_temporary();
                            func.assign_instr(
                                str_tmp.clone(),
                                qbe::Type::Long,
                                qbe::Instr::Call(
                                    "_int_to_str".to_string(),
                                    vec![(qbe::Type::Word, arg_val)],
                                    None,
                                ),
                            );
                            new_args.push((qbe::Type::Long, str_tmp));
                            continue;
                        }
                    }
                    if let Some(ref param_types) = param_types_opt {
                        if let Some(param_ty) = param_types.get(i) {
                            if *param_ty == qbe::Type::Long && arg_ty == qbe::Type::Word {
                                let widened = self.new_temporary();
                                func.assign_instr(
                                    widened.clone(),
                                    qbe::Type::Long,
                                    qbe::Instr::Extuw(arg_val),
                                );
                                new_args.push((qbe::Type::Long, widened));
                                continue;
                            } else if *param_ty == qbe::Type::Long && arg_ty == qbe::Type::Byte {
                                let widened = self.new_temporary();
                                func.assign_instr(
                                    widened.clone(),
                                    qbe::Type::Long,
                                    qbe::Instr::Extub(arg_val),
                                );
                                new_args.push((qbe::Type::Long, widened));
                                continue;
                            }
                        }
                    }
                    new_args.push((arg_ty, arg_val));
                }

                // Look up the return type from the pre-pass signature map.
                // A map entry of None means the function is void (no return value).
                let is_void = self
                    .fn_signatures
                    .get(fn_name)
                    .map(|v| v.is_none())
                    .unwrap_or(false);
                let ret_type_opt = self.fn_signatures.get(fn_name).cloned().flatten();

                if is_void {
                    func.add_instr(qbe::Instr::Call(fn_name.clone(), new_args, None));
                    Ok((qbe::Type::Word, qbe::Value::Const(0)))
                } else {
                    let ret_type = ret_type_opt.unwrap_or(qbe::Type::Word);
                    let tmp = self.new_temporary();
                    func.assign_instr(
                        tmp.clone(),
                        ret_type.clone(),
                        qbe::Instr::Call(fn_name.clone(), new_args, None),
                    );
                    Ok((ret_type, tmp))
                }
            }
            Expression::Variable(name) => {
                let (ty, val, _) = self.get_var(name)?;
                Ok((ty.to_owned(), val.to_owned()))
            }
            Expression::Selff => {
                let (ty, val, _) = self.get_var("self")?;
                Ok((ty.to_owned(), val.to_owned()))
            }
            Expression::BinOp { lhs, op, rhs } => self.generate_binop(func, lhs, op, rhs),
            Expression::StructInitialization { name, fields } => {
                self.generate_struct_init(func, name, fields)
            }
            Expression::FieldAccess { expr, field } => {
                if let Expression::FunctionCall { fn_name, args } = field.as_ref() {
                    self.generate_method_call(func, expr, fn_name, args)
                } else {
                    self.generate_field_access(func, expr, field)
                }
            }
            Expression::ArrayAccess { name, index } => {
                // Clone to avoid borrow conflicts with the later generate_expression call
                let (_, base, ast_type) = self.get_var(name)?.clone();
                let elem_ast_type = match ast_type {
                    Some(Type::Array(inner, _)) => *inner,
                    _ => return Err(format!("'{}' is not an array", name)),
                };
                let elem_qbe_type = self.get_type(elem_ast_type)?;
                let elem_size = self.type_size(&elem_qbe_type);

                let (_, idx_val) = self.generate_expression(func, index)?;

                // Sign-extend Word index to Long for pointer arithmetic
                let idx_long = self.new_temporary();
                func.assign_instr(
                    idx_long.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Extsw(idx_val),
                );

                // scaled = index * elem_size
                let scaled = self.new_temporary();
                func.assign_instr(
                    scaled.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Mul(idx_long, qbe::Value::Const(elem_size)),
                );

                // elem_ptr = base + 8 + scaled
                let with_header = self.new_temporary();
                func.assign_instr(
                    with_header.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Add(scaled, qbe::Value::Const(8)),
                );
                let elem_ptr = self.new_temporary();
                func.assign_instr(
                    elem_ptr.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Add(base, with_header),
                );

                // Load and return the element
                let result = self.new_temporary();
                func.assign_instr(
                    result.clone(),
                    elem_qbe_type.clone(),
                    qbe::Instr::Load(elem_qbe_type.clone(), elem_ptr),
                );

                Ok((elem_qbe_type, result))
            }
        }
    }

    /// Generates an `if` statement
    fn generate_if(
        &mut self,
        func: &mut qbe::Function,
        cond: &Expression,
        if_clause: &Statement,
        else_clause: &Option<Box<Statement>>,
    ) -> GeneratorResult<()> {
        let (_, result) = self.generate_expression(func, cond)?;

        self.tmp_counter += 1;
        let if_label = format!("cond.{}.if", self.tmp_counter);
        let else_label = format!("cond.{}.else", self.tmp_counter);
        let end_label = format!("cond.{}.end", self.tmp_counter);

        func.add_instr(qbe::Instr::Jnz(
            result,
            if_label.clone(),
            if else_clause.is_some() {
                else_label.clone()
            } else {
                end_label.clone()
            },
        ));

        func.add_block(if_label);
        self.generate_statement(func, if_clause)?;

        let needs_end_block = if let Some(else_clause) = else_clause {
            let if_falls_through = !func.blocks.last().is_some_and(|b| b.jumps());
            if if_falls_through {
                func.add_instr(qbe::Instr::Jmp(end_label.clone()));
            }

            func.add_block(else_label);
            self.generate_statement(func, else_clause)?;

            let else_falls_through = !func.blocks.last().is_some_and(|b| b.jumps());
            if_falls_through || else_falls_through
        } else {
            true // if-only always needs end block for fallthrough
        };

        if needs_end_block {
            func.add_block(end_label);
        }

        Ok(())
    }

    /// Generates a `while` statement
    fn generate_while(
        &mut self,
        func: &mut qbe::Function,
        cond: &Expression,
        body: &Statement,
    ) -> GeneratorResult<()> {
        self.tmp_counter += 1;
        let cond_label = format!("loop.{}.cond", self.tmp_counter);
        let body_label = format!("loop.{}.body", self.tmp_counter);
        let end_label = format!("loop.{}.end", self.tmp_counter);

        self.loop_labels.push(format!("loop.{}", self.tmp_counter));

        func.add_block(cond_label.clone());

        let (_, result) = self.generate_expression(func, cond)?;
        func.add_instr(qbe::Instr::Jnz(
            result,
            body_label.clone(),
            end_label.clone(),
        ));

        func.add_block(body_label);
        self.generate_statement(func, body)?;

        if !func.blocks.last().is_some_and(|b| b.jumps()) {
            func.add_instr(qbe::Instr::Jmp(cond_label));
        }

        func.add_block(end_label);

        self.loop_labels.pop();

        Ok(())
    }

    /// Generates a string
    fn generate_string(&mut self, string: &str) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        self.tmp_counter += 1;
        let name = format!("string.{}", self.tmp_counter);

        let mut items: Vec<(qbe::Type, qbe::DataItem)> = Vec::new();
        let mut buf = String::new();
        for ch in string.chars() {
            if ch.is_ascii() && !ch.is_ascii_control() && ch != '"' {
                buf.push(ch)
            } else {
                if !buf.is_empty() {
                    items.push((qbe::Type::Byte, qbe::DataItem::Str(buf.clone())));
                    buf.clear();
                }

                let mut buf = [0; 4];
                let len = ch.encode_utf8(&mut buf).len();

                for b in buf.iter().take(len) {
                    items.push((qbe::Type::Byte, qbe::DataItem::Const(*b as u64)));
                }
                continue;
            }
        }
        if !buf.is_empty() {
            items.push((qbe::Type::Byte, qbe::DataItem::Str(buf)));
        }
        // NUL terminator
        items.push((qbe::Type::Byte, qbe::DataItem::Const(0)));

        let data_def = qbe::DataDef::new(qbe::Linkage::public(), name.clone(), None, items);
        self.datadefs.push(data_def);

        Ok((qbe::Type::Long, qbe::Value::Global(name)))
    }

    /// Returns the result of a binary operation (e.g. `+` or `*=`).
    fn generate_binop(
        &mut self,
        func: &mut qbe::Function,
        lhs: &Expression,
        op: &BinOp,
        rhs: &Expression,
    ) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        let (lhs_ty, lhs_val) = self.generate_expression(func, lhs)?;
        let (rhs_ty, rhs_val) = self.generate_expression(func, rhs)?;
        let tmp = self.new_temporary();

        // String concatenation: when the LHS is a string expression and the
        // operator is addition, emit a call to the _str_concat C builtin.
        // We check the AST expression rather than the QBE type because
        // both strings and arrays map to qbe::Type::Long.
        let is_string =
            matches!(op, BinOp::Addition | BinOp::AddAssign) && self.is_string_expression(lhs);

        if is_string {
            func.assign_instr(
                tmp.clone(),
                qbe::Type::Long,
                qbe::Instr::Call(
                    "_str_concat".into(),
                    vec![
                        (qbe::Type::Long, lhs_val.clone()),
                        (qbe::Type::Long, rhs_val),
                    ],
                    None,
                ),
            );

            // Handle AddAssign for strings
            if matches!(op, BinOp::AddAssign) {
                let tmp_clone = tmp.clone();
                self.generate_assignment(func, lhs, tmp_clone)?;
            }

            return Ok((qbe::Type::Long, tmp));
        }

        // String comparison: use strcmp for == and != on string operands
        if matches!(op, BinOp::Equal | BinOp::NotEqual) && self.is_string_expression(lhs) {
            let strcmp_result = self.new_temporary();
            func.assign_instr(
                strcmp_result.clone(),
                qbe::Type::Word,
                qbe::Instr::Call(
                    "_strcmp".into(),
                    vec![
                        (qbe::Type::Long, lhs_val.clone()),
                        (qbe::Type::Long, rhs_val),
                    ],
                    None,
                ),
            );
            let cmp_op = if matches!(op, BinOp::Equal) {
                qbe::Cmp::Eq
            } else {
                qbe::Cmp::Ne
            };
            func.assign_instr(
                tmp.clone(),
                qbe::Type::Word,
                qbe::Instr::Cmp(
                    qbe::Type::Word,
                    cmp_op,
                    strcmp_result,
                    qbe::Value::Const(0),
                ),
            );
            return Ok((qbe::Type::Word, tmp));
        }

        // Use the wider of the two operand types for the result
        let ty = Self::wider_type(&lhs_ty, &rhs_ty);

        // Widen operands if needed (e.g. Byte → Word via extub)
        let lhs_val = if lhs_ty == qbe::Type::Byte && ty != qbe::Type::Byte {
            let widened = self.new_temporary();
            func.assign_instr(widened.clone(), ty.clone(), qbe::Instr::Extub(lhs_val));
            widened
        } else if lhs_ty == qbe::Type::Word && ty == qbe::Type::Long {
            let widened = self.new_temporary();
            func.assign_instr(widened.clone(), ty.clone(), qbe::Instr::Extuw(lhs_val));
            widened
        } else {
            lhs_val
        };
        let rhs_val = if rhs_ty == qbe::Type::Byte && ty != qbe::Type::Byte {
            let widened = self.new_temporary();
            func.assign_instr(widened.clone(), ty.clone(), qbe::Instr::Extub(rhs_val));
            widened
        } else if rhs_ty == qbe::Type::Word && ty == qbe::Type::Long {
            let widened = self.new_temporary();
            func.assign_instr(widened.clone(), ty.clone(), qbe::Instr::Extuw(rhs_val));
            widened
        } else {
            rhs_val
        };

        func.assign_instr(
            tmp.clone(),
            ty.clone(),
            match op {
                BinOp::Addition | BinOp::AddAssign => qbe::Instr::Add(lhs_val, rhs_val),
                BinOp::Subtraction | BinOp::SubtractAssign => qbe::Instr::Sub(lhs_val, rhs_val),
                BinOp::Multiplication | BinOp::MultiplyAssign => qbe::Instr::Mul(lhs_val, rhs_val),
                BinOp::Division | BinOp::DivideAssign => qbe::Instr::Div(lhs_val, rhs_val),
                BinOp::Modulus => qbe::Instr::Rem(lhs_val, rhs_val),

                BinOp::And => qbe::Instr::And(lhs_val, rhs_val),
                BinOp::Or => qbe::Instr::Or(lhs_val, rhs_val),

                // Others should be comparisons
                cmp => qbe::Instr::Cmp(
                    ty.clone(),
                    match cmp {
                        BinOp::LessThan => qbe::Cmp::Slt,
                        BinOp::LessThanOrEqual => qbe::Cmp::Sle,
                        BinOp::GreaterThan => qbe::Cmp::Sgt,
                        BinOp::GreaterThanOrEqual => qbe::Cmp::Sge,
                        BinOp::Equal => qbe::Cmp::Eq,
                        BinOp::NotEqual => qbe::Cmp::Ne,
                        _ => unreachable!(),
                    },
                    lhs_val,
                    rhs_val,
                ),
            },
        );

        // *Assign BinOps work just like normal ones except that here the
        // result is assigned to the left hand side. This essentially makes
        // `a += 1` the same as `a = a + 1`.
        match op {
            BinOp::AddAssign
            | BinOp::SubtractAssign
            | BinOp::MultiplyAssign
            | BinOp::DivideAssign => {
                let tmp_clone = tmp.clone();
                self.generate_assignment(func, lhs, tmp_clone)?;
            }
            _ => {}
        };

        Ok((ty, tmp))
    }

    /// Generates an assignment to either a variable, field access or array
    /// access
    fn generate_assignment(
        &mut self,
        func: &mut qbe::Function,
        lhs: &Expression,
        rhs: qbe::Value,
    ) -> GeneratorResult<()> {
        match lhs {
            Expression::Variable(name) => {
                let (vty, tmp, _) = self.get_var(name)?;
                func.assign_instr(
                    tmp.to_owned(),
                    vty.to_owned(),
                    qbe::Instr::Copy(rhs),
                );
            }
            Expression::FieldAccess { expr, field } => {
                // First get all the info we need
                let access_result = self.resolve_field_access(func, expr, field)?;
                let (src, ty, offset) = access_result;

                // Then create a temporary for the field pointer
                let field_ptr = self.new_temporary();

                // Do the operations
                func.assign_instr(
                    field_ptr.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Add(src, qbe::Value::Const(offset)),
                );

                func.add_instr(qbe::Instr::Store(ty, field_ptr, rhs));
            }
            Expression::ArrayAccess { name, index } => {
                let (_, base, ast_type) = self.get_var(name)?.clone();
                let elem_ast_type = match ast_type {
                    Some(Type::Array(inner, _)) => *inner,
                    _ => return Err(format!("'{}' is not an array", name)),
                };
                let elem_qbe_type = self.get_type(elem_ast_type)?;
                let elem_size = self.type_size(&elem_qbe_type);

                let (_, idx_val) = self.generate_expression(func, index)?;

                let idx_long = self.new_temporary();
                func.assign_instr(
                    idx_long.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Extsw(idx_val),
                );

                let scaled = self.new_temporary();
                func.assign_instr(
                    scaled.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Mul(idx_long, qbe::Value::Const(elem_size)),
                );

                let with_header = self.new_temporary();
                func.assign_instr(
                    with_header.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Add(scaled, qbe::Value::Const(8)),
                );

                let elem_ptr = self.new_temporary();
                func.assign_instr(
                    elem_ptr.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Add(base, with_header),
                );

                func.add_instr(qbe::Instr::Store(elem_qbe_type, elem_ptr, rhs));
            }
            _ => return Err("Left side of an assignment must be either a variable, field access or array access".to_owned()),
        }

        Ok(())
    }

    /// Generates struct initialization
    fn generate_struct_init(
        &mut self,
        func: &mut qbe::Function,
        name: &str,
        fields: &HashMap<String, Box<Expression>>,
    ) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        // Get the struct info first
        let (ty, meta, size) = self
            .struct_map
            .get(name)
            .ok_or_else(|| format!("Initialization of undeclared struct '{}'", name))?
            .to_owned();

        // Allocate space for the struct
        let base = self.new_temporary();
        func.assign_instr(base.clone(), qbe::Type::Long, qbe::Instr::Alloc8(size));

        // Initialize each field
        for (name, expr) in fields {
            // Get field info
            let (field_type, offset, _) = meta
                .get(name)
                .ok_or_else(|| format!("Unknown field '{}'", name))?
                .clone();

            // Generate expression for field value
            let (expr_type, expr_tmp) = self.generate_expression(func, expr)?;

            match expr_type {
                qbe::Type::Aggregate(_) => {
                    let field_tmp = self.new_temporary();
                    func.assign_instr(
                        field_tmp.clone(),
                        qbe::Type::Long,
                        qbe::Instr::Add(base.clone(), qbe::Value::Const(offset)),
                    );
                    let sz = self.type_size(&expr_type);
                    func.add_instr(qbe::Instr::Blit(expr_tmp, field_tmp, sz));
                }
                _ => {
                    let field_tmp = self.new_temporary();
                    func.assign_instr(
                        field_tmp.clone(),
                        qbe::Type::Long,
                        qbe::Instr::Add(base.clone(), qbe::Value::Const(offset)),
                    );

                    func.add_instr(qbe::Instr::Store(field_type, field_tmp, expr_tmp));
                }
            }
        }

        Ok((ty, base))
    }

    /// Retrieves the result of struct field access
    fn generate_field_access(
        &mut self,
        func: &mut qbe::Function,
        obj: &Expression,
        field: &Expression,
    ) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        // Get the field info first
        let access_result = self.resolve_field_access(func, obj, field)?;
        let (src, ty, offset) = access_result;

        // Create a temporary for the field pointer
        let field_ptr = self.new_temporary();
        func.assign_instr(
            field_ptr.clone(),
            qbe::Type::Long,
            qbe::Instr::Add(src, qbe::Value::Const(offset)),
        );

        // Load the field value
        let tmp = self.new_temporary();
        func.assign_instr(
            tmp.clone(),
            ty.clone(),
            qbe::Instr::Load(ty.clone(), field_ptr),
        );

        Ok((ty, tmp))
    }

    /// Resolves an expression that should evaluate to a struct value,
    /// returning `(source_value, struct_name, base_offset)`.
    fn resolve_struct_expr(
        &mut self,
        func: &mut qbe::Function,
        expr: &Expression,
    ) -> GeneratorResult<(qbe::Value, String, u64)> {
        match expr {
            Expression::Variable(var) => {
                let (_, src, ast_type) = self.get_var(var)?.to_owned();
                match ast_type {
                    Some(Type::Struct(name)) => Ok((src, name, 0)),
                    _ => Err(format!("Variable '{}' is not a struct", var)),
                }
            }
            Expression::Selff => {
                let (_, src, ast_type) = self.get_var("self")?.to_owned();
                match ast_type {
                    Some(Type::Struct(name)) => Ok((src, name, 0)),
                    _ => Err("'self' must refer to a struct".to_owned()),
                }
            }
            Expression::FunctionCall { fn_name, .. } => {
                let ast_ret = self.fn_ast_signatures.get(fn_name).cloned().flatten();
                match ast_ret {
                    Some(Type::Struct(struct_name)) => {
                        let (_, val) = self.generate_expression(func, expr)?;
                        Ok((val, struct_name, 0))
                    }
                    _ => Err(format!("Function '{}' does not return a struct", fn_name)),
                }
            }
            Expression::FieldAccess { expr, field } => {
                let (src, parent_name, parent_off) = self.resolve_struct_expr(func, expr)?;
                let field_name = match field.as_ref() {
                    Expression::Variable(v) => v,
                    _ => unreachable!(),
                };
                let (_, meta, _) = self
                    .struct_map
                    .get(&parent_name)
                    .ok_or_else(|| format!("Unknown struct '{}'", parent_name))?;
                let (_, field_off, ast_type) = meta
                    .get(field_name.as_str())
                    .ok_or_else(|| {
                        format!("No field '{}' on struct '{}'", field_name, parent_name)
                    })?
                    .to_owned();
                match ast_type {
                    Some(Type::Struct(name)) => Ok((src, name, parent_off + field_off)),
                    _ => Err(format!(
                        "Field '{}' on struct '{}' is not a struct",
                        field_name, parent_name
                    )),
                }
            }
            other => Err(format!(
                "Invalid field access type: expected variable, field access or 'self', got {:?}",
                other,
            )),
        }
    }

    /// Retrieves `(source, field_type, offset)` from field access expression
    fn resolve_field_access(
        &mut self,
        func: &mut qbe::Function,
        obj: &Expression,
        field: &Expression,
    ) -> GeneratorResult<(qbe::Value, qbe::Type, u64)> {
        let (src, struct_name, base_off) = self.resolve_struct_expr(func, obj)?;

        let field_name = match field {
            Expression::Variable(v) => v,
            Expression::FunctionCall { .. } => {
                unreachable!("method calls should be intercepted in generate_expression")
            }
            _ => unreachable!(),
        };

        let (_, meta, _) = self
            .struct_map
            .get(&struct_name)
            .ok_or_else(|| format!("Unknown struct '{}'", struct_name))?;

        let (ty, field_off, _) = meta
            .get(field_name.as_str())
            .ok_or_else(|| format!("No field '{}' on struct {}", field_name, struct_name))?
            .to_owned();

        Ok((src, ty, base_off + field_off))
    }

    /// Generates a `for` loop (for-in over an array)
    fn generate_for_loop(
        &mut self,
        func: &mut qbe::Function,
        ident: &Variable,
        expr: &Expression,
        body: &Statement,
    ) -> GeneratorResult<()> {
        // Element type from ident's declared type
        let elem_ast_type = ident
            .ty
            .as_ref()
            .ok_or_else(|| format!("Missing type for for-loop variable '{}'", ident.name))?
            .clone();
        let elem_type = self.get_type(elem_ast_type.clone())?;
        let elem_size = self.type_size(&elem_type);

        // Generate array expression -> base pointer
        let (_, base_ptr) = self.generate_expression(func, expr)?;

        // Load length (Long) from array header (offset 0)
        let len_tmp = self.new_temporary();
        func.assign_instr(
            len_tmp.clone(),
            qbe::Type::Long,
            qbe::Instr::Load(qbe::Type::Long, base_ptr.clone()),
        );

        // Set up loop labels
        self.tmp_counter += 1;
        let loop_n = self.tmp_counter;
        let cond_label = format!("loop.{}.cond", loop_n);
        let body_label = format!("loop.{}.body", loop_n);
        let end_label = format!("loop.{}.end", loop_n);
        self.loop_labels.push(format!("loop.{}", loop_n));

        // Push a scope for counter + ident variables
        self.scopes.push(HashMap::new());

        // Declare counter (Long) initialized to 0
        let counter_name = format!("__for_{}_counter", loop_n);
        let counter_tmp = self.new_var(&qbe::Type::Long, &counter_name, None)?;
        func.assign_instr(
            counter_tmp.clone(),
            qbe::Type::Long,
            qbe::Instr::Copy(qbe::Value::Const(0)),
        );

        // Declare ident variable (will be overwritten each iteration)
        let ident_tmp = self.new_var(&elem_type, &ident.name, Some(elem_ast_type))?;
        func.assign_instr(
            ident_tmp.clone(),
            elem_type.clone(),
            qbe::Instr::Copy(qbe::Value::Const(0)),
        );

        // Condition block: counter < len
        func.add_block(cond_label.clone());
        let cmp_tmp = self.new_temporary();
        func.assign_instr(
            cmp_tmp.clone(),
            qbe::Type::Word,
            qbe::Instr::Cmp(qbe::Type::Long, qbe::Cmp::Slt, counter_tmp.clone(), len_tmp),
        );
        func.add_instr(qbe::Instr::Jnz(
            cmp_tmp,
            body_label.clone(),
            end_label.clone(),
        ));

        // Body block: load arr[counter] into ident, run body, increment
        func.add_block(body_label);

        // element_ptr = base + 8 + counter * elem_size
        let off = self.new_temporary();
        func.assign_instr(
            off.clone(),
            qbe::Type::Long,
            qbe::Instr::Mul(counter_tmp.clone(), qbe::Value::Const(elem_size)),
        );
        let off2 = self.new_temporary();
        func.assign_instr(
            off2.clone(),
            qbe::Type::Long,
            qbe::Instr::Add(off, qbe::Value::Const(8)),
        );
        let elem_ptr = self.new_temporary();
        func.assign_instr(
            elem_ptr.clone(),
            qbe::Type::Long,
            qbe::Instr::Add(base_ptr, off2),
        );
        let elem_val = self.new_temporary();
        func.assign_instr(
            elem_val.clone(),
            elem_type.clone(),
            qbe::Instr::Load(elem_type.clone(), elem_ptr),
        );
        // Assign loaded value to ident
        func.assign_instr(ident_tmp, elem_type, qbe::Instr::Copy(elem_val));

        // Execute loop body
        self.generate_statement(func, body)?;

        // Increment counter and jump to cond (unless body already jumped)
        if !func.blocks.last().is_some_and(|b| b.jumps()) {
            let inc = self.new_temporary();
            func.assign_instr(
                inc.clone(),
                qbe::Type::Long,
                qbe::Instr::Add(counter_tmp.clone(), qbe::Value::Const(1)),
            );
            func.assign_instr(counter_tmp, qbe::Type::Long, qbe::Instr::Copy(inc));
            func.add_instr(qbe::Instr::Jmp(cond_label));
        }

        // End block
        func.add_block(end_label);

        self.scopes.pop();
        self.loop_labels.pop();

        Ok(())
    }

    /// Generates an array literal
    fn generate_array(
        &mut self,
        func: &mut qbe::Function,
        len: usize,
        items: &[Expression],
    ) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        let mut first_type: Option<qbe::Type> = None;
        let mut results: Vec<qbe::Value> = Vec::new();

        // First collect all item expressions to avoid borrowing issues
        let mut item_results = Vec::new();
        for item in items.iter() {
            let result = self.generate_expression(func, item)?;
            item_results.push(result);
        }

        // Then process the results
        for (ty, result) in item_results {
            results.push(result);

            if let Some(ref first_type) = first_type {
                if ty != *first_type {
                    return Err(format!(
                        "Inconsistent array types {:?} and {:?} (possibly more)",
                        first_type, ty
                    ));
                }
            } else {
                first_type = Some(ty);
            }
        }

        // Arrays have the following in-memory representation:
        // {
        //    length (long),
        //    values...
        // }
        let tmp = self.new_temporary();
        func.assign_instr(
            tmp.clone(),
            qbe::Type::Long,
            qbe::Instr::Alloc8(
                8 + if let Some(ref ty) = first_type {
                    self.type_size(ty) * (len as u64)
                } else {
                    0
                },
            ),
        );
        func.add_instr(qbe::Instr::Store(
            qbe::Type::Long,
            tmp.clone(),
            qbe::Value::Const(len as u64),
        ));

        for (i, value) in results.iter().enumerate() {
            let value_ptr = self.new_temporary();
            func.assign_instr(
                value_ptr.clone(),
                qbe::Type::Long,
                qbe::Instr::Add(
                    tmp.clone(),
                    qbe::Value::Const(
                        8 + (i as u64) * self.type_size(first_type.as_ref().unwrap()),
                    ),
                ),
            );

            func.add_instr(qbe::Instr::Store(
                first_type.as_ref().unwrap().clone(),
                value_ptr,
                value.to_owned(),
            ));
        }

        // Create a typedef for the array
        self.tmp_counter += 1;
        let name = format!("array.{}", self.tmp_counter);

        let typedef = qbe::TypeDef::Regular {
            ident: name.clone(),
            align: None,
            items: if let Some(ty) = first_type {
                vec![(qbe::Type::Long, 1), (ty, len)]
            } else {
                // No elements
                vec![(qbe::Type::Long, 1)]
            },
        };

        // Create a reference to the registered typedef
        let typedef_arc = Arc::new(typedef);
        self.module.add_type(Arc::clone(&typedef_arc));
        self.typedefs.push(typedef_arc);

        // Create an aggregate type using the typedef
        let array_type = qbe::Type::aggregate(self.typedefs.last().unwrap());

        Ok((array_type, tmp))
    }

    /// Returns a new unique temporary
    fn new_temporary(&mut self) -> qbe::Value {
        self.tmp_counter += 1;
        qbe::Value::Temporary(format!("tmp.{}", self.tmp_counter))
    }

    /// Returns a new temporary bound to a variable
    fn new_var(
        &mut self,
        ty: &qbe::Type,
        name: &str,
        ast_type: Option<Type>,
    ) -> GeneratorResult<qbe::Value> {
        if self.get_var(name).is_ok() {
            return Err(format!("Re-declaration of variable '{}'", name));
        }

        let tmp = self.new_temporary();

        let scope = self
            .scopes
            .last_mut()
            .expect("expected last scope to be present");
        scope.insert(name.to_owned(), (ty.to_owned(), tmp.to_owned(), ast_type));

        Ok(tmp)
    }

    /// Returns a temporary associated to a variable
    fn get_var(&self, name: &str) -> GeneratorResult<&VarInfo> {
        self.scopes
            .iter()
            .rev()
            .filter_map(|s| s.get(name))
            .next()
            .ok_or_else(|| format!("Undefined variable '{}'", name))
    }

    /// Returns a QBE type for the given AST type
    fn get_type(&self, ty: Type) -> GeneratorResult<qbe::Type> {
        match ty {
            Type::Any => Ok(qbe::Type::Long),
            Type::Int => Ok(qbe::Type::Word),
            Type::Bool => Ok(qbe::Type::Word),
            Type::Str => Ok(qbe::Type::Long),
            Type::Struct(name) => {
                let (ty, ..) = self
                    .struct_map
                    .get(&name)
                    .ok_or_else(|| format!("Use of undeclared struct '{}'", name))?
                    .to_owned();
                Ok(ty)
            }
            Type::Array(..) => Ok(qbe::Type::Long),
        }
    }

    // Returns a size, in bytes of a type
    fn type_size(&self, ty: &qbe::Type) -> u64 {
        ty.size()
    }

    /// Checks whether an AST expression is known to produce a string value.
    /// This uses the AST type stored in scope to distinguish strings from
    /// arrays, since both map to qbe::Type::Long.
    fn is_string_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Str(_) => true,
            Expression::Variable(name) => self
                .get_var(name)
                .map(|(_, _, ast_ty)| matches!(ast_ty, Some(Type::Str)))
                .unwrap_or(false),
            Expression::BinOp { lhs, op, .. } => {
                matches!(op, BinOp::Addition | BinOp::AddAssign) && self.is_string_expression(lhs)
            }
            Expression::FieldAccess { expr, field } => {
                if let Expression::Variable(field_name) = field.as_ref() {
                    if let Ok(struct_name) = self.get_struct_name_of(expr) {
                        if let Some((_, meta, _)) = self.struct_map.get(&struct_name) {
                            if let Some((_, _, ast_type)) = meta.get(field_name) {
                                return matches!(ast_type, Some(Type::Str));
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Returns the Antimony struct name for an expression that evaluates to a struct pointer
    fn get_struct_name_of(&self, expr: &Expression) -> GeneratorResult<String> {
        match expr {
            Expression::Variable(name) => {
                let (_, _, ast_type) = self.get_var(name)?;
                match ast_type {
                    Some(Type::Struct(s)) => Ok(s.clone()),
                    _ => Err(format!("'{}' is not a struct", name)),
                }
            }
            Expression::Selff => {
                let (_, _, ast_type) = self.get_var("self")?;
                match ast_type {
                    Some(Type::Struct(s)) => Ok(s.clone()),
                    _ => Err("'self' does not refer to a struct".to_owned()),
                }
            }
            _ => Err("Cannot determine struct type for complex expression".to_owned()),
        }
    }
}
