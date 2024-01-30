use thiserror::Error;
use crate::reader::{CharReader, ReaderError};

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
}

type Result<T> = std::result::Result<T, LexerError>;

impl Token {
    pub fn of_type(token_type: TokenType) -> Self {
        Self {
            token_type
        }
    }

    pub fn string_literal(data: String) -> Self {
        Self {
            token_type: TokenType::StringLiteral(data)
        }
    }

    pub fn get_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn to_inner(self) -> TokenType {
        self.token_type
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    StringLiteral(String),
    Url(String),
    Assignment,
    EOF,
    Whitespace,
    Unknown(char),
}

pub struct Lexer {
    reader: CharReader,
}

impl Lexer {
    pub fn new(reader: CharReader) -> Self {
        Self {
            reader
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        match self.reader.next_char() {
            Ok(mut next) => {
                match next {
                    '"' => {
                        let mut chars: Vec<char> = vec![];
                        next = self.reader.next_char()?;
                        while next != '"' {
                            chars.push(next);
                            next = self.reader.next_char()?;
                        }
                        Ok(Token::string_literal(String::from_iter(chars)))
                    }
                    ' ' | '\r' | '\n' | '\t' => {
                        Ok(Token::of_type(TokenType::Whitespace))
                    }
                    '<' => {
                        let mut chars: Vec<char> = vec![];
                        next = self.reader.next_char()?;
                        while next != '>' {
                            chars.push(next);
                            next = self.reader.next_char()?;
                        }
                        Ok(Token::of_type(TokenType::Url(String::from_iter(chars))))
                    }
                    '=' => {
                        Ok(Token::of_type(TokenType::Assignment))
                    }
                    any => {
                        Ok(Token::of_type(TokenType::Unknown(any)))
                    }
                }
            }
            Err(error) => {
                match error {
                    ReaderError::EOF => Ok(Token::of_type(TokenType::EOF)),
                    _ => Err(LexerError::from(error))
                }
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens: Vec<Token> = vec![];
        let mut next_token = self.next_token()?;
        while *next_token.get_type() != TokenType::EOF {
            tokens.push(next_token);
            next_token = self.next_token()?;
        }

        Ok(tokens)
    }
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("An error occurred while calling the underlying reader: {}", .0)]
    ReaderError(String)
}

impl From<ReaderError> for LexerError {
    fn from(value: ReaderError) -> Self {
        return LexerError::ReaderError(value.to_string());
    }
}