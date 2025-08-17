use std::collections::HashMap;

use crate::shir::resolver::ResolvedProgram;
use crate::shir::sym::{SymbolId, Type};
use crate::thir::types::{TExpr, TStmt, TStringPart, TypedProgram};

use super::types::{RExpr, RStmt, RStringPart, RustProgram};

/// Function mapping rules from source language to Rust
#[derive(Debug, Clone)]
pub struct FunctionMapping {
    pub source_name: String,
    pub rust_macro: String,
    pub arg_count: Option<usize>, // None means variable args
}

pub struct Converter<'a> {
    function_mappings: HashMap<String, FunctionMapping>,
    var_types: HashMap<SymbolId, Type>,
    resolved: &'a ResolvedProgram,
}

impl<'a> Converter<'a> {
    pub fn new(resolved: &'a ResolvedProgram) -> Self {
        let mut function_mappings = HashMap::new();

        // Define function mapping rules
        function_mappings.insert(
            "print".to_string(),
            FunctionMapping {
                source_name: "print".to_string(),
                rust_macro: "println!".to_string(),
                arg_count: None, // println! can take variable number of arguments
            },
        );

        Self {
            function_mappings,
            var_types: HashMap::new(),
            resolved,
        }
    }

    pub fn convert_program(&mut self, typed: &TypedProgram) -> RustProgram {
        self.var_types = typed.var_types.clone();

        let rhir = typed.thir.iter().map(|s| self.convert_stmt(s)).collect();

        RustProgram {
            rhir,
            var_types: self.var_types.clone(),
        }
    }

    fn convert_stmt(&mut self, s: &TStmt) -> RStmt {
        match s {
            TStmt::Assign { hir_id, sym, expr } => RStmt::Assign {
                hir_id: *hir_id,
                sym: *sym,
                expr: self.convert_expr(expr),
            },
            TStmt::ExprStmt { hir_id, expr } => RStmt::ExprStmt {
                hir_id: *hir_id,
                expr: self.convert_expr(expr),
            },
        }
    }

    fn convert_expr(&mut self, e: &TExpr) -> RExpr {
        match e {
            TExpr::Int { hir_id, value, ty } => RExpr::Int {
                hir_id: *hir_id,
                value: *value,
                ty: ty.clone(),
            },
            TExpr::Str { hir_id, value, ty } => RExpr::Str {
                hir_id: *hir_id,
                value: value.clone(),
                ty: ty.clone(),
            },
            TExpr::Name { hir_id, sym, ty } => RExpr::Name {
                hir_id: *hir_id,
                sym: *sym,
                ty: ty.clone(),
            },
            TExpr::Binary {
                hir_id,
                left,
                op,
                right,
                ty,
            } => RExpr::Binary {
                hir_id: *hir_id,
                left: Box::new(self.convert_expr(left)),
                op: op.clone(),
                right: Box::new(self.convert_expr(right)),
                ty: ty.clone(),
            },
            TExpr::Call {
                hir_id,
                func,
                args,
                ty,
            } => {
                // Check if this is a function call that should be converted to a macro
                if let TExpr::Name { sym, .. } = func.as_ref() {
                    // Get the function name from the symbol table
                    if let Some(symbol_info) = self.resolved.symbols.infos.get(sym.0 as usize) {
                        let func_name = &symbol_info.name;

                        // Check if we have a mapping for this function
                        if let Some(mapping) = self.function_mappings.get(func_name) {
                            return RExpr::MacroCall {
                                hir_id: *hir_id,
                                macro_name: mapping.rust_macro.clone(),
                                args: args.iter().map(|a| self.convert_expr(a)).collect(),
                                ty: ty.clone(),
                            };
                        }
                    }
                }

                // If no mapping found, keep as a regular call (though this shouldn't happen in our current setup)
                RExpr::MacroCall {
                    hir_id: *hir_id,
                    macro_name: "unknown_macro!".to_string(),
                    args: args.iter().map(|a| self.convert_expr(a)).collect(),
                    ty: ty.clone(),
                }
            }
            TExpr::InterpolatedString { hir_id, parts, ty } => RExpr::InterpolatedString {
                hir_id: *hir_id,
                parts: parts.iter().map(|p| self.convert_string_part(p)).collect(),
                ty: ty.clone(),
            },
        }
    }

    fn convert_string_part(&mut self, p: &TStringPart) -> RStringPart {
        match p {
            TStringPart::Text { hir_id, value } => RStringPart::Text {
                hir_id: *hir_id,
                value: value.clone(),
            },
            TStringPart::Expr { hir_id, expr } => RStringPart::Expr {
                hir_id: *hir_id,
                expr: self.convert_expr(expr),
            },
        }
    }
}

pub fn convert_to_rhir(typed: &TypedProgram, resolved: &ResolvedProgram) -> RustProgram {
    let mut converter = Converter::new(resolved);
    converter.convert_program(typed)
}
