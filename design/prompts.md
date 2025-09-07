When I am running my language Keyton in Jupyter kernel, I want to be able to import a Rust library with all of its functions and types. I want to have a reqwest library as an example and make a request with it.
I want to be able to compile the libarary to a dll so that the library can be accessed through kayton_vm. For example, I compile the library. When loading send the function definition to the vm via kayton_api. This includes not only functions but also types, traits, macros. Whatever there is to import.
The functions with generics should have them implemented with i64, f64, &str, String, Vec<i64>, Vec<f64>, and a Dynamic type, which means any type that is registered with kayton_vm.

The syntax of import is:
```
rimport reqwest
from reqwest rimport Client, StatusCode
```

Could you make a high-level plan what needs to be implemented and what problems need to be solved in order to be able to import the Rust library?

---------------------

Execute step 4 and 5:
#### 6. Add rimport parsing to keyton_rust_compiler
- **Location**: `crates/keyton_rust_compiler/src/`
- **Tasks**:
  - Extend lexer for `rimport` and `from ... rimport ...` syntax
  - Add AST nodes for import statements
  - Update parser to handle import declarations
  - Add import statement to HIR
- **Dependencies**: None
- **Estimated effort**: 2-3 days

#### 7. Implement rimport name resolution and typechecking
- **Location**: `crates/keyton_rust_compiler/src/`
- **Tasks**:
  - Environment-aware name resolution
  - Plugin manifest loading and validation
  - Type checking against plugin signatures
  - Import error handling with remediation messages
  - Integration with existing typechecker
- **Dependencies**: Steps 2, 6
- **Estimated effort**: 4-5 days

Look into these documents for guidance:
@20250907_rimport_steps.md 
@20250907_rimport_plan.md 
@20250907_current_design_for_rimport.md 

Execute tests with `cargo nextest run --status-level=fail`.
Ignore the failing tests in compile_rust/tests when run with `cargo test`
If one run of `cargo nextest run --status-level=fail` fails then run it once again.