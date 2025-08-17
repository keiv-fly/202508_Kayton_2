use super::{TExpr, TStmt, TStringPart, typecheck_program};
use crate::hir::hir_types::{HirBinOp, HirId};
use crate::hir::lower_program;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::shir::resolve_program;
use crate::shir::sym::{SymKind, SymbolId, Type};

#[test]
fn program1_thir() {
    let input = r#"x = 12
x = x + 1
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);
    let typed = typecheck_program(&resolved);

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

    // Full THIR tree comparison
    assert_eq!(
        typed.thir,
        vec![
            TStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1),
                expr: TExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                    ty: Type::Int,
                },
            },
            TStmt::Assign {
                hir_id: HirId(3),
                sym: SymbolId(1),
                expr: TExpr::Binary {
                    hir_id: HirId(4),
                    left: Box::new(TExpr::Name {
                        hir_id: HirId(5),
                        sym: SymbolId(1),
                        ty: Type::Int,
                    }),
                    op: HirBinOp::Add,
                    right: Box::new(TExpr::Int {
                        hir_id: HirId(6),
                        value: 1,
                        ty: Type::Int,
                    }),
                    ty: Type::Int,
                },
            },
            TStmt::ExprStmt {
                hir_id: HirId(7),
                expr: TExpr::Call {
                    hir_id: HirId(8),
                    func: Box::new(TExpr::Name {
                        hir_id: HirId(9),
                        sym: SymbolId(0),
                        ty: Type::Any,
                    }),
                    args: vec![TExpr::Name {
                        hir_id: HirId(10),
                        sym: SymbolId(1),
                        ty: Type::Int,
                    }],
                    ty: Type::Unit,
                },
            },
        ]
    );

    // Var types snapshot includes x: Int
    assert_eq!(typed.var_types.get(&SymbolId(1)), Some(&Type::Int));
}

#[test]
fn program4_thir() {
    let input = r#"x = 12
x = "Hello"
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);
    let typed = typecheck_program(&resolved);

    // No hard error on reassignment; types unify to Any.
    assert!(
        typed.report.errors.is_empty(),
        "unexpected type errors: {:?}",
        typed.report.errors
    );

    // Symbols: print (0, BuiltinFunc), x (1, GlobalVar)
    assert_eq!(resolved.symbols.infos.len(), 2);
    assert_eq!(resolved.symbols.infos[0].name, "print");
    assert_eq!(resolved.symbols.infos[1].name, "x");
    assert_eq!(resolved.symbols.infos[0].kind, SymKind::BuiltinFunc);
    assert_eq!(resolved.symbols.infos[1].kind, SymKind::GlobalVar);

    assert_eq!(
        typed.thir,
        vec![
            TStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1),
                expr: TExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                    ty: Type::Int,
                },
            },
            TStmt::Assign {
                hir_id: HirId(3),
                sym: SymbolId(1),
                expr: TExpr::Str {
                    hir_id: HirId(4),
                    value: "Hello".to_string(),
                    ty: Type::Str,
                },
            },
            TStmt::ExprStmt {
                hir_id: HirId(5),
                expr: TExpr::Call {
                    hir_id: HirId(6),
                    func: Box::new(TExpr::Name {
                        hir_id: HirId(7),
                        sym: SymbolId(0),
                        ty: Type::Any,
                    }),
                    args: vec![TExpr::Name {
                        hir_id: HirId(8),
                        sym: SymbolId(1),
                        ty: Type::Any,
                    }],
                    ty: Type::Unit,
                },
            },
        ]
    );

    // Var type snapshot: x unified to Any after Int then Str assignments
    assert_eq!(typed.var_types.get(&SymbolId(1)), Some(&Type::Any));
}

#[test]
fn program2_thir() {
    let input = r#"print("Hello, World")"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);
    let typed = typecheck_program(&resolved);

    assert!(
        typed.report.errors.is_empty(),
        "type errors: {:?}",
        typed.report.errors
    );

    assert_eq!(
        typed.thir,
        vec![TStmt::ExprStmt {
            hir_id: HirId(1),
            expr: TExpr::Call {
                hir_id: HirId(2),
                func: Box::new(TExpr::Name {
                    hir_id: HirId(3),
                    sym: SymbolId(0),
                    ty: Type::Any,
                }),
                args: vec![TExpr::Str {
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
fn program3_thir() {
    let input = r#"x = 12
print(f"{x}")
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);
    let typed = typecheck_program(&resolved);

    assert!(
        typed.report.errors.is_empty(),
        "type errors: {:?}",
        typed.report.errors
    );

    assert_eq!(
        typed.thir,
        vec![
            TStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1),
                expr: TExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                    ty: Type::Int,
                },
            },
            TStmt::ExprStmt {
                hir_id: HirId(3),
                expr: TExpr::Call {
                    hir_id: HirId(4),
                    func: Box::new(TExpr::Name {
                        hir_id: HirId(5),
                        sym: SymbolId(0),
                        ty: Type::Any,
                    }),
                    args: vec![TExpr::InterpolatedString {
                        hir_id: HirId(6),
                        parts: vec![
                            TStringPart::Text {
                                hir_id: HirId(7),
                                value: "".to_string(),
                            },
                            TStringPart::Expr {
                                hir_id: HirId(8),
                                expr: TExpr::Name {
                                    hir_id: HirId(9),
                                    sym: SymbolId(1),
                                    ty: Type::Int,
                                },
                            },
                            TStringPart::Text {
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
