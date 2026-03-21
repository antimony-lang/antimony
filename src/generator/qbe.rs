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
use std::rc::Rc;

// Use Rc to avoid lifetimes in some of the tricky spots
// TODO: This should be fixed in the QBE library
type RcTypeDef = Rc<qbe::TypeDef<'static>>;

/// Information stored for each variable in scope
type VarInfo = (qbe::Type<'static>, qbe::Value, Option<Type>);

pub struct QbeGenerator {
    /// Counter for unique temporary names
    tmp_counter: u32,
    /// Block-scoped variable -> (qbe_type, temporary, ast_type) mappings
    scopes: Vec<HashMap<String, VarInfo>>,
    /// Structure -> (type, meta data, size) mappings
    struct_map: HashMap<String, (qbe::Type<'static>, StructMeta, u64)>,
    /// Label prefix of loop scopes
    loop_labels: Vec<String>,
    /// Data defintions collected during generation
    datadefs: Vec<qbe::DataDef<'static>>,
    /// Type defintions collected during generation
    typedefs: Vec<RcTypeDef>,
    /// Function name -> return type (populated by pre-pass before codegen)
    fn_signatures: HashMap<String, Option<qbe::Type<'static>>>,
    /// Function name -> parameter types (populated by pre-pass before codegen)
    fn_param_types: HashMap<String, Vec<qbe::Type<'static>>>,
    /// Module being built
    module: qbe::Module<'static>,
}

/// Mapping of field -> (type, offset, ast_type)
type StructMeta = HashMap<String, (qbe::Type<'static>, u64, Option<Type>)>;

impl Generator for QbeGenerator {
    fn generate(prog: Module) -> GeneratorResult<String> {
        let mut generator = QbeGenerator {
            tmp_counter: 0,
            scopes: Vec::new(),
            struct_map: HashMap::new(),
            loop_labels: Vec::new(),
            datadefs: Vec::new(),
            typedefs: Vec::new(),
            fn_signatures: HashMap::new(),
            fn_param_types: HashMap::new(),
            module: qbe::Module::new(),
        };

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

            let typedef_rc = Rc::new(structure);
            generator.module.add_type((*typedef_rc).clone());
            generator.typedefs.push(typedef_rc);

            // Replace the Word placeholder in struct_map with the proper Aggregate type
            let struct_type = unsafe {
                std::mem::transmute::<qbe::Type<'_>, qbe::Type<'static>>(qbe::Type::Aggregate(
                    generator.typedefs.last().unwrap(),
                ))
            };
            if let Some(entry) = generator.struct_map.get_mut(&def.name) {
                entry.0 = struct_type;
            }
        }

        // Pre-pass: collect function return types so callers know what type to expect
        for func in &prog.func {
            let ret_type = if let Some(ty) = &func.ret_type {
                Some(generator.get_type(ty.to_owned())?.into_abi())
            } else {
                None
            };
            generator.fn_signatures.insert(func.name.clone(), ret_type);

            let param_types: Vec<qbe::Type<'static>> = func
                .arguments
                .iter()
                .filter_map(|arg| arg.ty.as_ref())
                .map(|ty| generator.get_type(ty.to_owned()).map(|t| t.into_abi()))
                .collect::<Result<Vec<_>, _>>()?;
            generator
                .fn_param_types
                .insert(func.name.clone(), param_types);
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

        Ok(generator.module.to_string())
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
                qbe::Type::Aggregate(td) => match td {
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

    /// Calculate the aligned offset for a field
    fn align_offset(&self, offset: u64, alignment: u64) -> u64 {
        (offset + alignment - 1) & !(alignment - 1)
    }

    /// Returns an aggregate type for a structure (note: has side effects)
    fn generate_struct(&mut self, def: &StructDef) -> GeneratorResult<qbe::TypeDef<'static>> {
        self.tmp_counter += 1;
        let ident = format!("struct.{}", self.tmp_counter);
        let mut items: Vec<(qbe::Type<'static>, usize)> = Vec::new();
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

    fn generate_function(&mut self, func: &Function) -> GeneratorResult<qbe::Function<'static>> {
        // Function argument scope
        self.scopes.push(HashMap::new());

        let mut arguments: Vec<(qbe::Type<'static>, qbe::Value)> = Vec::new();
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

        let return_ty = if let Some(ty) = &func.ret_type {
            Some(self.get_type(ty.to_owned())?.into_abi())
        } else {
            None
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
        let returns = qfunc.blocks.last().is_some_and(|b| {
            b.items.last().is_some_and(|item| {
                matches!(
                    item,
                    qbe::BlockItem::Statement(qbe::Statement::Volatile(qbe::Instr::Ret(_)))
                )
            })
        });

        // Automatically add return in void functions unless it already returns
        if !returns {
            if func.ret_type.is_none() {
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
    ) -> GeneratorResult<qbe::Function<'static>> {
        self.scopes.push(HashMap::new());

        // Prepend self as first argument (Long pointer to struct)
        let self_tmp = self.new_var(
            &qbe::Type::Long,
            "self",
            Some(Type::Struct(struct_def.name.clone())),
        )?;
        let mut arguments: Vec<(qbe::Type<'static>, qbe::Value)> =
            vec![(qbe::Type::Long, self_tmp)];

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

        let returns = qfunc.blocks.last().is_some_and(|b| {
            b.items.last().is_some_and(|item| {
                matches!(
                    item,
                    qbe::BlockItem::Statement(qbe::Statement::Volatile(qbe::Instr::Ret(_)))
                )
            })
        });

        if !returns {
            if method.ret_type.is_none() {
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
        func: &mut qbe::Function<'static>,
        obj: &Expression,
        method_name: &str,
        args: &[Expression],
    ) -> GeneratorResult<(qbe::Type<'static>, qbe::Value)> {
        let struct_name = self.get_struct_name_of(obj)?;
        let (_, obj_ptr) = self.generate_expression(func, obj)?;

        let mut call_args: Vec<(qbe::Type<'static>, qbe::Value)> = vec![(qbe::Type::Long, obj_ptr)];
        for arg in args {
            let result = self.generate_expression(func, arg)?;
            call_args.push(result);
        }

        let mangled_name = format!("{}_{}", struct_name, method_name);
        let ret_type = self
            .fn_signatures
            .get(&mangled_name)
            .cloned()
            .flatten()
            .unwrap_or(qbe::Type::Word);

        let tmp = self.new_temporary();
        func.assign_instr(
            tmp.clone(),
            ret_type.clone(),
            qbe::Instr::Call(mangled_name, call_args, None),
        );

        Ok((ret_type, tmp))
    }

    /// Generates a statement
    fn generate_statement(
        &mut self,
        func: &mut qbe::Function<'static>,
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
                let tmp = self.new_var(&ty, &variable.name, Some(ast_type))?;

                if let Some(expr) = value {
                    let (expr_type, expr_value) = self.generate_expression(func, expr)?;
                    func.assign_instr(tmp, expr_type, qbe::Instr::Copy(expr_value));
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

    /// Generates an expression
    fn generate_expression(
        &mut self,
        func: &mut qbe::Function<'static>,
        expr: &Expression,
    ) -> GeneratorResult<(qbe::Type<'static>, qbe::Value)> {
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
                // Collect arguments first to avoid multiple mutable borrows
                let mut arg_results = Vec::new();
                for arg in args.iter() {
                    let result = self.generate_expression(func, arg)?;
                    arg_results.push(result);
                }

                let tmp = self.new_temporary();

                // Widen arguments if the callee expects a larger type (e.g. Type::Any → Long)
                let param_types_opt = self.fn_param_types.get(fn_name).cloned();
                let mut new_args: Vec<(qbe::Type<'static>, qbe::Value)> = Vec::new();
                for (i, (arg_ty, arg_val)) in arg_results.into_iter().enumerate() {
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

                // Look up the return type from the pre-pass signature map
                let ret_type = self
                    .fn_signatures
                    .get(fn_name)
                    .cloned()
                    .flatten()
                    .unwrap_or(qbe::Type::Word);

                func.assign_instr(
                    tmp.clone(),
                    ret_type.clone(),
                    qbe::Instr::Call(fn_name.clone(), new_args, None),
                );

                Ok((ret_type, tmp))
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
        func: &mut qbe::Function<'static>,
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

        if let Some(else_clause) = else_clause {
            // Jump over to the end to prevent fallthrough into else
            // clause, unless the last block already jumps
            if !func.blocks.last().is_some_and(|b| b.jumps()) {
                func.add_instr(qbe::Instr::Jmp(end_label.clone()));
            }

            func.add_block(else_label);
            self.generate_statement(func, else_clause)?;
        }

        func.add_block(end_label);

        Ok(())
    }

    /// Generates a `while` statement
    fn generate_while(
        &mut self,
        func: &mut qbe::Function<'static>,
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
    fn generate_string(
        &mut self,
        string: &str,
    ) -> GeneratorResult<(qbe::Type<'static>, qbe::Value)> {
        self.tmp_counter += 1;
        let name = format!("string.{}", self.tmp_counter);

        let mut items: Vec<(qbe::Type<'static>, qbe::DataItem)> = Vec::new();
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
        func: &mut qbe::Function<'static>,
        lhs: &Expression,
        op: &BinOp,
        rhs: &Expression,
    ) -> GeneratorResult<(qbe::Type<'static>, qbe::Value)> {
        let (_, lhs_val) = self.generate_expression(func, lhs)?;
        let (_, rhs_val) = self.generate_expression(func, rhs)?;
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

        // TODO: take the biggest
        let ty = qbe::Type::Word;

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
        func: &mut qbe::Function<'static>,
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
                let access_result = self.resolve_field_access(expr, field)?;
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
        func: &mut qbe::Function<'static>,
        name: &str,
        fields: &HashMap<String, Box<Expression>>,
    ) -> GeneratorResult<(qbe::Type<'static>, qbe::Value)> {
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
                    // TODO: avoid memcpy here
                    func.add_instr(qbe::Instr::Call(
                        "memcpy".into(),
                        vec![
                            (qbe::Type::Long, field_tmp),
                            (qbe::Type::Long, expr_tmp),
                            (qbe::Type::Long, qbe::Value::Const(sz)),
                        ],
                        None,
                    ));
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
        func: &mut qbe::Function<'static>,
        obj: &Expression,
        field: &Expression,
    ) -> GeneratorResult<(qbe::Type<'static>, qbe::Value)> {
        // Get the field info first
        let access_result = self.resolve_field_access(obj, field)?;
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

    /// Retrieves `(source, offset)` from field access expression
    fn resolve_field_access(
        &mut self,
        obj: &Expression,
        field: &Expression,
    ) -> GeneratorResult<(qbe::Value, qbe::Type<'static>, u64)> {
        let (src, ty, off) = match obj {
            Expression::Variable(var) => {
                let (ty, src, _) = self.get_var(var)?.to_owned();
                (src, ty, 0)
            }
            Expression::FieldAccess { expr, field } => self.resolve_field_access(expr, field)?,
            Expression::Selff => {
                let (_, src, ast_type) = self.get_var("self")?.to_owned();
                let struct_name = match ast_type {
                    Some(Type::Struct(name)) => name,
                    _ => return Err("'self' must refer to a struct".to_owned()),
                };
                let (ty, ..) = self
                    .struct_map
                    .get(&struct_name)
                    .ok_or_else(|| format!("Unknown struct '{}'", struct_name))?
                    .to_owned();
                (src, ty, 0)
            }
            other => {
                return Err(format!(
                    "Invalid field access type: expected variable, field access or 'self', got {:?}",
                    other,
                ));
            }
        };
        let field = match field {
            Expression::Variable(v) => v,
            Expression::FunctionCall { .. } => {
                unreachable!("method calls should be intercepted in generate_expression")
            }
            // Parser should ensure this won't happen
            _ => unreachable!(),
        };

        // XXX: this is very hacky and inefficient
        let (name, meta) = self
            .struct_map
            .iter()
            .filter_map(
                |(name, (sty, meta, _))| {
                    if ty == *sty {
                        Some((name, meta))
                    } else {
                        None
                    }
                },
            )
            .next()
            .unwrap();

        let (ty, offset, _) = meta
            .get(field)
            .ok_or_else(|| format!("No field '{}' on struct {}", field, name))?
            .to_owned();

        Ok((src, ty, offset + off))
    }

    /// Generates a `for` loop (for-in over an array)
    fn generate_for_loop(
        &mut self,
        func: &mut qbe::Function<'static>,
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
        func: &mut qbe::Function<'static>,
        len: usize,
        items: &[Expression],
    ) -> GeneratorResult<(qbe::Type<'static>, qbe::Value)> {
        let mut first_type: Option<qbe::Type<'static>> = None;
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
        let typedef_rc = Rc::new(typedef);
        self.module.add_type((*typedef_rc).clone());
        self.typedefs.push(typedef_rc);

        // Create an aggregate type using the typedef
        let array_type = unsafe {
            // SAFETY: Using Rc to ensure the TypeDef outlives the reference
            std::mem::transmute::<qbe::Type<'_>, qbe::Type<'static>>(qbe::Type::Aggregate(
                self.typedefs.last().unwrap(),
            ))
        };

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
        ty: &qbe::Type<'static>,
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
    fn get_type(&self, ty: Type) -> GeneratorResult<qbe::Type<'static>> {
        match ty {
            Type::Any => Ok(qbe::Type::Long),
            Type::Int => Ok(qbe::Type::Word),
            Type::Bool => Ok(qbe::Type::Byte),
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
