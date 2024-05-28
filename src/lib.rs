#![allow(dead_code)]
#![allow(unused)]

pub mod ast;
pub mod gen;
pub mod lex;

pub fn compile(src: String) -> String {
    let tokens = lex::lex(src);
    let ast = ast::parse(&tokens).unwrap();
    let buf = gen::QbeGenerator::new().gen(&ast).unwrap();

    buf
}
