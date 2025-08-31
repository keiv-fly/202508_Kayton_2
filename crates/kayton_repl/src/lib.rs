use std::io::{self, Write};

use anyhow::Result;
use kayton_interactive_shared::{
    InteractiveState, PreparedCode, VarKind, execute_prepared, prepare_input,
};

/// Run the Kayton REPL loop
pub fn run_repl() -> Result<()> {
    let mut state = InteractiveState::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        write!(stdout, ">>> ")?;
        stdout.flush()?;

        let mut line = String::new();
        let n = stdin.read_line(&mut line)?;
        if n == 0 {
            break;
        }

        let first_line_no_crlf = line.trim_end_matches(&['\n', '\r'][..]).to_string();
        let first_line_trimmed = first_line_no_crlf.trim();

        // Multiline function entry like Python: if first line starts with `fn` and ends with ':'
        if first_line_trimmed.starts_with("fn ") && first_line_trimmed.ends_with(':') {
            let mut block = String::new();
            block.push_str(&first_line_no_crlf);
            block.push('\n');

            // Read subsequent indented lines until a blank line is entered
            loop {
                write!(stdout, "...     ")?; // Python-style continuation prompt with 4-space visual indent
                stdout.flush()?;

                let mut cont = String::new();
                let m = stdin.read_line(&mut cont)?;
                if m == 0 {
                    break;
                }
                let cont_no_crlf = cont.trim_end_matches(&['\n', '\r'][..]);
                if cont_no_crlf.is_empty() {
                    break; // blank line terminates the block
                }
                // Ensure body lines are indented by 4 spaces
                block.push_str("    ");
                block.push_str(cont_no_crlf.trim_start());
                block.push('\n');
            }

            // Persist function definition across entries
            state.stored_functions.push(block);
            // Do not compile/run immediately; continue to next prompt
            continue;
        }

        // Skip empty input
        if first_line_trimmed.is_empty() {
            continue;
        }

        // Prepend stored function definitions to current source
        let prep = match prepare_input(&mut state, &first_line_no_crlf) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}", e);
                state.input_counter += 1;
                continue;
            }
        };
        if let Err(e) = execute_prepared(&mut state, &prep) {
            eprintln!("{}", e);
        }
        state.input_counter += 1;
    }

    Ok(())
}
