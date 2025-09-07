use anyhow::Result;
use kayton_interactive_shared::{InteractiveState, execute_prepared, prepare_input};

#[test]
fn program_values_match_expected() -> Result<()> {
    let mut state = InteractiveState::new();

    // Execute the entire program in one block, mirroring a single Jupyter cell
    let code = r#"fn my(a,b):
    a+b
a=1
b=2
z = my(a,b)
z"#;
    let prepared = prepare_input(&mut state, code)?;
    execute_prepared(&mut state, &prepared)?;

    // Fetch values directly from the VM rather than relying on printed output
    let z_text = if let Some(h) = state.vm().resolve_name("z") {
        state.vm_mut().format_value_by_handle(h).unwrap_or_default()
    } else {
        String::new()
    };
    assert_eq!(z_text, "3");

    let a_text = if let Some(h) = state.vm().resolve_name("a") {
        state.vm_mut().format_value_by_handle(h).unwrap_or_default()
    } else {
        String::new()
    };
    assert_eq!(a_text, "1");

    // Verify that the last expression value is captured into __last
    let last_text = if let Some(h) = state.vm().resolve_name("__last") {
        state.vm_mut().format_value_by_handle(h).unwrap_or_default()
    } else {
        String::new()
    };
    assert_eq!(last_text, "3");

    Ok(())
}

#[test]
fn print_outputs_all_values() -> Result<()> {
    let mut state = InteractiveState::new();

    let code = r#"fn my(a,b):
    a+b
a=1
b=2
z = my(a,b)
print(z)
print(a)"#;
    let prepared = prepare_input(&mut state, code)?;
    execute_prepared(&mut state, &prepared)?;

    // Read captured printed output from __stdout and trim one trailing newline if present
    let text = if let Some(h) = state.vm().resolve_name("__stdout") {
        let s = state.vm_mut().format_value_by_handle(h).unwrap_or_default();
        s.trim_end_matches('\n').to_string()
    } else {
        String::new()
    };
    assert_eq!(text, "3\n1");

    Ok(())
}
