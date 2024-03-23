use crate::check::types;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
enum ScopeKind {
    Root,
    Block,
    Loop,
    Function { return_type: types::Id },
}

#[derive(Debug)]
struct Scope {
    parent: Option<Id>,
    kind: ScopeKind,
    name_map: HashMap<String, InnerId>,
    types: Vec<types::Id>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
struct InnerId {
    index: usize,
}

impl Scope {
    fn lookup(&self, name: &str) -> Option<(InnerId, types::Id)> {
        self.name_map
            .get(name)
            .map(|inner| (*inner, self.types[inner.index]))
    }

    fn insert(&mut self, name: String, ty: types::Id) -> InnerId {
        let inner = InnerId {
            index: self.types.len(),
        };
        self.types.push(ty);
        self.name_map.insert(name, inner);
        inner
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Id {
    index: usize,
}

impl Id {
    pub fn lookup(&self, name: &str, table: &Table) -> Option<(VariableId, types::Id)> {
        let scope = self.get(table);
        match scope.lookup(name) {
            Some((inner, ty)) => Some((
                VariableId {
                    scope: *self,
                    inner,
                },
                ty,
            )),
            None => match scope.parent {
                Some(parent) => parent.lookup(name, table),
                None => None,
            },
        }
    }

    pub fn insert(&self, name: String, ty: types::Id, table: &mut Table) -> VariableId {
        let inner = table.scopes[self.index].insert(name, ty);
        VariableId {
            scope: *self,
            inner,
        }
    }

    pub fn nearest_loop(&self, table: &Table) -> Option<Id> {
        let scope = self.get(table);
        if scope.kind == ScopeKind::Loop {
            Some(*self)
        } else {
            match scope.parent {
                Some(parent) => parent.nearest_loop(table),
                None => None,
            }
        }
    }

    pub fn return_type(&self, table: &Table) -> Option<types::Id> {
        let scope = self.get(table);
        match scope.kind {
            ScopeKind::Function { return_type } => Some(return_type),
            _ => match scope.parent {
                Some(parent) => parent.return_type(table),
                None => None,
            },
        }
    }

    pub fn push(&self, table: &mut Table) -> Id {
        table.add_node(Scope {
            parent: Some(*self),
            kind: ScopeKind::Block,
            name_map: HashMap::new(),
            types: Vec::new(),
        })
    }

    pub fn push_function(&self, return_type: types::Id, table: &mut Table) -> Id {
        table.add_node(Scope {
            parent: Some(*self),
            kind: ScopeKind::Function { return_type },
            name_map: HashMap::new(),
            types: Vec::new(),
        })
    }

    pub fn push_loop(&self, table: &mut Table) -> Id {
        table.add_node(Scope {
            parent: Some(*self),
            kind: ScopeKind::Loop,
            name_map: HashMap::new(),
            types: Vec::new(),
        })
    }

    fn get<'a>(&self, table: &'a Table) -> &'a Scope {
        &table.scopes[self.index]
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct VariableId {
    scope: Id,
    inner: InnerId,
}

impl VariableId {
    #[allow(dead_code)]
    pub fn get_type(&self, table: &Table) -> types::Id {
        let scope = self.scope.get(table);
        scope.types[self.inner.index]
    }
}

#[derive(Debug)]
pub struct Table {
    scopes: Vec<Scope>,
}

impl Table {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub fn add_root(&mut self) -> Id {
        self.add_node(Scope {
            parent: None,
            kind: ScopeKind::Root,
            name_map: HashMap::new(),
            types: Vec::new(),
        })
    }

    fn add_node(&mut self, scope: Scope) -> Id {
        let id = self.scopes.len();
        self.scopes.push(scope);
        Id { index: id }
    }
}
