use std::collections::HashMap;

use crate::rhir::types::{RExpr, RStmt, RStringPart, RustProgram};
use crate::shir::resolver::ResolvedProgram;
use crate::shir::sym::SymbolId;

use super::types::RustCode;

pub struct CodeGenerator<'a> {
    var_names: HashMap<SymbolId, String>,
    assigned_vars: std::collections::HashSet<SymbolId>,
    resolved: &'a ResolvedProgram,
    next_var_id: u32,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(resolved: &'a ResolvedProgram) -> Self {
        Self {
            var_names: HashMap::new(),
            assigned_vars: std::collections::HashSet::new(),
            resolved,
            next_var_id: 0,
        }
    }

    pub fn generate_code(&mut self, rhir_program: &RustProgram) -> RustCode {
        let mut source_code = String::new();

        // Add the main function
        source_code.push_str("fn main() {\n");

        // Generate statements
        for stmt in &rhir_program.rhir {
            source_code.push_str("    ");
            source_code.push_str(&self.convert_stmt_to_string(stmt));
            source_code.push_str("\n");
        }

        source_code.push_str("}\n");

        RustCode {
            source_code,
            var_names: self.var_names.clone(),
        }
    }

    fn convert_stmt_to_string(&mut self, stmt: &RStmt) -> String {
        match stmt {
            RStmt::Assign { sym, expr, .. } => {
                let var_name = self.get_or_create_var_name(*sym);
                let expr_str = self.convert_expr_to_string(expr);

                // Check if this variable has been assigned before
                let is_mutable = self.assigned_vars.contains(sym);
                self.assigned_vars.insert(*sym);

                if is_mutable {
                    format!("{} = {};", var_name, expr_str)
                } else {
                    format!("let mut {} = {};", var_name, expr_str)
                }
            }
            RStmt::ExprStmt { expr, .. } => {
                let expr_str = self.convert_expr_to_string(expr);
                format!("{};", expr_str)
            }
        }
    }

    fn convert_expr_to_string(&mut self, expr: &RExpr) -> String {
        match expr {
            RExpr::Int { value, .. } => value.to_string(),
            RExpr::Str { value, .. } => format!("\"{}\"", value),
            RExpr::Name { sym, .. } => self.get_or_create_var_name(*sym),
            RExpr::Binary {
                left, op, right, ..
            } => {
                let left_str = self.convert_expr_to_string(left);
                let right_str = self.convert_expr_to_string(right);
                let op_str = match op {
                    crate::hir::hir_types::HirBinOp::Add => "+",
                };
                format!("({} {} {})", left_str, op_str, right_str)
            }
            RExpr::MacroCall {
                macro_name, args, ..
            } => {
                let args_str = args
                    .iter()
                    .map(|a| self.convert_expr_to_string(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", macro_name, args_str)
            }
            RExpr::InterpolatedString { parts, .. } => {
                self.convert_interpolated_string_to_format(parts)
            }
        }
    }

    fn convert_interpolated_string_to_format(&mut self, parts: &[RStringPart]) -> String {
        let mut format_string = String::new();
        let mut args = Vec::new();

        for part in parts {
            match part {
                RStringPart::Text { value, .. } => {
                    // Escape quotes and braces in the text
                    let escaped = value
                        .replace("\"", "\\\"")
                        .replace("{", "{{")
                        .replace("}", "}}");
                    format_string.push_str(&escaped);
                }
                RStringPart::Expr { expr, .. } => {
                    format_string.push_str("{}");
                    args.push(self.convert_expr_to_string(expr));
                }
            }
        }

        if args.is_empty() {
            format!("\"{}\"", format_string)
        } else {
            let args_str = args.join(", ");
            format!("format!(\"{}\", {})", format_string, args_str)
        }
    }

    fn get_or_create_var_name(&mut self, sym: SymbolId) -> String {
        if let Some(name) = self.var_names.get(&sym) {
            name.clone()
        } else {
            // Get the original name from the symbol table
            if let Some(symbol_info) = self.resolved.symbols.infos.get(sym.0 as usize) {
                let base_name = symbol_info.name.clone();

                // Count how many variables with this base name we've already seen
                let existing_count = self
                    .var_names
                    .values()
                    .filter(|v| **v == base_name || v.starts_with(&format!("{}_", base_name)))
                    .count();

                let var_name = if existing_count > 0 {
                    // If name conflicts, add a suffix based on the count
                    format!("{}_{}", base_name, existing_count - 1)
                } else {
                    base_name
                };

                self.var_names.insert(sym, var_name.clone());
                var_name
            } else {
                // Fallback for unknown symbols
                let var_name = format!("var_{}", self.next_var_id);
                self.var_names.insert(sym, var_name.clone());
                self.next_var_id += 1;
                var_name
            }
        }
    }
}

pub fn generate_rust_code(rhir_program: &RustProgram, resolved: &ResolvedProgram) -> RustCode {
    let mut generator = CodeGenerator::new(resolved);
    generator.generate_code(rhir_program)
}
