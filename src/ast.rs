use core::slice;
use std::{
    fmt::{self, Display},
    iter, usize,
};

use crate::lex::Token;

type ParserResult<I, O, E> = Result<(I, O), ParserError<E>>;

#[derive(Debug)]
enum ParserError<E> {
    Err(E),
    Failure(E),
}

#[derive(Debug)]
pub enum Error {
    UnexpectedToken(Token),
    EndOfInput,
}

fn take_one_of(input: Tokens, of: Token) -> ParserResult<Tokens, Token, Error> {
    let i = input.first().ok_or(ParserError::Err(Error::EndOfInput))?;
    if i == &of {
        Ok((&input[1..], of))
    } else {
        Err(ParserError::Err(Error::UnexpectedToken(i.clone())))
    }
}

type Tokens<'a> = &'a [Token];

trait Node
where
    Self: Sized,
{
    fn parse(input: Tokens) -> ParserResult<Tokens, Self, Error>;
    fn print(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result;
}

pub type Prog = NodeBlock;
pub type Ast = NodeBlock;

impl Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.print(f, 0)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Attr {}

#[derive(Debug, PartialEq)]
pub struct NodeBlock {
    pub attr: Attr,
    pub stats: Vec<NodeStatement>,
}

impl Node for NodeBlock {
    fn parse(input: Tokens) -> ParserResult<Tokens, Self, Error> {
        let mut input = input;
        let mut node = Self {
            attr: Attr::default(),
            stats: Vec::new(),
        };

        loop {
            match NodeStatement::parse(input) {
                Err(ParserError::Failure(f)) => return ParserResult::Err(ParserError::Failure(f)),
                Err(ParserError::Err(_)) => break,
                Ok((rest, stat)) => {
                    input = rest;
                    node.stats.push(stat);
                }
            };
        }

        ParserResult::Ok((input, node))
    }

    fn print(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        writeln!(f, "{:indent$}Block", "", indent = depth * 2)?;
        self.stats
            .iter()
            .try_for_each(|stat| stat.print(f, depth + 1))
    }
}

#[derive(Debug, PartialEq)]
pub struct NodeStatement {
    pub attr: Attr,
    pub stat: Statement,
}

impl Node for NodeStatement {
    fn parse(input: Tokens) -> ParserResult<Tokens, Self, Error> {
        let t = input.first().ok_or(ParserError::Err(Error::EndOfInput))?;
        match t {
            Token::MoveL(count) => ParserResult::Ok((
                &input[1..],
                NodeStatement {
                    attr: Attr::default(),
                    stat: Statement::MoveL(*count),
                },
            )),
            Token::MoveR(count) => ParserResult::Ok((
                &input[1..],
                NodeStatement {
                    attr: Attr::default(),
                    stat: Statement::MoveR(*count),
                },
            )),
            Token::Read => ParserResult::Ok((
                &input[1..],
                NodeStatement {
                    attr: Attr::default(),
                    stat: Statement::Read,
                },
            )),
            Token::Write => ParserResult::Ok((
                &input[1..],
                NodeStatement {
                    attr: Attr::default(),
                    stat: Statement::Write,
                },
            )),
            Token::Inc(count) => ParserResult::Ok((
                &input[1..],
                NodeStatement {
                    attr: Attr::default(),
                    stat: Statement::Add(*count),
                },
            )),
            Token::Dec(count) => ParserResult::Ok((
                &input[1..],
                NodeStatement {
                    attr: Attr::default(),
                    stat: Statement::Sub(*count),
                },
            )),
            Token::JmpZero => {
                let (input, block) = NodeBlock::parse(&input[1..])?;
                let (input, _) = take_one_of(input, Token::JmpNoZero)?;

                ParserResult::Ok((
                    input,
                    NodeStatement {
                        attr: Attr::default(),
                        stat: Statement::Loop(Box::new(block)),
                    },
                ))
            }
            _ => ParserResult::Err(ParserError::Err(Error::UnexpectedToken(t.clone()))),
        }
    }

    fn print(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        match &self.stat {
            Statement::Loop(block) => {
                writeln!(f, "{:indent$}Loop", "", indent = depth * 2)?;
                block.print(f, depth + 1)
            }
            _ => writeln!(f, "{:indent$}{:?}", "", self.stat, indent = depth * 2),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    MoveL(usize),
    MoveR(usize),
    Add(usize),
    Sub(usize),
    Read,
    Write,
    Loop(Box<NodeBlock>),
}

pub fn parse(input: Tokens) -> Result<Ast, Error> {
    match NodeBlock::parse(input) {
        Err(ParserError::Err(e)) => Err(e),
        Err(ParserError::Failure(e)) => Err(e),
        Ok((rest, ast)) => {
            if !rest.is_empty() {
                Err(Error::EndOfInput)
            } else {
                Ok(ast)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lex::Token;

    #[test]
    fn parse_statement() {
        let input = vec![Token::MoveL(1)];
        let (rest, ast) = NodeStatement::parse(&input).unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(ast.stat, Statement::MoveL(1));

        let input = vec![Token::MoveR(1)];
        let (rest, ast) = NodeStatement::parse(&input).unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(ast.stat, Statement::MoveR(1));
    }

    #[test]
    fn parse_block_1() {
        let input = vec![Token::MoveL(1), Token::MoveR(1)];
        let (rest, ast) = super::NodeBlock::parse(&input).unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(ast.stats.len(), 2);
        assert_eq!(ast.stats[0].stat, Statement::MoveL(1));
        assert_eq!(ast.stats[1].stat, Statement::MoveR(1));
    }

    #[test]
    fn parse_block_2() {
        let input = vec![
            Token::MoveL(1),
            Token::JmpZero,
            Token::MoveR(1),
            Token::JmpNoZero,
        ];
        let (rest, ast) = NodeBlock::parse(&input).unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(ast.stats.len(), 2);
        assert_eq!(ast.stats[0].stat, Statement::MoveL(1));

        if let Statement::Loop(block) = &ast.stats[1].stat {
            assert_eq!(block.stats.len(), 1);
            assert_eq!(block.stats[0].stat, Statement::MoveR(1));
        } else {
            panic!("Expected loop statement");
        }
    }
}
