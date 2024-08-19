use std::num::ParseFloatError;

use crate::input::{Cursor, LineCol, Pos, Span};

#[derive(Debug, Clone)]
pub enum Error {
    InvalidChar(LineCol, char),
    InvalidNum(LineCol, String, ParseFloatError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidChar((line, col), c) => {
                write!(f, "Invalid character '{}' at {}:{}", c, line, col)
            }
            Error::InvalidNum((line, col), s, e) => {
                write!(f, "Invalid number '{}' ({}) at {}:{}", s, e, line, col)
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Num(f64),
    Symbol(String),
    OpenPar,
    ClosePar,
    Equal,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Comma,
    NewLine,
    Space,
    Comment(String),
}

#[derive(Debug)]
pub struct Tokenizer<I> {
    cursor: Cursor<I>,
}

impl<I> Tokenizer<I> {
    pub fn new(cursor: Cursor<I>) -> Tokenizer<I> {
        Tokenizer { cursor }
    }
}

impl<I> Tokenizer<I>
where
    I: Iterator<Item = char>,
{
    fn parse_num(&mut self, pos: Pos, first: char) -> Result<f64> {
        let mut s = String::from(first);
        loop {
            let c = self.cursor.next();
            match c {
                Some(c) if c.is_ascii_digit() || c == '.' => s.push(c),
                Some(c) => {
                    self.cursor.put_back(c);
                    break;
                }
                None => break,
            }
        }
        match s.parse::<f64>() {
            Ok(n) => Ok(n),
            Err(err) => Err(Error::InvalidNum(self.cursor.line_col(pos), s, err)),
        }
    }

    fn next_token_kind(&mut self, pos: Pos) -> Result<Option<TokenKind>> {
        let c = match self.cursor.next() {
            None => return Ok(None),
            Some(c) => c,
        };

        let kind = match c {
            '(' => TokenKind::OpenPar,
            ')' => TokenKind::ClosePar,
            '=' => TokenKind::Equal,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            ',' => TokenKind::Comma,
            '\n' => TokenKind::NewLine,
            '#' => {
                let mut s = String::new();
                loop {
                    let c = self.cursor.next();
                    match c {
                        Some(c) if c != '\n' => s.push(c),
                        Some(c) => {
                            self.cursor.put_back(c);
                            break;
                        }
                        None => break,
                    }
                }
                TokenKind::Comment(s)
            }
            '0'..='9' | '.' => {
                let num = self.parse_num(pos, c)?;
                TokenKind::Num(num)
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut sym = String::new();
                sym.push(c);
                loop {
                    let c = self.cursor.next();
                    match c {
                        Some(c @ ('0'..='9' | 'a'..='z' | 'A'..='Z' | '_')) => sym.push(c),
                        Some(c) => {
                            self.cursor.put_back(c);
                            break;
                        }
                        None => break,
                    }
                }
                TokenKind::Symbol(sym)
            }
            c if c.is_ascii_whitespace() => {
                loop {
                    let c = self.cursor.next();
                    match c {
                        Some(c) if c.is_ascii_whitespace() => (),
                        Some(c) => {
                            self.cursor.put_back(c);
                            break;
                        }
                        None => break,
                    }
                }
                TokenKind::Space
            }
            _ => return Err(Error::InvalidChar(self.cursor.line_col(pos), c)),
        };
        Ok(Some(kind))
    }
}

impl<I> Iterator for Tokenizer<I>
where
    I: Iterator<Item = char>,
{
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Result<Token>> {
        let pos = self.cursor.pos();
        let kind = match self.next_token_kind(pos) {
            Ok(Some(kind)) => kind,
            Ok(None) => return None,
            Err(err) => return Some(Err(err)),
        };
        let end = self.cursor.pos();
        Some(Ok(Token {
            kind,
            span: (pos, end),
        }))
    }
}

#[test]
fn test_tokenizer() {
    let c = Cursor::new("1 + 2 # a comment".chars());
    let tokenizer = Tokenizer::new(c);
    let tokens: Vec<_> = tokenizer
        .collect::<Vec<Result<Token>>>()
        .into_iter()
        .map(|tok| tok.unwrap())
        .map(|tok| tok.kind)
        .collect();
    assert_eq!(
        tokens,
        vec![
            TokenKind::Num(1.0),
            TokenKind::Space,
            TokenKind::Plus,
            TokenKind::Space,
            TokenKind::Num(2.0),
            TokenKind::Space,
            TokenKind::Comment(" a comment".to_string()),
        ]
    );
}
