use crate::shir::resolver::ResolvedProgram;
use crate::thir::types::TypeError;

fn highlight_var_in_source(source: &str, var_name: &str, indent: &str) -> (String, usize) {
    let pos_opt = source.find(var_name);
    let highlighted_code_line = if let Some(p) = pos_opt {
        let before = &source[..p];
        let after = &source[p + var_name.len()..];
        format!("{}{}\x1b[31m{}\x1b[0m{}", indent, before, var_name, after)
    } else {
        format!("{}{}", indent, source)
    };
    let pos = pos_opt.unwrap_or(0);
    (highlighted_code_line, pos)
}

pub fn format_type_error(
    source: &str,
    resolved: &ResolvedProgram,
    err: &TypeError,
    file_label: &str,
) -> Option<String> {
    match err {
        TypeError::UnknownVarType { sym, .. } => {
            let var_name = &resolved.symbols.infos[sym.0 as usize].name;
            let indent = "    ";
            let (highlighted, pos) = highlight_var_in_source(source, var_name, indent);
            let caret_pos = indent.len() + pos;
            let caret_line = format!("{}\x1b[31m^\x1b[0m", " ".repeat(caret_pos));

            let mut out = String::new();
            out.push_str("\x1b[31mCompilation failed:\x1b[0m\n");
            out.push_str(&format!("  File \"{}\", line 1, in <module>\n", file_label));
            out.push_str(&highlighted);
            out.push('\n');
            out.push_str(&caret_line);
            out.push('\n');
            out.push_str(&format!(
                "\x1b[1;31mNameError\x1b[0m: name '{}' is not defined",
                var_name
            ));
            Some(out)
        }
        _ => None,
    }
}
