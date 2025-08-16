use super::resolver::ResolveError;
use super::*;
use crate::hir::hir_types::{HirBinOp, HirId};
use crate::hir::lower_program;
use crate::lexer::Lexer;
use crate::parser::Parser;

#[test]
fn program1_rhir() {
    let input = r#"x = 12
x = x + 1
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);

    assert_eq!(
        resolved.rhir,
        vec![
            RStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1),
                expr: RExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
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
                    }),
                    op: HirBinOp::Add,
                    right: Box::new(RExpr::Int {
                        hir_id: HirId(6),
                        value: 1,
                    }),
                },
            },
            RStmt::ExprStmt {
                hir_id: HirId(7),
                expr: RExpr::Call {
                    hir_id: HirId(8),
                    func: Box::new(RExpr::Name {
                        hir_id: HirId(9),
                        sym: SymbolId(0),
                    }),
                    args: vec![RExpr::Name {
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
fn program2_rhir() {
    let input = r#"print("Hello, World")"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let resolved = resolve_program(&hir);
    assert_eq!(
        resolved.rhir,
        vec![RStmt::ExprStmt {
            hir_id: HirId(1),
            expr: RExpr::Call {
                hir_id: HirId(2),
                func: Box::new(RExpr::Name {
                    hir_id: HirId(3),
                    sym: SymbolId(0),
                }),
                args: vec![RExpr::Str {
                    hir_id: HirId(4),
                    value: "Hello, World".to_string(),
                }],
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
    let resolved = resolve_program(&hir);
    assert_eq!(
        resolved.rhir,
        vec![
            RStmt::Assign {
                hir_id: HirId(1),
                sym: SymbolId(1),
                expr: RExpr::Int {
                    hir_id: HirId(2),
                    value: 12,
                },
            },
            RStmt::ExprStmt {
                hir_id: HirId(3),
                expr: RExpr::Call {
                    hir_id: HirId(4),
                    func: Box::new(RExpr::Name {
                        hir_id: HirId(5),
                        sym: SymbolId(0),
                    }),
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
                                },
                            },
                            RStringPart::Text {
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
    let hir = lower_program(ast);

    let mut resolver = Resolver::new();
    let rhir = resolver.resolve_program(&hir);

    // Expect: define y (sid 0), then define x during lookup_name (sid 1)
    assert_eq!(
        rhir,
        vec![RStmt::Assign {
            hir_id: HirId(1),
            sym: SymbolId(0),
            expr: RExpr::Name {
                hir_id: HirId(2),
                sym: SymbolId(1),
            },
        }]
    );

    assert_eq!(resolver.report.errors.len(), 1);
    match &resolver.report.errors[0] {
        ResolveError::UnresolvedName { hir_id, name } => {
            assert_eq!(*hir_id, HirId(2));
            assert_eq!(name, "x");
        }
    }

    // Symbols: y then x, both globals in scope 0
    assert_eq!(resolver.syms.infos.len(), 2);
    assert_eq!(resolver.syms.infos[0].name, "y");
    assert_eq!(resolver.syms.infos[1].name, "x");
    assert_eq!(resolver.syms.infos[0].kind, SymKind::GlobalVar);
    assert_eq!(resolver.syms.infos[1].kind, SymKind::GlobalVar);
}
