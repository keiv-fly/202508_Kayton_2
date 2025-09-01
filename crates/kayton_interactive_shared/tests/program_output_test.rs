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

    // Read captured printed output from __stdout and trim one trailing newline if present
    let text = if let Some(h) = state.vm().resolve_name("__stdout") {
        let s = state.vm_mut().format_value_by_handle(h).unwrap_or_default();
        s.trim_end_matches('\n').to_string()
    } else {
        String::new()
    };

    assert_eq!(text, "3");

    Ok(())
}
