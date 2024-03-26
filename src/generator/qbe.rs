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
use crate::ast::types::{Type, TypeKind};
use crate::ast::*;
use crate::util::Either;
use std::collections::HashMap;

pub struct QbeGenerator {
    /// Counter for unique temporary names
    tmp_counter: u32,
    /// Block-scoped variable -> temporary mappings
    scopes: Vec<HashMap<String, (qbe::Type, qbe::Value)>>,
    /// Structure -> (type, meta data, size) mappings
    struct_map: HashMap<String, (qbe::Type, StructMeta, u64)>,
    /// Label prefix of loop scopes
    loop_labels: Vec<String>,
    /// Data defintions collected during generation
    datadefs: Vec<qbe::DataDef>,
    /// Type defintions collected during generation
    typedefs: Vec<qbe::TypeDef>,
}

/// Mapping of field -> (type, offset)
type StructMeta = HashMap<String, (qbe::Type, u64)>;

impl Generator for QbeGenerator {
    fn generate(prog: Module) -> GeneratorResult<String> {
        let mut generator = QbeGenerator {
            tmp_counter: 0,
            scopes: Vec::new(),
            struct_map: HashMap::new(),
            loop_labels: Vec::new(),
            datadefs: Vec::new(),
            typedefs: Vec::new(),
        };

        // TODO: use `qbe::Module` API instead of writing to the buffer directly
        let mut buf = String::new();

        for def in &prog.structs {
            let structure = generator.generate_struct(def)?;

            #[cfg(debug_assertions)]
            {
                // Just in case it incorrectly calculates offsets
                let (_, meta, size) = generator.struct_map.get(&def.name).unwrap();
                buf.push_str(&format!("# size: {}\n", size));
                buf.push_str(&format!("# meta: {:?}\n", meta));
            }
            buf.push_str(&format!("{}\n", structure));
        }

        for func in &prog.func {
            if func.body.is_some() {
                let func = generator.generate_function(func)?;
                buf.push_str(&format!("{}\n", func));
            }
        }

        for def in &generator.typedefs {
            buf.push_str(&format!("{}\n", def));
        }

        for def in &generator.datadefs {
            buf.push_str(&format!("{}\n", def));
        }

        Ok(buf)
    }
}

impl QbeGenerator {
    /// Returns an aggregate type for a structure (note: has side effects)
    fn generate_struct(&mut self, def: &StructDef) -> GeneratorResult<qbe::TypeDef> {
        self.tmp_counter += 1;
        let mut typedef = qbe::TypeDef {
            name: format!("struct.{}", self.tmp_counter),
            align: None,
            items: Vec::new(),
        };
        let mut meta: StructMeta = StructMeta::new();
        let mut offset = 0_u64;

        for field in &def.fields {
            let ty = self.get_type(&field.ty)?;

            meta.insert(field.name.clone(), (ty.clone(), offset));
            typedef.items.push((ty.clone(), 1));

            offset += ty.size();
        }
        self.struct_map.insert(
            def.name.clone(),
            (qbe::Type::Aggregate(typedef.name.clone()), meta, offset),
        );

        Ok(typedef)
    }

    fn generate_function(&mut self, func: &Function) -> GeneratorResult<qbe::Function> {
        // Function argument scope
        self.scopes.push(HashMap::new());

        let callable = &func.callable;
        let mut arguments: Vec<(qbe::Type, qbe::Value)> = Vec::new();
        for arg in &callable.arguments {
            let ty = self.get_type(&arg.ty)?;
            let tmp = self.new_var(&ty, &arg.name)?;

            arguments.push((ty.into_abi(), tmp));
        }

        let return_ty = if let Some(ty) = &callable.ret_type {
            Some(self.get_type(ty)?.into_abi())
        } else {
            None
        };

        let mut qfunc = qbe::Function {
            linkage: qbe::Linkage::public(),
            name: callable.name.clone(),
            arguments,
            return_ty,
            blocks: Vec::new(),
        };

        qfunc.add_block("start".to_owned());

        self.generate_statement(&mut qfunc, func.body.as_ref().unwrap())?;

        let returns = qfunc.last_block().statements.last().map_or(false, |i| {
            matches!(i, qbe::Statement::Volatile(qbe::Instr::Ret(_)))
        });
        // Automatically add return in void functions unless it already returns,
        // non-void functions raise an error
        if !returns {
            if callable.ret_type.is_none() {
                qfunc.add_instr(qbe::Instr::Ret(None));
            } else {
                return Err(format!(
                    "Function '{}' does not return in all code paths",
                    &callable.name
                ));
            }
        }

        self.scopes.pop();

        Ok(qfunc)
    }

    /// Generates a statement
    fn generate_statement(
        &mut self,
        func: &mut qbe::Function,
        stmt: &Statement,
    ) -> GeneratorResult<()> {
        match &stmt.kind {
            StatementKind::Block {
                statements,
                scope: _,
            } => {
                self.scopes.push(HashMap::new());
                for stmt in statements.iter() {
                    self.generate_statement(func, stmt)?;
                }
                self.scopes.pop();
            }
            StatementKind::Declare { variable, value } => {
                let ty =
                    self.get_type(variable.ty.as_ref().ok_or_else(|| {
                        format!("Missing type for variable '{}'", &variable.name)
                    })?)?;
                let tmp = self.new_var(&ty, &variable.name)?;

                if let Some(expr) = value {
                    let (ty, result) = self.generate_expression(func, expr)?;
                    func.assign_instr(tmp, ty, qbe::Instr::Copy(result));
                }
            }
            StatementKind::Assign { lhs, op, rhs } => {
                self.generate_assignment(func, lhs, *op, Either::Right(rhs))?;
            }
            StatementKind::Return(val) => match val {
                Some(expr) => {
                    let (_, result) = self.generate_expression(func, expr)?;
                    // TODO: Cast to function return type
                    func.add_instr(qbe::Instr::Ret(Some(result)));
                }
                None => func.add_instr(qbe::Instr::Ret(None)),
            },
            StatementKind::If {
                condition,
                body,
                else_branch,
            } => {
                self.generate_if(func, condition, body, else_branch)?;
            }
            StatementKind::While { condition, body } => {
                self.generate_while(func, condition, body)?;
            }
            StatementKind::Break => {
                if let Some(label) = &self.loop_labels.last() {
                    func.add_instr(qbe::Instr::Jmp(format!("{}.end", label)));
                } else {
                    return Err("break used outside of a loop".to_owned());
                }
            }
            StatementKind::Continue => {
                if let Some(label) = &self.loop_labels.last() {
                    func.add_instr(qbe::Instr::Jmp(format!("{}.cond", label)));
                } else {
                    return Err("continue used outside of a loop".to_owned());
                }
            }
            StatementKind::Exp(expr) => {
                self.generate_expression(func, expr)?;
            }
            _ => todo!("statement: {:?}", stmt),
        }
        Ok(())
    }

    /// Generates an expression
    fn generate_expression(
        &mut self,
        func: &mut qbe::Function,
        expr: &Expression,
    ) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        match &expr.kind {
            ExpressionKind::Int(literal) => {
                let tmp = self.new_temporary();
                func.assign_instr(
                    tmp.clone(),
                    qbe::Type::Word,
                    qbe::Instr::Copy(qbe::Value::Const(*literal as u64)),
                );

                Ok((qbe::Type::Word, tmp))
            }
            ExpressionKind::Str(string) => self.generate_string(string),
            ExpressionKind::Bool(literal) => {
                let tmp = self.new_temporary();
                func.assign_instr(
                    tmp.clone(),
                    qbe::Type::Word,
                    qbe::Instr::Copy(qbe::Value::Const(u64::from(*literal))),
                );

                Ok((qbe::Type::Word, tmp))
            }
            ExpressionKind::Array(elements) => self.generate_array(func, elements),
            ExpressionKind::FunctionCall { expr, args } => {
                let mut new_args: Vec<(qbe::Type, qbe::Value)> = Vec::new();
                for arg in args.iter() {
                    new_args.push(self.generate_expression(func, arg)?);
                }

                let fn_name = match &expr.as_ref().kind {
                    ExpressionKind::Variable(name) => name.to_owned(),
                    _ => todo!("methods"),
                };

                let tmp = self.new_temporary();
                func.assign_instr(
                    tmp.clone(),
                    // TODO: get that type properly
                    qbe::Type::Word,
                    qbe::Instr::Call(fn_name, new_args),
                );

                Ok((qbe::Type::Word, tmp))
            }
            ExpressionKind::Variable(name) => self.get_var(name).map(|v| v.to_owned()),
            ExpressionKind::BinOp { lhs, op, rhs } => self.generate_binop(func, lhs, op, rhs),
            ExpressionKind::StructInitialization { name, fields } => {
                self.generate_struct_init(func, name, fields)
            }
            ExpressionKind::FieldAccess { expr, field } => {
                self.generate_field_access(func, expr, field)
            }
            _ => todo!("expression: {:?}", expr),
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

        if let Some(else_clause) = else_clause {
            // Jump over to the end to prevent fallthrough into else
            // clause, unless the last block already jumps
            if !func.blocks.last().map_or(false, |b| b.jumps()) {
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

        if !func.blocks.last().map_or(false, |b| b.jumps()) {
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

        self.datadefs.push(qbe::DataDef {
            linkage: qbe::Linkage::public(),
            name: name.clone(),
            align: None,
            items,
        });

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
        let (_, lhs_val) = self.generate_expression(func, lhs)?;
        let (_, rhs_val) = self.generate_expression(func, rhs)?;
        let tmp = self.new_temporary();

        // TODO: take the biggest
        let ty = qbe::Type::Word;

        func.assign_instr(
            tmp.clone(),
            ty.clone(),
            match op {
                BinOp::Addition => qbe::Instr::Add(lhs_val, rhs_val),
                BinOp::Subtraction => qbe::Instr::Sub(lhs_val, rhs_val),
                BinOp::Multiplication => qbe::Instr::Mul(lhs_val, rhs_val),
                BinOp::Division => qbe::Instr::Div(lhs_val, rhs_val),
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

        Ok((ty, tmp))
    }

    /// Generates an assignment to either a variable, field access or array
    /// access
    fn generate_assignment(
        &mut self,
        func: &mut qbe::Function,
        lhs: &Expression,
        op: AssignOp,
        rhs: Either<qbe::Value, &Expression>,
    ) -> GeneratorResult<()> {
        if op != AssignOp::Set {
            let binop = match op {
                AssignOp::Add => BinOp::Addition,
                AssignOp::Subtract => BinOp::Subtraction,
                AssignOp::Multiply => BinOp::Multiplication,
                AssignOp::Divide => BinOp::Division,
                _ => unreachable!(),
            };
            let rhs = match rhs {
                Either::Left(_) => unreachable!(),
                Either::Right(expr) => expr,
            };
            // Desugar 'a += b' to 'a = a + b'
            let (_, new_value) = self.generate_binop(func, lhs, &binop, rhs)?;
            return self.generate_assignment(func, lhs, AssignOp::Set, Either::Left(new_value));
        }

        let rhs = match rhs {
            Either::Left(qval) => qval,
            Either::Right(expr) => self.generate_expression(func, expr)?.1,
        };
        match &lhs.kind {
            ExpressionKind::Variable(name) => {
                let (vty, tmp) = self.get_var(name)?;
                func.assign_instr(
                    tmp.to_owned(),
                    vty.to_owned(),
                    qbe::Instr::Copy(rhs),
                );
            }
            ExpressionKind::FieldAccess { expr, field } => {
                let (src, ty, offset) = self.resolve_field_access(expr, field)?;

                let field_ptr = self.new_temporary();
                func.assign_instr(
                    field_ptr.clone(),
                    qbe::Type::Long,
                    qbe::Instr::Add(src, qbe::Value::Const(offset)),
                );

                func.add_instr(qbe::Instr::Store(ty, field_ptr, rhs));
            }
            ExpressionKind::ArrayAccess { .. } => todo!(),
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
        let base = self.new_temporary();
        let (ty, meta, size) = self
            .struct_map
            .get(name)
            .ok_or_else(|| format!("Initialization of undeclared struct '{}'", name))?
            .to_owned();

        func.assign_instr(
            base.clone(),
            qbe::Type::Long,
            // XXX: Always align to 8 bytes?
            qbe::Instr::Alloc8(size),
        );

        for (name, expr) in fields {
            let (_, offset) = meta
                .get(name)
                .ok_or_else(|| format!("Unknown field '{}'", name))?;

            let (ty, expr_tmp) = self.generate_expression(func, expr)?;

            let field_tmp = self.new_temporary();
            func.assign_instr(
                field_tmp.clone(),
                qbe::Type::Long,
                qbe::Instr::Add(base.clone(), qbe::Value::Const(*offset)),
            );

            func.add_instr(qbe::Instr::Store(ty, field_tmp, expr_tmp));
        }

        Ok((ty, base))
    }

    /// Retrieves the result of struct field access
    fn generate_field_access(
        &mut self,
        func: &mut qbe::Function,
        obj: &Expression,
        field: &str,
    ) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        let (src, ty, offset) = self.resolve_field_access(obj, field)?;

        let field_ptr = self.new_temporary();
        func.assign_instr(
            field_ptr.clone(),
            qbe::Type::Long,
            qbe::Instr::Add(src, qbe::Value::Const(offset)),
        );

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
        field: &str,
    ) -> GeneratorResult<(qbe::Value, qbe::Type, u64)> {
        let (ty, src) = match &obj.kind {
            ExpressionKind::Variable(var) => self.get_var(var)?.to_owned(),
            ExpressionKind::FieldAccess { .. } => todo!("nested field access"),
            ExpressionKind::Selff => unimplemented!("methods"),
            other => {
                return Err(format!(
                    "Invalid field access type: expected variable, field access or 'self', got {:?}",
                    other,
                ));
            }
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

        let (ty, offset) = meta
            .get(field)
            .ok_or_else(|| format!("No field '{}' on struct {}", field, name))?
            .to_owned();

        Ok((src, ty, offset))
    }

    /// Generates an array literal
    fn generate_array(
        &mut self,
        func: &mut qbe::Function,
        items: &[Expression],
    ) -> GeneratorResult<(qbe::Type, qbe::Value)> {
        let len = items.len();
        let mut first_type: Option<qbe::Type> = None;
        let mut results: Vec<qbe::Value> = Vec::new();

        for item in items.iter() {
            let (ty, result) = self.generate_expression(func, item)?;
            results.push(result);

            if let Some(first_type) = first_type.clone() {
                if ty != first_type {
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
                qbe::Type::Long.size()
                    + if let Some(ref ty) = first_type {
                        ty.size() * (len as u64)
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
                        qbe::Type::Long.size() + (i as u64) * first_type.as_ref().unwrap().size(),
                    ),
                ),
            );

            func.add_instr(qbe::Instr::Store(
                first_type.as_ref().unwrap().clone(),
                value_ptr,
                value.to_owned(),
            ));
        }

        self.tmp_counter += 1;
        let name = format!("array.{}", self.tmp_counter);
        let typedef = qbe::TypeDef {
            name: name.clone(),
            align: None,
            items: if let Some(ty) = first_type {
                vec![(qbe::Type::Long, 1), (ty, len)]
            } else {
                // No elements
                vec![(qbe::Type::Long, 1)]
            },
        };
        self.typedefs.push(typedef);

        Ok((qbe::Type::Aggregate(name), tmp))
    }

    /// Returns a new unique temporary
    fn new_temporary(&mut self) -> qbe::Value {
        self.tmp_counter += 1;
        qbe::Value::Temporary(format!("tmp.{}", self.tmp_counter))
    }

    /// Returns a new temporary bound to a variable
    fn new_var(&mut self, ty: &qbe::Type, name: &str) -> GeneratorResult<qbe::Value> {
        if self.get_var(name).is_ok() {
            return Err(format!("Re-declaration of variable '{}'", name));
        }

        let tmp = self.new_temporary();

        let scope = self
            .scopes
            .last_mut()
            .expect("expected last scope to be present");
        scope.insert(name.to_owned(), (ty.to_owned(), tmp.to_owned()));

        Ok(tmp)
    }

    /// Returns a temporary accociated to a variable
    fn get_var(&self, name: &str) -> GeneratorResult<&(qbe::Type, qbe::Value)> {
        self.scopes
            .iter()
            .rev()
            .filter_map(|s| s.get(name))
            .next()
            .ok_or_else(|| format!("Undefined variable '{}'", name))
    }

    /// Returns a QBE type for the given AST type
    fn get_type(&self, ty: &Type) -> GeneratorResult<qbe::Type> {
        match &ty.kind {
            TypeKind::Any => Err("'any' type is not supported".into()),
            TypeKind::Int => Ok(qbe::Type::Word),
            TypeKind::Bool => Ok(qbe::Type::Byte),
            TypeKind::Str => Ok(qbe::Type::Long),
            TypeKind::Struct(name) => {
                let (ty, ..) = self
                    .struct_map
                    .get(name)
                    .ok_or_else(|| format!("Use of undeclared struct '{}'", name))?
                    .to_owned();
                Ok(ty)
            }
            TypeKind::Array(..) => Ok(qbe::Type::Long),
        }
    }
}
