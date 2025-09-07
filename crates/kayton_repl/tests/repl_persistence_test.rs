use anyhow::Result;
use kayton_interactive_shared::{InteractiveState, execute_prepared, prepare_input};

#[test]
fn variable_persists_across_lines() -> Result<()> {
    let mut state = InteractiveState::new();

    // First line assigns a value to x
    let prepared1 = prepare_input(&mut state, "x = 1")?;
    execute_prepared(&mut state, &prepared1)?;

    // Second line evaluates x so the value is captured in __last
    let prepared2 = prepare_input(&mut state, "x")?;
    execute_prepared(&mut state, &prepared2)?;

    let last_text = if let Some(h) = state.vm().resolve_name("__last") {
        state.vm_mut().format_value_by_handle(h).unwrap_or_default()
    } else {
        String::new()
    };
    assert_eq!(last_text, "1");

    Ok(())
}

#[test]
fn define_function_in_one_block_and_call_it() -> Result<()> {
    let mut state = InteractiveState::new();

    let code = r#"fn my_sum(x, y):
    x + y
x = 1
y = 2
my_sum(x,y)"#;
    let prepared = prepare_input(&mut state, code)?;
    execute_prepared(&mut state, &prepared)?;

    let last_text = if let Some(h) = state.vm().resolve_name("__last") {
        state.vm_mut().format_value_by_handle(h).unwrap_or_default()
    } else {
        String::new()
    };
    assert_eq!(last_text, "3");

    Ok(())
}

#[test]
fn print_all_values_in_one_test() -> Result<()> {
    let mut state = InteractiveState::new();

    let code = r#"x = 1
fn my_sum(a,b):
    a + b
y = my_sum(x,2)
print(x)
print(y)"#;
    let prepared = prepare_input(&mut state, code)?;
    execute_prepared(&mut state, &prepared)?;

    let text = if let Some(h) = state.vm().resolve_name("__stdout") {
        let s = state.vm_mut().format_value_by_handle(h).unwrap_or_default();
        s.trim_end_matches('\n').to_string()
    } else {
        String::new()
    };
    assert_eq!(text, "1\n3");

    Ok(())
}
