use super::resolver::ResolveError;
use super::*;
use crate::hir::hir_types::{HirBinOp, HirId};
use crate::hir::{lower_program, lower_program_with_spans};
use crate::lexer::Lexer;
use crate::parser::Parser;
extern crate alloc;
use serial_test::serial;
use std::fs;
use kayton_plugin_sdk::{kayton_manifest};

#[test]
fn program1_shir() {
    let input = r#"x = 12
x = x + 1
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);

    assert_eq!(
        resolved.shir,
        vec![
            SStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1),
                expr: SExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                },
            },
            SStmt::Assign {
                hir_id: HirId(3),
                sym: SymbolId(1),
                expr: SExpr::Binary {
                    hir_id: HirId(4),
                    left: Box::new(SExpr::Name {
                        hir_id: HirId(5),
                        sym: SymbolId(1),
                    }),
                    op: HirBinOp::Add,
                    right: Box::new(SExpr::Int {
                        hir_id: HirId(6),
                        value: 1,
                    }),
                },
            },
            SStmt::ExprStmt {
                hir_id: HirId(7),
                expr: SExpr::Call {
                    hir_id: HirId(8),
                    func: Box::new(SExpr::Name {
                        hir_id: HirId(9),
                        sym: SymbolId(0),
                    }),
                    args: vec![SExpr::Name {
                        hir_id: HirId(10),
                        sym: SymbolId(1),
                    }],
                },
            },
        ]
    );

    // Symbols: print (0, BuiltinFunc), x (1, GlobalVar)
    assert_eq!(resolved.symbols.infos.len(), 2);
    assert_eq!(resolved.symbols.infos[0].name, "print");
    assert_eq!(resolved.symbols.infos[1].name, "x");
    assert_eq!(resolved.symbols.infos[0].kind, SymKind::BuiltinFunc);
    assert_eq!(resolved.symbols.infos[1].kind, SymKind::GlobalVar);
}

#[test]
fn program4_shir() {
    let input = r#"x = 12
x = "Hello"
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);

    assert_eq!(
        resolved.shir,
        vec![
            SStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1),
                expr: SExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                },
            },
            SStmt::Assign {
                hir_id: HirId(3),
                sym: SymbolId(1),
                expr: SExpr::Str {
                    hir_id: HirId(4),
                    value: "Hello".to_string(),
                },
            },
            SStmt::ExprStmt {
                hir_id: HirId(5),
                expr: SExpr::Call {
                    hir_id: HirId(6),
                    func: Box::new(SExpr::Name {
                        hir_id: HirId(7),
                        sym: SymbolId(0),
                    }),
                    args: vec![SExpr::Name {
                        hir_id: HirId(8),
                        sym: SymbolId(1),
                    }],
                },
            },
        ]
    );

    // Symbols: print (0, BuiltinFunc), x (1, GlobalVar)
    assert_eq!(resolved.symbols.infos.len(), 2);
    assert_eq!(resolved.symbols.infos[0].name, "print");
    assert_eq!(resolved.symbols.infos[1].name, "x");
    assert_eq!(resolved.symbols.infos[0].kind, SymKind::BuiltinFunc);
    assert_eq!(resolved.symbols.infos[1].kind, SymKind::GlobalVar);
}

#[test]
fn program2_shir() {
    let input = r#"print("Hello, World")"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);
    assert_eq!(
        resolved.shir,
        vec![SStmt::ExprStmt {
            hir_id: HirId(1),
            expr: SExpr::Call {
                hir_id: HirId(2),
                func: Box::new(SExpr::Name {
                    hir_id: HirId(3),
                    sym: SymbolId(0),
                }),
                args: vec![SExpr::Str {
                    hir_id: HirId(4),
                    value: "Hello, World".to_string(),
                }],
            },
        }]
    );
}

#[test]
fn program3_shir() {
    let input = r#"x = 12
print(f"{x}")
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);
    assert_eq!(
        resolved.shir,
        vec![
            SStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1),
                expr: SExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                },
            },
            SStmt::ExprStmt {
                hir_id: HirId(3),
                expr: SExpr::Call {
                    hir_id: HirId(4),
                    func: Box::new(SExpr::Name {
                        hir_id: HirId(5),
                        sym: SymbolId(0),
                    }),
                    args: vec![SExpr::InterpolatedString {
                        hir_id: HirId(6),
                        parts: vec![
                            SStringPart::Text {
                                hir_id: HirId(7),
                                value: "".to_string(),
                            },
                            SStringPart::Expr {
                                hir_id: HirId(8),
                                expr: SExpr::Name {
                                    hir_id: HirId(9),
                                    sym: SymbolId(1),
                                },
                            },
                            SStringPart::Text {
                                hir_id: HirId(10),
                                value: "".to_string(),
                            },
                        ],
                    }],
                },
            },
        ]
    );
}

#[test]
fn unresolved_name_reports_error() {
    // x is used but never defined; should be reported and given a fresh symbol id
    let input = r#"y = x"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let (hir, spans) = lower_program_with_spans(ast);

    let mut resolver = Resolver::new(spans);
    let shir = resolver.resolve_program(&hir);

    // Expect: define y (sid 0), then define x during lookup_name (sid 1)
    assert_eq!(
        shir,
        vec![SStmt::Assign {
            hir_id: HirId(1),
            sym: SymbolId(0),
            expr: SExpr::Name {
                hir_id: HirId(2),
                sym: SymbolId(1),
            },
        }]
    );

    assert_eq!(resolver.report.errors.len(), 1);
    match &resolver.report.errors[0] {
        ResolveError::UnresolvedName { span, name } => {
            assert_eq!(*span, crate::span::Span::new(2, 2));
            assert_eq!(name, "x");
        }
        ResolveError::ImportError { .. } => {}
    }

    // Symbols: y then x, both globals in scope 0
    assert_eq!(resolver.syms.infos.len(), 2);
    assert_eq!(resolver.syms.infos[0].name, "y");
    assert_eq!(resolver.syms.infos[1].name, "x");
    assert_eq!(resolver.syms.infos[0].kind, SymKind::GlobalVar);
    assert_eq!(resolver.syms.infos[1].kind, SymKind::GlobalVar);
}

#[test]
fn unresolved_name_in_call_reports_error() {
    // x is used in a call but never defined; should be reported
    let input = r#"print(x)"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let (hir, spans) = lower_program_with_spans(ast);

    let mut resolver = Resolver::new(spans);
    resolver.add_builtin("print");
    let shir = resolver.resolve_program(&hir);

    assert_eq!(
        shir,
        vec![SStmt::ExprStmt {
            hir_id: HirId(1),
            expr: SExpr::Call {
                hir_id: HirId(2),
                func: Box::new(SExpr::Name {
                    hir_id: HirId(3),
                    sym: SymbolId(0),
                }),
                args: vec![SExpr::Name {
                    hir_id: HirId(4),
                    sym: SymbolId(1),
                }],
            },
        }]
    );

    assert_eq!(resolver.report.errors.len(), 1);
    match &resolver.report.errors[0] {
        ResolveError::UnresolvedName { name, .. } => {
            assert_eq!(name, "x");
        }
        ResolveError::ImportError { .. } => {}
    }

    // Symbols: print (builtin) then x (global)
    assert_eq!(resolver.syms.infos.len(), 2);
    assert_eq!(resolver.syms.infos[0].name, "print");
    assert_eq!(resolver.syms.infos[1].name, "x");
    assert_eq!(resolver.syms.infos[0].kind, SymKind::BuiltinFunc);
    assert_eq!(resolver.syms.infos[1].kind, SymKind::GlobalVar);
}

#[test]
#[serial]
fn rimport_items_loads_manifest_and_defines_symbols() {
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

    let input = "from math rimport add";
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);

    assert!(resolved.plugins.contains_key("math"));

    let add_info = resolved
        .symbols
        .infos
        .iter()
        .find(|i| i.name == "add")
        .expect("add symbol");
    assert_eq!(add_info.kind, SymKind::BuiltinFunc);
    assert_eq!(add_info.sig.as_ref().unwrap().params, vec![Type::I64, Type::I64]);
    assert_eq!(add_info.sig.as_ref().unwrap().ret, Type::I64);

    assert!(matches!(
        resolved.shir.as_slice(),
        [SStmt::RImportItems { module, items, .. }] if module == "math" && items == &vec!["add".to_string()]
    ));

    std::env::set_current_dir(old_dir).unwrap();
    unsafe {
        std::env::remove_var("KAYTON_ACTIVE_ENV");
    }
}
