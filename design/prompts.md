When I am running my language Keyton in Jupyter kernel, I want to be able to import a Rust library with all of its functions and types. I want to have a reqwest library as an example and make a request with it.
I want to be able to compile the libarary to a dll so that the library can be accessed through kayton_vm. For example, I compile the library. When loading send the function definition to the vm via kayton_api. This includes not only functions but also types, traits, macros. Whatever there is to import.
The functions with generics should have them implemented with i64, f64, &str, String, Vec<i64>, Vec<f64>, and a Dynamic type, which means any type that is registered with kayton_vm.

The syntax of import is:
```
rimport reqwest
from reqwest rimport Client, StatusCode
```

Could you make a high-level plan what needs to be implemented and what problems need to be solved in order to be able to import the Rust library?