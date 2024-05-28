use brainfuck_compiler::{ast, gen, lex};
use std::{env, fs};

fn main() {
    let path = env::args().skip(1).next().expect("path expected");
    let src = fs::read_to_string(path).expect("could not read the file");
    let out = brainfuck_compiler::compile(src);
    println!("{}", out);
}
