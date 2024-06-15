use crate::reader::{CharReader, ReaderError};
use std::fmt::Debug;
use thiserror::Error;

const VALID_IDEN_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
}

impl PartialEq<TokenType> for Token {
    fn eq(&self, other: &TokenType) -> bool {
        match self.get_type() {
            TokenType::Literal(t, _) => match other {
                TokenType::Literal(ot, _) => &t == ot,
                _ => false,
            },
            TokenType::Assignment => match other {
                TokenType::Assignment => true,
                _ => false,
            },
            TokenType::Paren(p) => match other {
                TokenType::Paren(op) => &p == op,
                _ => false,
            },
            TokenType::EOF => match other {
                TokenType::EOF => true,
                _ => false,
            },
            TokenType::Whitespace => match other {
                TokenType::Whitespace => true,
                _ => false,
            },
            TokenType::Page => match other {
                TokenType::Page => true,
                _ => false,
            },
            TokenType::Unknown(_) => match other {
                TokenType::Unknown(_) => true,
                _ => false,
            },
            TokenType::Selector(_, _) => match other {
                TokenType::Selector(_, _) => true,
                _ => false,
            },
        }
    }
}

type Result<T> = std::result::Result<T, LexerError>;

impl Token {
    pub fn of_type(token_type: TokenType) -> Self {
        Self { token_type }
    }

    pub fn get_type(&self) -> TokenType {
        self.token_type.clone()
    }

    pub fn to_inner(self) -> TokenType {
        self.token_type
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Literal(LiteralType, String),
    Assignment,
    Paren(ParenType),
    EOF,
    Whitespace,
    Page,
    Unknown(char),
    Selector(String, Quantifier),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Quantifier {
    Single,
    Many,
    Fixed(usize),
    Any,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralType {
    String,
    Url,
    Identifier,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParenType {
    BlockOpen,
    BlockClose,
}

pub struct Lexer {
    reader: CharReader,
}

impl Lexer {
    pub fn new(reader: CharReader) -> Self {
        Self { reader }
    }

    fn parse_literal_raw(&mut self, end_delimiter: char) -> Result<String> {
        let mut chars: Vec<char> = vec![];
        let mut next = self.reader.next_char()?;
        while next != end_delimiter {
            chars.push(next);
            next = self.reader.next_char()?;
        }
        return Ok(String::from_iter(chars));
    }

    fn parse_literal(
        &mut self,
        literal_type: LiteralType,
        (start_delimiter, end_delimiter): (char, char),
    ) -> Result<Token> {
        let start_char = self.reader.next_char()?;
        if start_char != start_delimiter {
            return Err(LexerError::UnmatchedTokenError(TokenType::Unknown(
                start_char,
            )));
        }

        let mut chars: Vec<char> = vec![];
        let mut next = self.reader.next_char()?;
        while next != end_delimiter {
            chars.push(next);
            next = self.reader.next_char()?;
        }
        Ok(Token::of_type(TokenType::Literal(
            literal_type,
            String::from_iter(chars),
        )))
    }

    fn parse_identifier(&mut self) -> Result<Token> {
        let start_char = self.reader.next_char()?;
        if start_char != '$' {
            return Err(LexerError::UnmatchedTokenError(TokenType::Literal(
                LiteralType::Identifier,
                "".to_string(),
            )));
        }

        let mut chars: Vec<char> = vec![];
        let mut next = self.reader.next_char()?;
        while VALID_IDEN_CHARS.chars().any(|c| c == next) {
            chars.push(next);
            next = self.reader.next_char()?;
        }
        Ok(Token::of_type(TokenType::Literal(
            LiteralType::Identifier,
            String::from_iter(chars),
        )))
    }

    fn parse_page(&mut self) -> Result<Token> {
        let buf = self.reader.peek_many(4).unwrap();
        if buf == ['p', 'a', 'g', 'e'] {
            self.reader.advance(4);
            Ok(Token::of_type(TokenType::Page))
        } else {
            Err(LexerError::ReaderError("".to_string()))
        }
    }

    fn parse_quantifier(str: &str) -> Result<Quantifier> {
        match str {
            "" => Ok(Quantifier::Many),
            q => match q.parse::<u32>() {
                Ok(amt) => Ok(Quantifier::Fixed(amt as usize)),
                Err(err) => Err(LexerError::InvalidQuantifier(err.to_string())),
            },
        }
    }

    fn parse_selector(&mut self) -> Result<Token> {
        let selector = self.parse_literal_raw(';')?;
        let selector_string;
        let quantifier;

        if selector.contains('*') {
            let spl: Vec<&str> = selector.split('*').collect();
            selector_string = spl[0];
            let mut quantifier_str = "";
            if spl.len() > 1 {
                quantifier_str = spl[1];
            }
            match Self::parse_quantifier(quantifier_str) {
                Ok(q) => quantifier = q,
                Err(err) => {
                    return Err(LexerError::InvalidQuantifier(format!(
                        "{} ({})",
                        quantifier_str.to_string(),
                        err.to_string()
                    )))
                }
            }
        } else {
            selector_string = selector.as_str();
            quantifier = Quantifier::Single
        }

        Ok(Token::of_type(TokenType::Selector(
            selector_string.to_string(),
            quantifier,
        )))
    }

    pub fn next_non_whitespace(&mut self) -> Result<Token> {
        let mut token = self.next_token()?;
        while token.token_type == TokenType::Whitespace {
            token = self.next_token()?;
        }
        Ok(token)
    }

    pub fn next_token(&mut self) -> Result<Token> {
        match self.reader.peek() {
            Ok(next) => match next {
                '"' => self.parse_literal(LiteralType::String, ('"', '"')),
                ' ' | '\r' | '\n' | '\t' => {
                    self.reader.advance(1);
                    Ok(Token::of_type(TokenType::Whitespace))
                }
                '<' => self.parse_literal(LiteralType::Url, ('<', '>')),
                '=' => {
                    self.reader.advance(1);
                    Ok(Token::of_type(TokenType::Assignment))
                }
                'p' => {
                    let page_parse_result = self.parse_page();
                    match page_parse_result {
                        Ok(res) => Ok(res),
                        Err(error) => match error {
                            LexerError::UnmatchedTokenError(_) => self.parse_selector(),
                            err => Err(err),
                        },
                    }
                }
                '$' => self.parse_identifier(),
                '{' => {
                    self.reader.advance(1);
                    Ok(Token::of_type(TokenType::Paren(ParenType::BlockOpen)))
                }
                '}' => {
                    self.reader.advance(1);
                    Ok(Token::of_type(TokenType::Paren(ParenType::BlockClose)))
                }
                any => match self.parse_selector() {
                    Ok(res) => Ok(res),
                    Err(err) => match err {
                        LexerError::UnmatchedTokenError(_) => {
                            self.reader.advance(1);
                            Ok(Token::of_type(TokenType::Unknown(any)))
                        }
                        err => Err(err),
                    },
                },
            },
            Err(error) => match error {
                ReaderError::EOF => Ok(Token::of_type(TokenType::EOF)),
                _ => Err(LexerError::from(error)),
            },
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens: Vec<Token> = vec![];
        let mut next_token = self.next_token()?;
        while next_token.get_type() != TokenType::EOF {
            tokens.push(next_token);
            next_token = self.next_token()?;
        }

        Ok(tokens)
    }
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("An error occurred while calling the underlying reader: {}", .0)]
    ReaderError(String),

    #[error("Unmatched token type: {:?}", .0)]
    UnmatchedTokenError(TokenType),

    #[error("An error occurred while parsing. Unexpected char: {}", .0)]
    UnexpectedChar(char),

    #[error("Invalid quantifier encountered: {}", .0)]
    InvalidQuantifier(String),
}

impl From<ReaderError> for LexerError {
    fn from(value: ReaderError) -> Self {
        return LexerError::ReaderError(value.to_string());
    }
}
