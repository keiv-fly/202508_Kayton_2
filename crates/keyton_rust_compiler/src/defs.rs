use std::collections::HashMap;

use crate::parser::{NodeId, Stmt};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DefId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub enum DefKind {
    Var,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub def_id: DefId,
    pub name: String,
    pub node_id: NodeId,
    pub kind: DefKind,
}

#[derive(Debug, Default)]
pub struct DefTable {
    next_def_id: u32,
    // Map from name to all definitions (multiple assignments allowed)
    pub by_name: HashMap<String, Vec<Definition>>,
    // Map from AST node id to def
    pub by_node: HashMap<NodeId, Definition>,
    // Flat list of all definitions in order of appearance
    pub all: Vec<Definition>,
}

impl DefTable {
    pub fn new() -> Self {
        Self {
            next_def_id: 1,
            by_name: HashMap::new(),
            by_node: HashMap::new(),
            all: Vec::new(),
        }
    }

    fn alloc_id(&mut self) -> DefId {
        let id = self.next_def_id;
        self.next_def_id += 1;
        DefId(id)
    }
}

pub fn collect_definitions(program: &[Stmt]) -> DefTable {
    let mut table = DefTable::new();
    for stmt in program {
        match stmt {
            Stmt::Assign {
                node_id,
                name,
                expr: _,
            } => {
                let def = Definition {
                    def_id: table.alloc_id(),
                    name: name.clone(),
                    node_id: *node_id,
                    kind: DefKind::Var,
                };
                table.by_node.insert(*node_id, def.clone());
                table
                    .by_name
                    .entry(name.clone())
                    .or_default()
                    .push(def.clone());
                table.all.push(def);
            }
            Stmt::ExprStmt { .. } => {}
        }
    }
    table
}
