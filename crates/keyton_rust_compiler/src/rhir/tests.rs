use super::{RExpr, RStmt, RStringPart, convert_to_rhir};
use crate::hir::hir_types::{HirBinOp, HirId};
use crate::hir::lower_program;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::shir::resolve_program;
use crate::shir::sym::{SymKind, SymbolId, Type};
use crate::thir::typecheck_program;

#[test]
fn program1_rhir() {
    let input = r#"x = 12
x = x + 1
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rust_program = convert_to_rhir(&typed, &resolved);

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

    // Full RHIR tree comparison - print should be converted to println! macro
    assert_eq!(
        rust_program.rhir,
        vec![
            RStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1),
                expr: RExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                    ty: Type::I64,
                },
            },
            RStmt::Assign {
                hir_id: HirId(3),
                sym: SymbolId(1),
                expr: RExpr::Binary {
                    hir_id: HirId(4),
                    left: Box::new(RExpr::Name {
                        hir_id: HirId(5),
                        sym: SymbolId(1),
                        ty: Type::I64,
                    }),
                    op: HirBinOp::Add,
                    right: Box::new(RExpr::Int {
                        hir_id: HirId(6),
                        value: 1,
                        ty: Type::I64,
                    }),
                    ty: Type::I64,
                },
            },
            RStmt::ExprStmt {
                hir_id: HirId(7),
                expr: RExpr::MacroCall {
                    hir_id: HirId(8),
                    macro_name: "println!".to_string(),
                    args: vec![RExpr::Name {
                        hir_id: HirId(10),
                        sym: SymbolId(1),
                        ty: Type::I64,
                    }],
                    ty: Type::Unit,
                },
            },
        ]
    );

    // Var types snapshot includes x: Int
    assert_eq!(rust_program.var_types.get(&SymbolId(1)), Some(&Type::I64));
}

#[test]
fn program4_rhir() {
    let input = r#"x = 12
x = "Hello"
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rust_program = convert_to_rhir(&typed, &resolved);

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

    assert_eq!(
        rust_program.rhir,
        vec![
            RStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1), // First x assignment
                expr: RExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                    ty: Type::I64,
                },
            },
            RStmt::Assign {
                hir_id: HirId(3),
                sym: SymbolId(2), // Second x assignment - new symbol
                expr: RExpr::Str {
                    hir_id: HirId(4),
                    value: "Hello".to_string(),
                    ty: Type::Str,
                },
            },
            RStmt::ExprStmt {
                hir_id: HirId(5),
                expr: RExpr::MacroCall {
                    hir_id: HirId(6),
                    macro_name: "println!".to_string(),
                    args: vec![RExpr::Name {
                        hir_id: HirId(8),
                        sym: SymbolId(2), // Uses the shadowed x (SymbolId(2))
                        ty: Type::Str,
                    }],
                    ty: Type::Unit,
                },
            },
        ]
    );

    // Var type snapshot: x(1): Int, x(2): Str
    assert_eq!(rust_program.var_types.get(&SymbolId(1)), Some(&Type::I64));
    assert_eq!(rust_program.var_types.get(&SymbolId(2)), Some(&Type::Str));
}

#[test]
fn program2_rhir() {
    let input = r#"print("Hello, World")"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rust_program = convert_to_rhir(&typed, &resolved);

    assert!(
        typed.report.errors.is_empty(),
        "type errors: {:?}",
        typed.report.errors
    );

    assert_eq!(
        rust_program.rhir,
        vec![RStmt::ExprStmt {
            hir_id: HirId(1),
            expr: RExpr::MacroCall {
                hir_id: HirId(2),
                macro_name: "println!".to_string(),
                args: vec![RExpr::Str {
                    hir_id: HirId(4),
                    value: "Hello, World".to_string(),
                    ty: Type::Str,
                }],
                ty: Type::Unit,
            },
        }]
    );
}

#[test]
fn program3_rhir() {
    let input = r#"x = 12
print(f"{x}")
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rust_program = convert_to_rhir(&typed, &resolved);

    assert!(
        typed.report.errors.is_empty(),
        "type errors: {:?}",
        typed.report.errors
    );

    assert_eq!(
        rust_program.rhir,
        vec![
            RStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1),
                expr: RExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                    ty: Type::I64,
                },
            },
            RStmt::ExprStmt {
                hir_id: HirId(3),
                expr: RExpr::MacroCall {
                    hir_id: HirId(4),
                    macro_name: "println!".to_string(),
                    args: vec![RExpr::InterpolatedString {
                        hir_id: HirId(6),
                        parts: vec![
                            RStringPart::Text {
                                hir_id: HirId(7),
                                value: "".to_string(),
                            },
                            RStringPart::Expr {
                                hir_id: HirId(8),
                                expr: RExpr::Name {
                                    hir_id: HirId(9),
                                    sym: SymbolId(1),
                                    ty: Type::I64,
                                },
                            },
                            RStringPart::Text {
                                hir_id: HirId(10),
                                value: "".to_string(),
                            },
                        ],
                        ty: Type::Str,
                    }],
                    ty: Type::Unit,
                },
            },
        ]
    );
}

#[test]
fn program5_rhir_same_type_reuse() {
    let input = r#"x = 12
x = 42
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rust_program = convert_to_rhir(&typed, &resolved);

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

    assert_eq!(
        rust_program.rhir,
        vec![
            RStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1), // First x assignment
                expr: RExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                    ty: Type::I64,
                },
            },
            RStmt::Assign {
                hir_id: HirId(3),
                sym: SymbolId(1), // Second x assignment - same symbol reused
                expr: RExpr::Int {
                    hir_id: HirId(4),
                    value: 42,
                    ty: Type::I64,
                },
            },
            RStmt::ExprStmt {
                hir_id: HirId(5),
                expr: RExpr::MacroCall {
                    hir_id: HirId(6),
                    macro_name: "println!".to_string(),
                    args: vec![RExpr::Name {
                        hir_id: HirId(8),
                        sym: SymbolId(1), // Uses the same x (SymbolId(1))
                        ty: Type::I64,
                    }],
                    ty: Type::Unit,
                },
            },
        ]
    );

    // Var type snapshot: x(1): Int (same symbol reused)
    assert_eq!(rust_program.var_types.get(&SymbolId(1)), Some(&Type::I64));
}

#[test]
fn test_function_mapping() {
    // Test that print function calls are properly converted to println! macro calls
    let input = r#"print("test")
print(123)
print(f"value: {42}")"#;

    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    let rust_program = convert_to_rhir(&typed, &resolved);

    assert_eq!(rust_program.rhir.len(), 3);

    // Check that all print calls were converted to println! macro calls
    for stmt in &rust_program.rhir {
        if let RStmt::ExprStmt { expr, .. } = stmt {
            if let RExpr::MacroCall { macro_name, .. } = expr {
                assert_eq!(macro_name, "println!");
            } else {
                panic!("Expected MacroCall, got {:?}", expr);
            }
        } else {
            panic!("Expected ExprStmt, got {:?}", stmt);
        }
    }
}
