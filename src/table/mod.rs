use std::collections::HashMap;
use crate::parser::node_type::Function;
use crate::parser::node_type::Type;

pub struct Table {
    types: Vec<Type>,
    functions: HashMap<String, Function>,
    modules: Vec<String>,
}

impl Table {
    pub(crate) fn new() -> Self {
        Self {
            types: Vec::new(),
            functions: HashMap::new(),
            modules: Vec::new()
        }
    }
}

