use core::{num, slice};
use std::{io::Lines, string};
use std::io;

const STRING_BORDER: char = '"';

#[derive(Debug)]
pub enum NonTerminal {
    ArgumentOpen,
    ArgumentClose,
    BlockOpen,
    BlockClose,
    Equal,
    TryReturn,
    Macro,
    StmtEnd,
    Plus,
    StringBorder,
    Dot,
    Number,
    Newline,
}

impl NonTerminal {
    fn try_from_char(c: char) -> Option<NonTerminal> {
        use NonTerminal::*;
        let nonterm = match c {
            '(' => ArgumentOpen,
            ')' => ArgumentClose,
            '{' => BlockOpen,
            '}' => BlockClose,
            '=' => Equal,
            '?' => TryReturn,
            '!' => Macro,
            ';' => StmtEnd,
            '+' => Plus,
            STRING_BORDER => StringBorder,
            '.' => Dot,
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => Number,
            _ => return None,
        };
        Some(nonterm)
    }
}

#[derive(Debug)]
enum FlowChar {
    Whitespace,
    Newline,
}

impl FlowChar {
    fn try_from_char(c: char) -> Option<FlowChar> {
        let flowchar = match c {
            '\n' => FlowChar::Newline,
            ' ' => FlowChar::Whitespace,
            _ => return None,
        };
        Some(flowchar)
    }
}

#[derive(Debug)]
pub enum Token {
    Symbol(NonTerminal),
    Word(String),
    Number(NumberType),
    String(String),
}

#[derive(Debug)]
pub enum NumberType {
    Integer(String),
    Float(String),
}

pub struct Lexer {
    orig: String,
    txt: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(code: &str) -> Self {
        let txt: Vec<char> = code.chars().rev().collect();
        let pos = txt.len() - 1;
        let orig = code.to_string();

        Self { txt, pos, orig }
    }

    fn next(&mut self) -> Option<char> {
        if self.pos == 0 {
            None
        } else {
            self.pos -= 1;
            self.txt.pop()
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos == 0 {
            None
        } else {
            Some(self.txt[self.pos])
        }
    }

    fn eat(&mut self) {
        let _ = self.next();
    }

    fn expect(&mut self, exp: char) {
        match self.next() {
            Some(got) if got == exp => (),
            Some(got) => report_error(format!("Expected [{}], got: [{}]", exp, got), &self),
            None => report_error(format!("Expected: [{}], found nothing.", exp), &self),
        }
    }

    fn read_word(&mut self) -> String {
        let mut word = String::new();
        while let Some(c) = self.peek() {
            match FlowChar::try_from_char(c) {
                Some(_) => break,
                None => (),
            };

            match NonTerminal::try_from_char(c) {
                Some(_) => break,
                None => (),
            };

            word.push(self.next().unwrap());
        }

        word
    }

    fn read_string(&mut self) -> String {
        let mut string = String::new();
        while let Some(c) = self.peek() {
            match NonTerminal::try_from_char(c) {
                Some(NonTerminal::StringBorder) => break,
                _ => (),
            };

            string.push(self.next().unwrap());
        }
        string
    }

    fn read_number(&mut self) -> NumberType {
        let mut number = String::new();
        let mut is_float = false;

        while let Some(c) = self.peek() {
            match FlowChar::try_from_char(c) {
                Some(_) => break,
                None => (),
            };

            match NonTerminal::try_from_char(c) {
                Some(NonTerminal::Number) => (),
                Some(NonTerminal::Dot) => is_float = true, // TODO: check for double dots
                _ => break,
            };

            number.push(self.next().unwrap());
        }

        if is_float {
            NumberType::Float(number)
        } else {
            NumberType::Integer(number)
        }
    }

    pub fn lex(mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while let Some(c) = self.peek() {
            match FlowChar::try_from_char(c) {
                Some(FlowChar::Newline) => {
                    tokens.push(Token::Symbol(NonTerminal::Newline));
                    self.eat();
                    continue;
                }
                Some(_) => {
                    self.eat();
                    continue;
                }
                None => (),
            };

            let nonterm = match NonTerminal::try_from_char(c) {
                Some(nt) => nt,
                None => {
                    tokens.push(Token::Word(self.read_word()));
                    continue;
                }
            };

            match nonterm {
                NonTerminal::Number => tokens.push(Token::Number(self.read_number())),
                NonTerminal::StringBorder => {
                    tokens.push(Token::Symbol(NonTerminal::StringBorder));
                    self.expect(STRING_BORDER);
                    tokens.push(Token::String(self.read_string()));
                    tokens.push(Token::Symbol(NonTerminal::StringBorder));
                    self.expect(STRING_BORDER);
                }

                _ => {
                    tokens.push(Token::Symbol(nonterm));
                    self.eat();
                }
            }
        }

        tokens
    }
}

fn report_error(error: String, l: &Lexer) {
    const ERR_LEFT: usize = 15;
    const ERR_RIGHT: usize = 15;

    let mut txt = String::new();

    let pos = l.orig.len() - l.pos - 1;

    let end = if pos + ERR_RIGHT > l.orig.len() {
        l.orig.len()
    } else {
        pos + ERR_RIGHT
    };

    let start = if pos < ERR_LEFT {
        0
    } else {
        pos - ERR_LEFT
    };

    println!("pos: {}, len: {}", l.pos, l.orig.len());
    let slice = &l.orig[start..end];

    txt.push_str(&error);
    txt.push('\n');
    txt.push('\n');

    let mut marker = String::new();
    for (i, c) in slice.chars().enumerate() {
        if c == '\n' {
            break;
        }
        txt.push(c);
        if start + i < pos {
            marker.push('-');
        }
    }
    txt.push('\n');
    marker.pop();
    marker.push('^');
    txt.push_str(&marker);

    panic!("{}\n", txt);
}

fn main() {
    let test = r#"
    fn main() {
        let mut rt = Runtime::new();
        async fut {
            let io = Reactor::new_io();
            let data: u32 = io.get().await?;
            let res = data + 100;
            println!("{}", data);
        }
        rt.block_on(fut).unwrap();
    }"#;

    let lexer = Lexer::new(test);
    let tokens = lexer.lex();
    println!("{:?}", tokens);
}