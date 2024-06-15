use crate::{Error, Parser};
use async_trait::async_trait;

type Result<T> = std::result::Result<T, Error>;

pub trait ScrapeBindable {
    fn bind(page: &ScrapedPage) -> Self;
}

#[derive(Debug)]
pub struct ScrapedPage {
    url: String,
    name: Option<String>,
    elements: Vec<ScrapedElement>,
}

#[derive(Debug)]
pub struct ScrapedElement {
    name: String,
    values: Vec<String>,
}

#[async_trait]
pub trait ParserExt {
    async fn scrape<T>(&mut self) -> Result<Vec<T>>
    where
        T: ScrapeBindable;
}

#[async_trait]
impl ParserExt for Parser {
    async fn scrape<T>(&mut self) -> Result<Vec<T>>
    where
        T: ScrapeBindable,
    {
        todo!("Not implemented!");
    }
}

