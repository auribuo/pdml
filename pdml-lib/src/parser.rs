use crate::lexer;
use crate::lexer::{Lexer, LexerError, LiteralType, ParenType, Token, TokenType};
use crate::parser::Error::{UnexpectedTokenError, UnexpectedTokenValidManyError};
use crate::reader::{CharReader, ReaderError};
#[cfg(feature = "scrape")]
use crate::Error::ScraperError;
use std::string::ToString;
use thiserror::Error;

const ANY: &'static str = "any";

macro_rules! any_string {
    () => {
        ANY.to_string()
    };
}

pub struct Parser {
    file: String,
}

type Result<T> = std::result::Result<T, Error>;

impl Parser {
    pub fn for_file(file: String) -> Self {
        Self { file }
    }

    pub fn parse(&mut self) -> Result<Vec<Page>> {
        let reader = CharReader::from_file(&self.file)?;
        let lexer = Lexer::new(reader);
        PageParser { lexer }.parse_pages()
    }
}

struct PageParser {
    lexer: Lexer,
}

fn expect(token_type: TokenType, got: &Token) -> Result<()> {
    if *got == token_type {
        Ok(())
    } else {
        Err(UnexpectedTokenError(token_type, got.get_type()))
    }
}

impl PageParser {
    pub fn parse_pages(mut self) -> Result<Vec<Page>> {
        let mut token = self.lexer.next_non_whitespace()?;
        let mut pages: Vec<Page> = vec![];
        while token.get_type() != TokenType::EOF {
            let mut partial_page = PartialPage::default();
            expect(TokenType::Page, &token)?;
            token = self.lexer.next_non_whitespace()?;
            expect(TokenType::Literal(LiteralType::Url, any_string!()), &token)?;
            match token.get_type() {
                TokenType::Literal(LiteralType::Url, str) => {
                    partial_page.url = Some(str);
                }
                _ => panic!("Unexpected behaviour"),
            }
            token = self.lexer.next_non_whitespace()?;
            match token.get_type() {
                TokenType::Assignment => {
                    token = self.lexer.next_non_whitespace()?;
                    expect(
                        TokenType::Literal(LiteralType::String, any_string!()),
                        &token,
                    )?;
                    match token.get_type() {
                        TokenType::Literal(LiteralType::String, str) => {
                            partial_page.name = Some(str);
                        }
                        _ => panic!("Unexpected behaviour"),
                    }
                    token = self.lexer.next_non_whitespace()?;
                    expect(TokenType::Paren(ParenType::BlockOpen), &token)?;
                    pages.push(self.parse_page(partial_page)?);
                    token = self.lexer.next_non_whitespace()?;
                }
                TokenType::Paren(ParenType::BlockOpen) => {
                    pages.push(self.parse_page(partial_page)?);
                }
                t => {
                    return Err(UnexpectedTokenValidManyError(
                        vec![
                            TokenType::Assignment,
                            TokenType::Paren(ParenType::BlockOpen),
                        ],
                        t,
                    ));
                }
            }
        }
        Ok(pages)
    }

    fn parse_page(&mut self, mut partial_page: PartialPage) -> Result<Page> {
        let token = self.lexer.next_non_whitespace()?;
        match token.get_type() {
            TokenType::Paren(ParenType::BlockClose) => Ok(partial_page.into()),
            TokenType::Literal(LiteralType::Identifier, _) | TokenType::Selector(_, _) => {
                partial_page.elements = Some(self.parse_block(token.clone())?);
                Ok(partial_page.into())
            }
            t => Err(UnexpectedTokenValidManyError(
                vec![
                    TokenType::Paren(ParenType::BlockClose),
                    TokenType::Literal(LiteralType::Identifier, any_string!()),
                    TokenType::Selector(any_string!(), Quantifier::Any),
                ],
                t,
            )),
        }
    }

    fn parse_block(&mut self, initial_token: Token) -> Result<Vec<Element>> {
        let mut token = initial_token;
        let mut elements: Vec<Element> = vec![];
        while token.get_type() != TokenType::Paren(ParenType::BlockClose) {
            let mut elem = PartialElement::default();
            match token.get_type() {
                TokenType::Literal(LiteralType::Identifier, iden) => {
                    elem.identifier = Some(iden);
                    token = self.lexer.next_non_whitespace()?;
                    expect(TokenType::Assignment, &token)?;
                    token = self.lexer.next_non_whitespace()?;
                    expect(TokenType::Selector(any_string!(), Quantifier::Any), &token)?;
                    match token.get_type() {
                        TokenType::Selector(sel_str, quant) => {
                            elem.selector = Some(sel_str);
                            elem.quantifier = Some(quant.into());
                        }
                        _ => panic!("Unexpected behaviour"),
                    }
                }
                TokenType::Selector(selector, quantifier) => {
                    elem.selector = Some(selector);
                    elem.quantifier = Some(quantifier.into());
                }
                t => {
                    return Err(UnexpectedTokenValidManyError(
                        vec![
                            TokenType::Literal(LiteralType::Identifier, any_string!()),
                            TokenType::Selector(any_string!(), Quantifier::Any),
                        ],
                        t,
                    ));
                }
            }
            token = self.lexer.next_non_whitespace()?;
            if token.get_type() == TokenType::Paren(ParenType::BlockOpen) {
                token = self.lexer.next_non_whitespace()?;
                elem.children = Some(self.parse_block(token.clone())?); // TODO performance
                token = self.lexer.next_non_whitespace()?;
            }
            elements.push(elem.into());
        }
        Ok(elements)
    }
}

#[partial]
#[derive(Debug)]
pub struct Page {
    url: String,
    name: Option<String>,
    elements: Vec<Element>,
}
impl Page {
    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn elements(&self) -> &Vec<Element> {
        &self.elements
    }
}

#[partial]
#[derive(Debug)]
pub struct Element {
    identifier: Option<String>,
    selector: String,
    quantifier: Quantifier,
    children: Option<Vec<Element>>,
}

impl Element {
    pub fn identifier(&self) -> &Option<String> {
        &self.identifier
    }
    pub fn selector(&self) -> &str {
        &self.selector
    }
    pub fn quantifier(&self) -> &Quantifier {
        &self.quantifier
    }
    pub fn children(&self) -> &Option<Vec<Element>> {
        &self.children
    }
}

type Quantifier = lexer::Quantifier;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error while reading the source: {}", .0)]
    ReaderError(String),

    #[error("Error while processing the source: {}", .0)]
    LexerError(String),

    #[error("Unexpected token: expected {:?}, got {:?}", .0, .1)]
    UnexpectedTokenError(TokenType, TokenType),

    #[error("Unexpected token: expected either of the following {:?}, got {:?}", .0, .1)]
    UnexpectedTokenValidManyError(Vec<TokenType>, TokenType),

    #[cfg(feature = "scrape")]
    #[error("Error while scraping the site: {}", .0)]
    ScraperError(String),
}

impl From<ReaderError> for Error {
    fn from(value: ReaderError) -> Self {
        Error::ReaderError(value.to_string())
    }
}

impl From<LexerError> for Error {
    fn from(value: LexerError) -> Self {
        Error::LexerError(value.to_string())
    }
}

#[cfg(feature = "scrape")]
impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        ScraperError(value.to_string())
    }
}
