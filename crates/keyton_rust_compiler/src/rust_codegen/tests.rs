use super::generate_rust_code;
use crate::hir::lower_program;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::rhir::convert_to_rhir;
use crate::shir::resolve_program;
use crate::shir::sym::{SymKind, SymbolId};
use crate::thir::typecheck_program;
extern crate alloc;
use serial_test::serial;
use std::fs;
use kayton_plugin_sdk::kayton_manifest;

#[test]
fn program1_rust_codegen() {
    let input = r#"x = 12
x = x + 1
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rhir_program = convert_to_rhir(&typed, &resolved);
    let rust_code = generate_rust_code(&rhir_program, &resolved);

    assert!(
        typed.report.errors.is_empty(),
        "type errors: {:?}",
        typed.report.errors
    );

    // Compare symbols: print (0, BuiltinFunc), x (1, GlobalVar)
    assert_eq!(resolved.symbols.infos.len(), 2);
    assert_eq!(resolved.symbols.infos[0].name, "print");
    assert_eq!(resolved.symbols.infos[1].name, "x");
    assert_eq!(resolved.symbols.infos[0].kind, SymKind::BuiltinFunc);
    assert_eq!(resolved.symbols.infos[1].kind, SymKind::GlobalVar);

    // Check the generated Rust code
    let expected_code = r#"fn main() {
    let mut x = 12;
    x = (x + 1);
    println!(x);
}
"#;
    assert_eq!(rust_code.source_code, expected_code);

    // Check variable name mapping
    assert_eq!(
        rust_code.var_names.get(&SymbolId(1)),
        Some(&"x".to_string())
    );
}

#[test]
fn program4_rust_codegen() {
    let input = r#"x = 12
x = "Hello"
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rhir_program = convert_to_rhir(&typed, &resolved);
    let rust_code = generate_rust_code(&rhir_program, &resolved);

    // No errors - shadowing is allowed
    assert!(
        typed.report.errors.is_empty(),
        "unexpected type errors: {:?}",
        typed.report.errors
    );

    // Symbols: print (0, BuiltinFunc), x (1, GlobalVar), x (2, GlobalVar) - shadowed
    assert_eq!(resolved.symbols.infos.len(), 3);
    assert_eq!(resolved.symbols.infos[0].name, "print");
    assert_eq!(resolved.symbols.infos[1].name, "x");
    assert_eq!(resolved.symbols.infos[2].name, "x");
    assert_eq!(resolved.symbols.infos[0].kind, SymKind::BuiltinFunc);
    assert_eq!(resolved.symbols.infos[1].kind, SymKind::GlobalVar);
    assert_eq!(resolved.symbols.infos[2].kind, SymKind::GlobalVar);

    // Check the generated Rust code
    let expected_code = r#"fn main() {
    let mut x = 12;
    let mut x_0 = "Hello";
    println!(x_0);
}
"#;
    assert_eq!(rust_code.source_code, expected_code);

    // Check variable name mappings
    assert_eq!(
        rust_code.var_names.get(&SymbolId(1)),
        Some(&"x".to_string())
    );
    assert_eq!(
        rust_code.var_names.get(&SymbolId(2)),
        Some(&"x_0".to_string())
    );
}

#[test]
fn program2_rust_codegen() {
    let input = r#"print("Hello, World")"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rhir_program = convert_to_rhir(&typed, &resolved);
    let rust_code = generate_rust_code(&rhir_program, &resolved);

    assert!(
        typed.report.errors.is_empty(),
        "type errors: {:?}",
        typed.report.errors
    );

    let expected_code = r#"fn main() {
    println!("Hello, World");
}
"#;
    assert_eq!(rust_code.source_code, expected_code);
}

#[test]
fn program_user_fn_inline_print_rust_codegen() {
    let input = r#"fn my_sum(x, y):
    x + y

x = 1
y = 2
print(my_sum(x,y))
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rhir_program = convert_to_rhir(&typed, &resolved);
    let rust_code = generate_rust_code(&rhir_program, &resolved);

    assert!(
        typed.report.errors.is_empty(),
        "type errors: {:?}",
        typed.report.errors
    );

    let expected_code = r#"fn main() {
    let mut x = 1;
    let mut y = 2;
    println!((x + y));
}
"#;
    assert_eq!(rust_code.source_code, expected_code);
}

#[test]
fn program3_rust_codegen() {
    let input = r#"x = 12
print(f"{x}")
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rhir_program = convert_to_rhir(&typed, &resolved);
    let rust_code = generate_rust_code(&rhir_program, &resolved);

    assert!(
        typed.report.errors.is_empty(),
        "type errors: {:?}",
        typed.report.errors
    );

    let expected_code = r#"fn main() {
    let mut x = 12;
    println!(format!("{}", x));
}
"#;
    assert_eq!(rust_code.source_code, expected_code);
}

#[test]
fn program5_rust_codegen_same_type_reuse() {
    let input = r#"x = 12
x = 42
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rhir_program = convert_to_rhir(&typed, &resolved);
    let rust_code = generate_rust_code(&rhir_program, &resolved);

    // No errors - same type reuse is allowed
    assert!(
        typed.report.errors.is_empty(),
        "unexpected type errors: {:?}",
        typed.report.errors
    );

    // Symbols: print (0, BuiltinFunc), x (1, GlobalVar) - same symbol reused
    assert_eq!(resolved.symbols.infos.len(), 2);
    assert_eq!(resolved.symbols.infos[0].name, "print");
    assert_eq!(resolved.symbols.infos[1].name, "x");
    assert_eq!(resolved.symbols.infos[0].kind, SymKind::BuiltinFunc);
    assert_eq!(resolved.symbols.infos[1].kind, SymKind::GlobalVar);

    let expected_code = r#"fn main() {
    let mut x = 12;
    x = 42;
    println!(x);
}
"#;
    assert_eq!(rust_code.source_code, expected_code);

    // Check variable name mapping: x(1): x (same symbol reused)
    assert_eq!(
        rust_code.var_names.get(&SymbolId(1)),
        Some(&"x".to_string())
    );
}

#[test]
fn test_function_mapping_rust_codegen() {
    // Test that print function calls are properly converted to println! macro calls
    let input = r#"print("test")
print(123)
print(f"value: {42}")"#;

    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rhir_program = convert_to_rhir(&typed, &resolved);
    let rust_code = generate_rust_code(&rhir_program, &resolved);

    let expected_code = r#"fn main() {
    println!("test");
    println!(123);
    println!(format!("value: {}", 42));
}
"#;
    assert_eq!(rust_code.source_code, expected_code);
}

#[test]
fn test_string_interpolation_rust_codegen() {
    let input = r#"x = 42
y = "world"
print(f"Hello {x} {y}")"#;

    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rhir_program = convert_to_rhir(&typed, &resolved);
    let rust_code = generate_rust_code(&rhir_program, &resolved);

    let expected_code = r#"fn main() {
    let mut x = 42;
    let mut y = "world";
    println!(format!("Hello {} {}", x, y));
}
"#;
    assert_eq!(rust_code.source_code, expected_code);
}

#[test]
#[serial]
fn rimport_codegen_inserts_prelude() {
    let tmp = tempfile::tempdir().unwrap();
    let old_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    unsafe {
        std::env::set_var("KAYTON_ACTIVE_ENV", "local");
    }

    let kayton_dir = tmp.path().join(".kayton");
    fs::create_dir_all(kayton_dir.join("metadata")).unwrap();
    fs::write(kayton_dir.join("metadata").join("registry.json"), b"{}").unwrap();
    let manifest_dir = kayton_dir.join("libs").join("math").join("0.1.0").join("x");
    fs::create_dir_all(&manifest_dir).unwrap();
    let mani = kayton_manifest!(
        crate_name = "math",
        crate_version = "0.1.0",
        functions = [{ stable: "add", symbol: "add", params: [I64, I64], ret: I64 }],
        types = []
    );
    fs::write(manifest_dir.join("manifest.json"), mani.to_json_bytes()).unwrap();

    let input = "from math rimport add\nprint(add(1, 2))";
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    assert!(typed.report.errors.is_empty());
    let rhir = convert_to_rhir(&typed, &resolved);
    let rust_code = generate_rust_code(&rhir, &resolved);

    let src = &rust_code.source_code;
    assert!(src.contains("load_plugin(\"math\")"));
    assert!(src.contains("get_fn_ptr(\"add\")"));

    std::env::set_current_dir(old_dir).unwrap();
    unsafe {
        std::env::remove_var("KAYTON_ACTIVE_ENV");
    }
}
