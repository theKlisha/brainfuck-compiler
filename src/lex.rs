use std::{iter::Peekable, str::Chars};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    MoveL(usize),
    MoveR(usize),
    Read,
    Write,
    Inc(usize),
    Dec(usize),
    JmpZero,
    JmpNoZero,
}

struct LexerState<'a> {
    iter: Peekable<Chars<'a>>,
}

fn next(state: &mut LexerState) -> Option<Token> {
    let c = state.iter.peek()?;
    match c {
        '<' => {
            let mut count = 0;
            while let Some('<') = state.iter.peek() {
                count += 1;
                state.iter.next();
            }
            Some(Token::MoveL(count))
        }
        '>' => {
            let mut count = 0;
            while let Some('>') = state.iter.peek() {
                count += 1;
                state.iter.next();
            }
            Some(Token::MoveR(count))
        }
        '+' => {
            let mut count = 0;
            while let Some('+') = state.iter.peek() {
                count += 1;
                state.iter.next();
            }
            Some(Token::Inc(count))
        }
        '-' => {
            let mut count = 0;
            while let Some('-') = state.iter.peek() {
                count += 1;
                state.iter.next();
            }
            Some(Token::Dec(count))
        }
        '.' => {
            state.iter.next();
            Some(Token::Write)
        }
        ',' => {
            state.iter.next();
            Some(Token::Read)
        }
        '[' => {
            state.iter.next();
            Some(Token::JmpZero)
        }
        ']' => {
            state.iter.next();
            Some(Token::JmpNoZero)
        }
        _ => {
            while let Some(c) = state.iter.peek() {
                match c {
                    '<' | '>' | '+' | '-' | '.' | ',' | '[' | ']' => break,
                    _ => {
                        state.iter.next();
                    }
                }
            }
            
            next(state)
        }
    }
}

pub fn lex(input: String) -> Vec<Token> {
    let mut iter = input.chars().peekable();
    let mut state = LexerState { iter };
    let mut tokens = Vec::new();

    while let Some(token) = next(&mut state) {
        tokens.push(token);
    }

    tokens
}
