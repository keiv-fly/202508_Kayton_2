// Example usage of the SHIR (Symbol HIR) module
// This shows the complete pipeline from source code to typed IR

use crate::hir::lower_program;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::rhir::resolve_program;
use crate::shir::typecheck_program;

pub fn example_usage() {
    let source_code = r#"
x = 42
y = x + 10
print(y)
print("Hello, World!")
"#;

    // Step 1: Lexical analysis
    let tokens = Lexer::new(source_code).tokenize();
    println!("Tokens: {:?}", tokens);

    // Step 2: Parsing
    let ast = Parser::new(tokens).parse_program();
    println!("AST: {:?}", ast);

    // Step 3: Lower to HIR
    let hir = lower_program(ast);
    println!("HIR: {:?}", hir);

    // Step 4: Resolve names (RHIR)
    let resolved = resolve_program(&hir);
    println!("Resolved symbols: {:?}", resolved.symbols.infos);

    // Step 5: Type checking (SHIR)
    let typed = typecheck_program(&resolved);
    println!("Type errors: {:?}", typed.report.errors);
    println!("Variable types: {:?}", typed.var_types);
    println!("Typed IR: {:?}", typed.shir);

    // Check if there were any type errors
    if typed.report.errors.is_empty() {
        println!("✅ Type checking passed!");
    } else {
        println!(
            "❌ Type checking failed with {} errors",
            typed.report.errors.len()
        );
    }
}
