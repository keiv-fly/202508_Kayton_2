use std::collections::HashMap;

use crate::hir::hir_types::HirId;
use crate::span::Span;

use super::errors::ResolveReport;
use super::super::sym::{FuncSig, ScopeId, SymKind, SymbolId, SymbolTable, Type};
use super::user_funcs::UserFuncDef;

pub struct Resolver {
    pub syms: SymbolTable,
    pub report: ResolveReport,
    scope_stack: Vec<ScopeId>,
    pub(super) builtins: HashMap<String, SymbolId>,
    pub(super) spans: HashMap<HirId, Span>,
    pub(super) user_funcs: HashMap<SymbolId, UserFuncDef>,
    pub(super) plugin_manifests: HashMap<String, kayton_plugin_sdk::manifest::Manifest>,
}

impl Resolver {
    pub fn new(spans: HashMap<HirId, Span>) -> Self {
        let (syms, global) = SymbolTable::new();
        Self {
            syms,
            report: ResolveReport::default(),
            scope_stack: vec![global],
            builtins: HashMap::new(),
            spans,
            user_funcs: HashMap::new(),
            plugin_manifests: HashMap::new(),
        }
    }

    pub fn add_builtin(&mut self, name: &str) -> SymbolId {
        let g = self.global_scope();
        let sid = self.syms.define(g, name, SymKind::BuiltinFunc);
        if let Some(info) = self.syms.infos.get_mut(sid.0 as usize) {
            info.sig = Some(FuncSig {
                params: vec![Type::Any],
                ret: Type::Unit,
            });
        }
        self.builtins.insert(name.to_string(), sid);
        sid
    }

    pub(super) fn current_scope(&self) -> ScopeId {
        *self.scope_stack.last().unwrap()
    }
    pub(super) fn global_scope(&self) -> ScopeId {
        ScopeId(0)
    }

    #[allow(dead_code)]
    pub(super) fn enter_scope(&mut self) -> ScopeId {
        let parent = self.current_scope();
        let s = self.syms.new_scope(parent);
        self.scope_stack.push(s);
        s
    }

    #[allow(dead_code)]
    pub(super) fn leave_scope(&mut self) {
        self.scope_stack.pop();
    }
}
