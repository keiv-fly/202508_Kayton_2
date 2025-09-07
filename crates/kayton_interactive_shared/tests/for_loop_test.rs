use anyhow::Result;
use kayton_interactive_shared::{InteractiveState, execute_prepared, prepare_input};

#[test]
fn for_loop_sums_to_three() -> Result<()> {
    let mut state = InteractiveState::new();

    let code = r#"n = 3
s = 0
for x in 0..n:
    s += x
"#;

    let prepared = prepare_input(&mut state, code)?;
    execute_prepared(&mut state, &prepared)?;

    // Fetch the reported value of s from the VM
    let s_text = if let Some(h) = state.vm().resolve_name("s") {
        state.vm_mut().format_value_by_handle(h).unwrap_or_default()
    } else {
        String::new()
    };
    assert_eq!(s_text, "3");

    Ok(())
}

