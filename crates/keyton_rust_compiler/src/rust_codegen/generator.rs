use std::collections::HashMap;

use crate::rhir::types::{RExpr, RStmt, RStringPart, RustProgram};
use crate::shir::resolver::ResolvedProgram;
use crate::shir::sym::{SymbolId, Type};

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
            if self.should_skip_stmt(stmt) {
                continue;
            }
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

    /// Same as generate_code but seeds the assigned set and inserts a prelude at the top of main.
    pub fn generate_code_with_preassigned_and_prelude(
        &mut self,
        rhir_program: &RustProgram,
        pre_assigned: &std::collections::HashSet<SymbolId>,
        prelude_lines: &[String],
        epilogue_lines: &[String],
    ) -> RustCode {
        // Seed with already-assigned variable symbols so we emit `x = ...;` instead of `let mut x = ...;`
        for sym in pre_assigned.iter() {
            self.assigned_vars.insert(*sym);
        }

        let mut source_code = String::new();
        source_code.push_str("fn main() {\n");

        // Insert prelude lines (already indented by 4 spaces here)
        for line in prelude_lines {
            source_code.push_str("    ");
            source_code.push_str(line);
            if !line.ends_with('\n') {
                source_code.push('\n');
            }
        }

        // Determine the last non-skipped expression statement with non-Unit type
        let mut last_expr_idx: Option<(usize, Type)> = None;
        for (idx, stmt) in rhir_program.rhir.iter().enumerate() {
            if self.should_skip_stmt(stmt) {
                continue;
            }
            if let RStmt::ExprStmt { expr, .. } = stmt {
                let ty = expr.ty().clone();
                if ty != Type::Unit {
                    last_expr_idx = Some((idx, ty));
                }
            }
        }

        // Generate statements, capturing last expression value if present
        for (idx, stmt) in rhir_program.rhir.iter().enumerate() {
            if self.should_skip_stmt(stmt) {
                continue;
            }
            source_code.push_str("    ");
            match (idx, stmt) {
                (i, RStmt::ExprStmt { expr, .. })
                    if last_expr_idx.as_ref().map(|(j, _)| *j) == Some(i) =>
                {
                    // Capture last expression value into a temp so we can report it later without re-evaluating
                    let expr_str = self.convert_expr_to_string(expr);
                    source_code.push_str("let __kayton_last = ");
                    source_code.push_str(&expr_str);
                    source_code.push_str(";\n");
                }
                _ => {
                    source_code.push_str(&self.convert_stmt_to_string(stmt));
                    source_code.push_str("\n");
                }
            }
        }

        // Insert epilogue lines at end of main
        for line in epilogue_lines {
            source_code.push_str("    ");
            source_code.push_str(line);
            if !line.ends_with('\n') {
                source_code.push('\n');
            }
        }

        // Also report the captured last expression value if any
        if let Some((_, ty)) = last_expr_idx {
            match ty {
                Type::I64 => {
                    source_code
                        .push_str("    unsafe { report_int(\"__last\", __kayton_last as i64); }\n");
                }
                Type::Str => {
                    source_code
                        .push_str("    unsafe { report_str(\"__last\", &__kayton_last); }\n");
                }
                _ => {}
            }
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
            RStmt::ForRange {
                sym,
                start,
                end,
                body,
                ..
            } => {
                let var_name = self.get_or_create_var_name(*sym);
                let start_str = self.convert_expr_to_string(start);
                let end_str = self.convert_expr_to_string(end);
                // Emit body statements with relative indentation (4 spaces). The caller adds the base indent.
                let mut out = String::new();
                out.push_str(&format!(
                    "for {} in {}..{} {{\n",
                    var_name, start_str, end_str
                ));
                for inner in body {
                    if self.should_skip_stmt(inner) {
                        continue;
                    }
                    out.push_str("    "); // 4 spaces relative to the block
                    out.push_str(&self.convert_stmt_to_string(inner));
                    out.push_str("\n");
                }
                out.push_str("}");
                out
            }
        }
    }

    fn should_skip_stmt(&self, stmt: &RStmt) -> bool {
        match stmt {
            RStmt::ExprStmt { expr, .. } => match expr {
                RExpr::Int { value, .. } if *value == 0 => true,
                _ => false,
            },
            _ => false,
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
            RExpr::Call { func, args, .. } => {
                if let RExpr::Name { sym, .. } = func.as_ref() {
                    if let Some(info) = self.resolved.symbols.infos.get(sym.0 as usize) {
                        match info.name.as_str() {
                            "vec" => {
                                let elems = args
                                    .iter()
                                    .map(|a| self.convert_expr_to_string(a))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                return format!("vec![{}]", elems);
                            }
                            "append" => {
                                let target = self.convert_expr_to_string(&args[0]);
                                let value = self.convert_expr_to_string(&args[1]);
                                return format!("{}.push({})", target, value);
                            }
                            "sum" => {
                                let target = self.convert_expr_to_string(&args[0]);
                                return format!("{}.iter().sum::<i64>()", target);
                            }
                            _ => {}
                        }
                    }
                }
                let func_str = self.convert_expr_to_string(func);
                let args_str = args
                    .iter()
                    .map(|a| self.convert_expr_to_string(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", func_str, args_str)
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
