# 202508_Kayton_2
A programming language

Run REPL with `cargo run --bin kayton_repl`

Run tests with `cargo nextest run --status-level=fail`

To register a jupyter kernel: `./target/debug/kayton_kernel.exe --install`

To calculate lines in all files to limit the number of lines in a file to 500: `python notebooks/calc_lines_in_files.py`
