use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SymbolId(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ScopeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymKind {
    GlobalVar,
    LocalVar,
    Func,
    BuiltinFunc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Str,
    Unit,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncSig {
    pub params: Vec<Type>,
    pub ret: Type,
}

#[derive(Debug, Clone)]
pub struct SymInfo {
    pub name: String,
    pub scope: ScopeId,
    pub kind: SymKind,
    pub sig: Option<FuncSig>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub parent: Option<ScopeId>,
    pub names: HashMap<String, SymbolId>,
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub infos: Vec<SymInfo>,
    pub scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn new() -> (Self, ScopeId) {
        let global = Scope {
            parent: None,
            names: HashMap::new(),
        };
        let mut scopes = Vec::new();
        scopes.push(global);
        (
            Self {
                infos: Vec::new(),
                scopes,
            },
            ScopeId(0),
        )
    }

    pub fn new_scope(&mut self, parent: ScopeId) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        self.scopes.push(Scope {
            parent: Some(parent),
            names: HashMap::new(),
        });
        id
    }

    pub fn define(&mut self, scope: ScopeId, name: &str, kind: SymKind) -> SymbolId {
        if let Some(&sid) = self.scopes[scope.0 as usize].names.get(name) {
            return sid;
        }
        let sid = SymbolId(self.infos.len() as u32);
        self.infos.push(SymInfo {
            name: name.to_string(),
            scope,
            kind,
            sig: None,
        });
        self.scopes[scope.0 as usize]
            .names
            .insert(name.to_string(), sid);
        sid
    }

    pub fn lookup(&self, mut scope: ScopeId, name: &str) -> Option<SymbolId> {
        loop {
            if let Some(&sid) = self.scopes[scope.0 as usize].names.get(name) {
                return Some(sid);
            }
            if let Some(p) = self.scopes[scope.0 as usize].parent {
                scope = p;
            } else {
                return None;
            }
        }
    }
}
