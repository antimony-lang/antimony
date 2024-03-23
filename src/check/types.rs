use super::{Error, Result as CheckResult};
use crate::ast;
use crate::ast::types::Type as AstType;
use crate::ast::types::TypeKind as AstTypeKind;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Id {
    index: usize,
}

impl Id {
    pub fn dealias(&self, table: &Table) -> Id {
        match self.repr(table) {
            Repr::Named { inner, .. } => inner.dealias(table),
            _ => *self,
        }
    }

    pub fn repr<'a>(&self, table: &'a Table) -> &'a Repr {
        &table.types[self.index].repr
    }

    pub fn dealiased_repr<'a>(&self, table: &'a Table) -> &'a Repr {
        self.dealias(table).repr(table)
    }

    pub fn size(&self, table: &Table) -> usize {
        table.types[self.index].size
    }

    pub fn alignment(&self, table: &Table) -> usize {
        table.types[self.index].alignment
    }
}

// TODO: put size/alignment in a struct of their own, like Dimensions

#[derive(Debug, Eq, PartialEq)]
struct Type {
    pub repr: Repr,
    pub size: usize,
    pub alignment: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Repr {
    Any,
    Void,
    Int,
    Str,
    Bool,
    Array {
        member_type: Id,
        length: usize,
    },
    DynamicArray(Id),
    Struct {
        fields: HashMap<String, StructField>,
        methods: HashMap<String, Id>,
    },
    Named {
        name: String,
        inner: Id,
    },
    Function {
        parameters: Vec<Id>,
        return_type: Id,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct StructField {
    pub ty: Id,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Table {
    types: Vec<Type>,
    // Builtin types
    pub any: Id,
    pub void: Id,
    pub int: Id,
    pub string: Id,
    pub boolean: Id,
}

impl Table {
    pub fn new() -> Self {
        let mut table = Self {
            types: vec![],
            any: Id { index: 0 },
            void: Id { index: 0 },
            int: Id { index: 0 },
            string: Id { index: 0 },
            boolean: Id { index: 0 },
        };
        let any = table.insert(Type {
            repr: Repr::Any,
            // Undefined size
            size: usize::MAX,
            alignment: usize::MAX,
        });
        let void = table.insert(Type {
            repr: Repr::Void,
            size: 0,
            alignment: 0,
        });
        let int = table.insert(Type {
            repr: Repr::Int,
            size: 4,
            alignment: 4,
        });
        let string = table.insert(Type {
            repr: Repr::Str,
            // TODO: 32-bit targets
            size: 8,
            alignment: 8,
        });
        let boolean = table.insert(Type {
            repr: Repr::Bool,
            size: 1,
            alignment: 1,
        });
        Self {
            any,
            void,
            int,
            string,
            boolean,
            ..table
        }
    }

    pub fn by_name(&self, name: &str) -> Option<Id> {
        // FIXME: this is slow
        self.types
            .iter()
            .enumerate()
            .find(|(_, existing)|
                matches!(&existing.repr, Repr::Named { name: existing_name, .. } if existing_name == name))
            .map(|(index, _)| Id { index })
    }

    fn insert(&mut self, ty: Type) -> Id {
        // FIXME: this is slow
        self.types
            .iter()
            .enumerate()
            .find(|(_, existing)| existing == &&ty)
            .map(|(index, _)| Id { index })
            .unwrap_or_else(|| {
                let index = self.types.len();
                self.types.push(ty);
                Id { index }
            })
    }

    pub fn insert_named(&mut self, name: String, inner: Id) -> Result<Id, String> {
        if self.by_name(&name).is_some() {
            return Err(format!("Redefinition of type '{name}'"));
        }
        let (size, alignment) = (inner.size(self), inner.alignment(self));
        Ok(self.insert(Type {
            repr: Repr::Named { name, inner },
            size,
            alignment,
        }))
    }

    pub fn insert_array(&mut self, member_type: Id, length: usize) -> Result<Id, String> {
        if length == 0 {
            return Err("Array length must not be 0".into());
        }

        let (member_size, alignment) = (member_type.size(self), member_type.size(self));
        Ok(self.insert(Type {
            repr: Repr::Array {
                member_type,
                length,
            },
            size: length * member_size,
            alignment,
        }))
    }

    pub fn insert_dynamic_array(&mut self, member_type: Id) -> Id {
        self.insert(Type {
            repr: Repr::DynamicArray(member_type),
            size: 8 + 8, // struct { data: *T, len: u64 }
            alignment: 8,
        })
    }

    pub fn insert_ast_callable(&mut self, callable: &ast::Callable) -> CheckResult<Id> {
        let parameters = callable
            .arguments
            .iter()
            .map(|ast::TypedVariable { ty, .. }| self.insert_ast_type(ty))
            .collect::<Result<Vec<_>, _>>()?;
        let return_type = callable
            .ret_type
            .as_ref()
            .map(|ty| self.insert_ast_type(ty))
            .transpose()?
            .unwrap_or(self.void);

        Ok(self.insert(Type {
            repr: Repr::Function {
                parameters,
                return_type,
            },
            // Undefined size
            size: usize::MAX,
            alignment: usize::MAX,
        }))
    }

    pub fn insert_ast_struct(&mut self, def: &ast::StructDef) -> CheckResult<Id> {
        let mut current_offset = 0usize;
        let fields = def
            .fields
            .iter()
            .map(|ast::TypedVariable { name, ty, .. }| {
                let ty = self.insert_ast_type(ty)?;
                let (size, alignment) = (ty.size(self), ty.alignment(self));
                assert!(size > 0); // TODO?

                let offset = current_offset;
                current_offset += (alignment - current_offset % alignment) % alignment;
                current_offset += size;
                Ok((name.clone(), StructField { ty, offset }))
            })
            .collect::<CheckResult<HashMap<_, _>>>()?;

        let alignment = fields
            .values()
            .map(|field| field.ty.alignment(self))
            .max()
            .unwrap(); // TODO: zero-size structs?
        let size = current_offset;

        let methods = def
            .methods
            .iter()
            .map(|function| {
                let ty = self.insert_ast_callable(&function.callable)?;
                Ok((function.callable.name.clone(), ty))
            })
            .collect::<CheckResult<HashMap<_, _>>>()?;

        let struct_type = self.insert(Type {
            repr: Repr::Struct { fields, methods },
            size,
            alignment,
        });
        self.insert_named(def.name.clone(), struct_type)
            .map_err(|msg| Error::new(def.pos, msg))
    }

    pub fn insert_ast_type(&mut self, ty: &AstType) -> CheckResult<Id> {
        match &ty.kind {
            AstTypeKind::Any => Ok(self.any),
            AstTypeKind::Int => Ok(self.int),
            AstTypeKind::Str => Ok(self.string),
            AstTypeKind::Bool => Ok(self.boolean),
            AstTypeKind::Array(member_type, Some(size)) => {
                let member_type = self.insert_ast_type(member_type)?;
                self.insert_array(member_type, *size)
                    .map_err(|msg| Error::new(ty.pos, msg))
            }
            AstTypeKind::Array(member_type, None) => {
                let member_type = self.insert_ast_type(member_type)?;
                Ok(self.insert_dynamic_array(member_type))
            }
            AstTypeKind::Struct(name) => self
                .by_name(name)
                .ok_or_else(|| Error::new(ty.pos, format!("Could not resolve type '{name}'"))),
        }
    }

    pub fn assignable(&self, left: Id, right: Id) -> bool {
        if left == right {
            return true;
        }

        match (left.repr(self), right.repr(self)) {
            (Repr::Any, _) => true,
            (Repr::DynamicArray(dyn_member), Repr::Array { member_type, .. })
                if dyn_member == member_type =>
            {
                true
            }
            _ => false,
        }
    }
}
