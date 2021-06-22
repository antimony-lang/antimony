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
use crate::util::Either;
use std::collections::HashMap;

pub struct QbeGenerator {
    /// Counter for unique temporary names
    tmp_counter: u32,
    /// Block-scoped variable -> temporary mappings
    scopes: Vec<HashMap<String, (QbeType, QbeTemporary)>>,
}

impl Generator for QbeGenerator {
    fn generate(prog: Module) -> GeneratorResult<String> {
        let mut generator = QbeGenerator {
            tmp_counter: 0,
            scopes: Vec::new(),
        };
        let mut buf = String::new();

        for func in &prog.func {
            let func = generator.generate_function(func)?;
            buf.push_str(&format!("{}\n", func));
        }

        Ok(buf)
    }
}

impl QbeGenerator {
    fn generate_function(&mut self, func: &Function) -> GeneratorResult<QbeFunction> {
        // Function argument scope
        self.scopes.push(HashMap::new());

        let mut arguments: Vec<(QbeType, QbeTemporary)> = Vec::new();
        for arg in &func.arguments {
            let ty = self
                .get_type(
                    arg.ty
                        .as_ref()
                        .ok_or("Function arguments must have a type")?
                        .to_owned(),
                )?
                .into_abi();
            let tmp = self.new_var(&ty, &arg.name)?;

            arguments.push((ty, tmp));
        }

        let return_ty = if let Some(ty) = &func.ret_type {
            Some(self.get_type(ty.to_owned())?.into_abi())
        } else {
            None
        };

        let mut qfunc = QbeFunction {
            exported: true,
            name: func.name.clone(),
            arguments,
            return_ty,
            blocks: Vec::new(),
        };

        qfunc.add_block("start".to_owned());

        self.generate_statement(&mut qfunc, &func.body)?;

        // Automatically add return in void functions
        // TODO: validate the same in non-void ones
        if func.ret_type.is_none() {
            qfunc.add_instr(QbeInstr::Ret(None));
        }

        self.scopes.pop();

        Ok(qfunc)
    }

    /// Generates a statement
    fn generate_statement(
        &mut self,
        func: &mut QbeFunction,
        stmt: &Statement,
    ) -> GeneratorResult<()> {
        match stmt {
            Statement::Block(statements, _) => {
                self.scopes.push(HashMap::new());
                for stmt in statements.iter() {
                    self.generate_statement(func, stmt)?;
                }
                self.scopes.pop();
            }
            Statement::Declare(var, expr) => {
                let ty = self.get_type(
                    var.ty
                        .as_ref()
                        .ok_or_else(|| format!("Missing type for variable '{}'", &var.name))?
                        .to_owned(),
                )?;
                let tmp = self.new_var(&ty, &var.name)?;

                if let Some(expr) = expr {
                    let (ty, result) = self.generate_expression(func, expr)?;
                    func.assign_instr(tmp, ty, QbeInstr::Copy(Either::Left(result)));
                }
            }
            Statement::Return(val) => match val {
                Some(expr) => {
                    let (_, result) = self.generate_expression(func, expr)?;
                    // TODO: Cast to function return type
                    func.add_instr(QbeInstr::Ret(Some(result)));
                }
                None => func.add_instr(QbeInstr::Ret(None)),
            },
            Statement::If(cond, if_clause, else_clause) => {
                self.generate_if(func, cond, if_clause, else_clause)?;
            }
            Statement::While(cond, body) => {
                self.generate_while(func, cond, body)?;
            }
            _ => todo!("statement: {:?}", stmt),
        }
        Ok(())
    }

    /// Generates an expression
    fn generate_expression(
        &mut self,
        func: &mut QbeFunction,
        expr: &Expression,
    ) -> GeneratorResult<(QbeType, QbeTemporary)> {
        match expr {
            Expression::Int(literal) => {
                let tmp = self.new_temporary();
                func.assign_instr(
                    tmp.clone(),
                    QbeType::Word,
                    QbeInstr::Copy(Either::Right(*literal)),
                );

                Ok((QbeType::Word, tmp))
            }
            Expression::Bool(literal) => {
                let tmp = self.new_temporary();
                func.assign_instr(
                    tmp.clone(),
                    QbeType::Word,
                    QbeInstr::Copy(Either::Right(if *literal { 1 } else { 0 })),
                );

                Ok((QbeType::Word, tmp))
            }
            Expression::Variable(name) => self.get_var(name).map(|v| v.to_owned()),
            Expression::BinOp(lhs, op, rhs) => {
                let (_, lhs) = self.generate_expression(func, &lhs)?;
                let (_, rhs) = self.generate_expression(func, &rhs)?;
                let tmp = self.new_temporary();

                // TODO: take the biggest
                let ty = QbeType::Word;

                func.assign_instr(
                    tmp.clone(),
                    ty.clone(),
                    match op {
                        BinOp::Addition => QbeInstr::Add(lhs, rhs),
                        BinOp::Subtraction => QbeInstr::Sub(lhs, rhs),
                        BinOp::Multiplication => QbeInstr::Mul(lhs, rhs),
                        BinOp::Division => QbeInstr::Div(lhs, rhs),
                        BinOp::Modulus => QbeInstr::Rem(lhs, rhs),

                        BinOp::AddAssign => todo!(),
                        BinOp::SubtractAssign => todo!(),
                        BinOp::MultiplyAssign => todo!(),
                        BinOp::DivideAssign => todo!(),

                        BinOp::And => QbeInstr::And(lhs, rhs),
                        BinOp::Or => QbeInstr::Or(lhs, rhs),

                        // Others should be comparisons
                        cmp => QbeInstr::Cmp(
                            ty.clone(),
                            match cmp {
                                BinOp::LessThan => QbeCmp::Slt,
                                BinOp::LessThanOrEqual => QbeCmp::Sle,
                                BinOp::GreaterThan => QbeCmp::Sgt,
                                BinOp::GreaterThanOrEqual => QbeCmp::Sge,
                                BinOp::Equal => QbeCmp::Eq,
                                BinOp::NotEqual => QbeCmp::Ne,
                                _ => unreachable!(),
                            },
                            lhs,
                            rhs,
                        ),
                    },
                );
                Ok((ty, tmp))
            }
            _ => todo!("expression: {:?}", expr),
        }
    }

    /// Generates an `if` statement
    fn generate_if(
        &mut self,
        func: &mut QbeFunction,
        cond: &Expression,
        if_clause: &Statement,
        else_clause: &Option<Box<Statement>>,
    ) -> GeneratorResult<()> {
        let (_, result) = self.generate_expression(func, cond)?;

        self.tmp_counter += 1;
        let if_label = format!("cond.{}.if", self.tmp_counter);
        let else_label = format!("cond.{}.else", self.tmp_counter);
        let end_label = format!("cond.{}.end", self.tmp_counter);

        func.add_instr(QbeInstr::Jnz(
            result,
            if_label.clone(),
            if else_clause.is_some() {
                else_label.clone()
            } else {
                end_label.clone()
            },
        ));

        func.add_block(if_label);
        self.generate_statement(func, &if_clause)?;

        if let Some(else_clause) = else_clause {
            // Jump over to the end to prevent fallthrough into else
            // clause, unless the last block already jumps
            if !func.blocks.last().map_or(false, |b| b.jumps()) {
                func.add_instr(QbeInstr::Jmp(end_label.clone()));
            }

            func.add_block(else_label);
            self.generate_statement(func, &else_clause)?;
        }

        if !func.blocks.last().map_or(false, |b| b.jumps()) {
            func.add_block(end_label);
        }

        Ok(())
    }

    /// Generates a `while` statement
    fn generate_while(
        &mut self,
        func: &mut QbeFunction,
        cond: &Expression,
        body: &Statement,
    ) -> GeneratorResult<()> {
        self.tmp_counter += 1;
        let cond_label = format!("loop.{}.cond", self.tmp_counter);
        let body_label = format!("loop.{}.body", self.tmp_counter);
        let end_label = format!("loop.{}.end", self.tmp_counter);

        func.add_block(cond_label.clone());

        let (_, result) = self.generate_expression(func, cond)?;
        func.add_instr(QbeInstr::Jnz(result, body_label.clone(), end_label.clone()));

        func.add_block(body_label);
        self.generate_statement(func, body)?;

        if !func.blocks.last().map_or(false, |b| b.jumps()) {
            func.add_instr(QbeInstr::Jmp(cond_label));
        }

        func.add_block(end_label);

        Ok(())
    }

    /// Returns a new unique temporary
    fn new_temporary(&mut self) -> QbeTemporary {
        self.tmp_counter += 1;
        QbeTemporary::new(format!("tmp.{}", self.tmp_counter))
    }

    /// Returns a new temporary bound to a variable
    fn new_var(&mut self, ty: &QbeType, name: &str) -> GeneratorResult<QbeTemporary> {
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
    fn get_var(&self, name: &str) -> GeneratorResult<&(QbeType, QbeTemporary)> {
        self.scopes
            .iter()
            .rev()
            .filter_map(|s| s.get(name))
            .next()
            .ok_or_else(|| format!("Undefined variable '{}'", name))
    }

    /// Returns a QBE type for the given AST type
    fn get_type(&self, ty: Type) -> GeneratorResult<QbeType> {
        match ty {
            Type::Any => Err("'any' type is not supported".into()),
            Type::Int => Ok(QbeType::Word),
            Type::Bool => Ok(QbeType::Byte),
            Type::Str | Type::Array(..) | Type::Struct(_) => todo!("aggregate types"),
        }
    }
}

use std::fmt;

/// QBE comparision
#[derive(Debug)]
enum QbeCmp {
    /// Returns 1 if first value is less than second, respecting signedness
    Slt,
    /// Returns 1 if first value is less than or equal to second, respecting signedness
    Sle,
    /// Returns 1 if first value is greater than second, respecting signedness
    Sgt,
    /// Returns 1 if first value is greater than or equal to second, respecting signedness
    Sge,
    /// Returns 1 if values are equal
    Eq,
    /// Returns 1 if values are not equal
    Ne,
}

/// QBE instruction
#[derive(Debug)]
enum QbeInstr {
    /// Adds values of two temporaries together
    Add(QbeTemporary, QbeTemporary),
    /// Subtracts the second value from the first one
    Sub(QbeTemporary, QbeTemporary),
    /// Multiplies values of two temporaries
    Mul(QbeTemporary, QbeTemporary),
    /// Divides the first value by the second one
    Div(QbeTemporary, QbeTemporary),
    /// Returns a remainder from division
    Rem(QbeTemporary, QbeTemporary),
    /// Performs a comparion between values
    Cmp(QbeType, QbeCmp, QbeTemporary, QbeTemporary),
    /// Performs a bitwise AND on values
    And(QbeTemporary, QbeTemporary),
    /// Performs a bitwise OR on values
    Or(QbeTemporary, QbeTemporary),
    /// Copies either a temporary or a literal value
    Copy(Either<QbeTemporary, usize>),
    /// Return from a function, optionally with a value
    Ret(Option<QbeTemporary>),
    /// Jumps to first label if a value is nonzero or to the second one otherwise
    Jnz(QbeTemporary, String, String),
    /// Unconditionally jumps to a label
    Jmp(String),
}

impl fmt::Display for QbeInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add(lhs, rhs) => write!(f, "add {}, {}", lhs, rhs),
            Self::Sub(lhs, rhs) => write!(f, "sub {}, {}", lhs, rhs),
            Self::Mul(lhs, rhs) => write!(f, "mul {}, {}", lhs, rhs),
            Self::Div(lhs, rhs) => write!(f, "div {}, {}", lhs, rhs),
            Self::Rem(lhs, rhs) => write!(f, "rem {}, {}", lhs, rhs),
            Self::Cmp(ty, cmp, lhs, rhs) => {
                assert!(
                    !matches!(ty, QbeType::Aggregate(_)),
                    "Cannot compare aggregate types"
                );

                write!(
                    f,
                    "c{}{} {}, {}",
                    match cmp {
                        QbeCmp::Slt => "slt",
                        QbeCmp::Sle => "sle",
                        QbeCmp::Sgt => "sgt",
                        QbeCmp::Sge => "sge",
                        QbeCmp::Eq => "eq",
                        QbeCmp::Ne => "ne",
                    },
                    ty,
                    lhs,
                    rhs,
                )
            }
            Self::And(lhs, rhs) => write!(f, "and {}, {}", lhs, rhs),
            Self::Or(lhs, rhs) => write!(f, "or {}, {}", lhs, rhs),
            Self::Copy(val) => match val {
                Either::Left(temp) => write!(f, "copy {}", temp),
                Either::Right(lit) => write!(f, "copy {}", *lit),
            },
            Self::Ret(val) => match val {
                Some(val) => write!(f, "ret {}", val),
                None => write!(f, "ret"),
            },
            Self::Jnz(val, if_nonzero, if_zero) => {
                write!(f, "jnz {}, @{}, @{}", val, if_nonzero, if_zero)
            }
            Self::Jmp(label) => write!(f, "jmp @{}", label),
        }
    }
}

/// QBE type
#[derive(Debug, Eq, PartialEq, Clone)]
#[allow(dead_code)]
enum QbeType {
    // Base types
    Word,
    Long,
    Single,
    Double,

    // Extended types
    Byte,
    Halfword,

    /// Aggregate type with a specified name
    Aggregate(String),
}

impl QbeType {
    /// Returns a C ABI type. Extended types are converted to closest base
    /// types
    fn into_abi(self) -> Self {
        match self {
            Self::Byte | Self::Halfword => Self::Word,
            other => other,
        }
    }
}

impl fmt::Display for QbeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Word => write!(f, "w"),
            Self::Long => write!(f, "l"),
            Self::Single => write!(f, "s"),
            Self::Double => write!(f, "d"),

            Self::Byte => write!(f, "b"),
            Self::Halfword => write!(f, "h"),

            Self::Aggregate(name) => write!(f, ":{}", name),
        }
    }
}

/// QBE temporary
#[derive(Debug, Clone)]
struct QbeTemporary {
    name: String,
}

impl QbeTemporary {
    /// Returns a new temporary
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl fmt::Display for QbeTemporary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.name)
    }
}

/// An IR statement
#[derive(Debug)]
enum QbeStatement {
    Assign(QbeTemporary, QbeType, QbeInstr),
    Volatile(QbeInstr),
}

impl fmt::Display for QbeStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assign(temp, ty, instr) => {
                write!(f, "{} ={} {}", temp, ty, instr)
            }
            Self::Volatile(instr) => write!(f, "{}", instr),
        }
    }
}

/// Function block with a label
#[derive(Debug)]
struct QbeBlock {
    /// Label before the block
    label: String,

    /// A list of instructions in the block
    instructions: Vec<QbeStatement>,
}

impl QbeBlock {
    /// Adds a new instruction to the block
    fn add_instr(&mut self, instr: QbeInstr) {
        self.instructions.push(QbeStatement::Volatile(instr));
    }

    /// Adds a new instruction assigned to a temporary
    fn assign_instr(&mut self, temp: QbeTemporary, ty: QbeType, instr: QbeInstr) {
        self.instructions.push(QbeStatement::Assign(temp, ty, instr));
    }

    /// Returns true if the block's last instruction is a jump
    fn jumps(&self) -> bool {
        let last = self.instructions.last();

        if let Some(QbeStatement::Volatile(instr)) = last {
            matches!(
                instr,
                QbeInstr::Ret(_) | QbeInstr::Jmp(_) | QbeInstr::Jnz(..)
            )
        } else {
            false
        }
    }
}

impl fmt::Display for QbeBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "@{}", self.label)?;

        write!(
            f,
            "{}",
            self.instructions
                .iter()
                .map(|instr| format!("\t{}", instr))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

/// QBE function
#[derive(Debug)]
struct QbeFunction {
    /// Should the function be available to outside users
    exported: bool,

    /// Function name
    name: String,

    /// Function arguments
    arguments: Vec<(QbeType, QbeTemporary)>,

    /// Return type
    return_ty: Option<QbeType>,

    /// Labelled blocks
    blocks: Vec<QbeBlock>,
}

impl QbeFunction {
    /// Adds a new empty block with a specified label
    fn add_block(&mut self, label: String) {
        self.blocks.push(QbeBlock {
            label,
            instructions: Vec::new(),
        });
    }

    /// Adds a new instruction to the last block
    fn add_instr(&mut self, instr: QbeInstr) {
        self.blocks
            .last_mut()
            .expect("Last block must be present")
            .add_instr(instr);
    }

    /// Adds a new instruction assigned to a temporary
    fn assign_instr(&mut self, temp: QbeTemporary, ty: QbeType, instr: QbeInstr) {
        self.blocks
            .last_mut()
            .expect("Last block must be present")
            .assign_instr(temp, ty, instr);
    }
}

impl fmt::Display for QbeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.exported {
            write!(f, "export ")?;
        }
        write!(f, "function")?;
        if let Some(ty) = &self.return_ty {
            write!(f, " {}", ty)?;
        }

        writeln!(
            f,
            " ${name}({args}) {{",
            name = self.name,
            args = self
                .arguments
                .iter()
                .map(|(ty, temp)| format!("{} {}", ty, temp))
                .collect::<Vec<String>>()
                .join(", "),
        )?;

        for blk in self.blocks.iter() {
            writeln!(f, "{}", blk)?;
        }

        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary() {
        let tmp = QbeTemporary::new("temp42".into());
        assert_eq!(format!("{}", tmp), "%temp42");
    }

    #[test]
    fn block() {
        let blk = QbeBlock {
            label: "start".into(),
            instructions: vec![QbeStatement::Volatile(QbeInstr::Ret(None))],
        };

        let formatted = format!("{}", blk);
        let mut lines = formatted.lines();
        assert_eq!(lines.next().unwrap(), "@start");
        assert_eq!(lines.next().unwrap(), "\tret");

        let blk = QbeBlock {
            label: "start".into(),
            instructions: vec![
                QbeStatement::Volatile(QbeInstr::Ret(None)),
                QbeStatement::Volatile(QbeInstr::Ret(None)),
            ],
        };

        let formatted = format!("{}", blk);
        let mut lines = formatted.lines();
        assert_eq!(lines.next().unwrap(), "@start");
        assert_eq!(lines.next().unwrap(), "\tret");
        assert_eq!(lines.next().unwrap(), "\tret");
    }

    #[test]
    fn function() {
        let func = QbeFunction {
            exported: true,
            return_ty: None,
            name: "main".into(),
            arguments: Vec::new(),
            blocks: vec![QbeBlock {
                label: "start".into(),
                instructions: vec![QbeStatement::Volatile(QbeInstr::Ret(None))],
            }],
        };

        let formatted = format!("{}", func);
        let mut lines = formatted.lines();
        assert_eq!(lines.next().unwrap(), "export function $main() {");
        assert_eq!(lines.next().unwrap(), "@start");
        assert_eq!(lines.next().unwrap(), "\tret");
        assert_eq!(lines.next().unwrap(), "}");
    }

    #[test]
    fn type_into_abi() {
        // Base types and aggregates should stay unchanged
        let unchanged = |ty: QbeType| assert_eq!(ty.clone().into_abi(), ty);
        unchanged(QbeType::Word);
        unchanged(QbeType::Long);
        unchanged(QbeType::Single);
        unchanged(QbeType::Double);
        unchanged(QbeType::Aggregate("foo".into()));

        // Extended types are transformed into closest base types
        assert_eq!(QbeType::Byte.into_abi(), QbeType::Word);
        assert_eq!(QbeType::Halfword.into_abi(), QbeType::Word);
    }
}
