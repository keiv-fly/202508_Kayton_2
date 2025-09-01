use anyhow::Result;
use kayton_interactive_shared::{InteractiveState, execute_prepared, prepare_input};

#[test]
fn program_output_matches_expected() -> Result<()> {
    let mut state = InteractiveState::new();

    // Execute the entire program in one block, mirroring a single Jupyter cell
    let code = r#"fn my(a,b):
    a+b
a=1
b=2
z = my(a,b)
print(z)"#;
    let prepared = prepare_input(&mut state, code)?;
    execute_prepared(&mut state, &prepared)?;

    // Format output exactly like the kernel's execute_result
    let mut lines: Vec<String> = Vec::new();
    for (name, handle) in state.vm().snapshot_globals() {
        let vm_ref = state.vm_mut();
        match vm_ref.format_value_by_handle(handle) {
            Ok(s) => lines.push(format!("{} = {}", name, s)),
            Err(_) => lines.push(format!("{} = <error>", name)),
        }
    }
    let text = if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n")
    };

    assert_eq!(text, "a = 1\nb = 2\nz = 3");

    Ok(())
}
