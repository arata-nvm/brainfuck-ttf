use std::time::Instant;

use brainfuck::execute_program;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <program> <input>", args[0]);
        std::process::exit(1);
    }
    let program = &args[1];
    let input = args
        .get(2)
        .map(|it| format!("{}\n", it.as_str()))
        .unwrap_or_default();

    let program = std::fs::read_to_string(program).expect("failed to read program");
    let start = Instant::now();
    let output = execute_program(&program, &input).unwrap_or(String::from("error"));
    let end = Instant::now();
    println!(
        "execution took {} ms",
        end.duration_since(start).as_millis()
    );
    println!("--- output ---");
    print!("{output}");
}
