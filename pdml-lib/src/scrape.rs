use std::rc::Rc;

use crate::lexer::Quantifier;
use crate::parser::{Element, Page};
use crate::{Error, Parser};
use async_trait::async_trait;
use soup::prelude::{Node, Soup};
use soup::{NodeExt, QueryBuilderExt};

type Result<T> = std::result::Result<T, Error>;

pub trait ScrapeBindable {
    fn bind(page: &ScrapedPage) -> Self;
}

enum Selector {
    Tag(String),
    Attr((String, String)),
    Both(String, (String, String)),
}

impl Selector {
    pub fn parse(selector: String) -> Result<Self> {
        return if selector.contains(".") {
            Ok(Self::extract_split(
                selector,
                ".".to_string(),
                "class".to_string(),
            ))
        } else if selector.contains("#") {
            Ok(Self::extract_split(
                selector,
                "#".to_string(),
                "id".to_string(),
            ))
        } else if selector.contains("[") {
            let spl: Vec<&str> = selector.split("[").collect();
            return if spl.len() != 2 {
                Err(Error::ScraperError(format!(
                    "Malformed selector: {}",
                    selector
                )))
            } else {
                let tag = spl[0].to_string();
                match spl[1].strip_suffix("]") {
                    Some(attr) => Ok(Selector::Both(tag, Self::parse_attr(attr.to_string())?)),
                    None => Err(Error::ScraperError(format!(
                        "Malformed selector: {}",
                        selector
                    ))),
                }
            };
        } else {
            Ok(Selector::Tag(selector))
        };
    }

    fn extract_split(selector: String, split_str: String, attr: String) -> Self {
        let dot_loc = selector.find(&split_str).unwrap();
        match dot_loc {
            0 => Selector::Attr((attr, selector.strip_prefix(&split_str).unwrap().to_string())),
            loc => {
                if loc == selector.len() - 1 {
                    return Selector::Tag(selector.strip_suffix(&split_str).unwrap().to_string());
                }
                let spl: Vec<&str> = selector.split(&split_str).collect();
                Selector::Both(spl[0].to_string(), (attr.to_string(), spl[1].to_string()))
            }
        }
    }

    fn parse_attr(attr: String) -> Result<(String, String)> {
        let attr_spl: Vec<&str> = attr.split("=").collect();
        if attr_spl.len() != 2 {
            return Err(Error::ScraperError(format!(
                "Malformed attribute list: {}",
                attr
            )));
        }
        Ok((attr_spl[0].to_string(), attr_spl[1].to_string()))
    }
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
        let parsed_data = self.parse()?;
        let mut scraped_pages: Vec<ScrapedPage> = vec![];
        for page in parsed_data {
            scraped_pages.push(scrape_page(page).await?);
        }
        Ok(scraped_pages.iter().map(|p| T::bind(p)).collect())
    }
}

async fn scrape_page(page: Page) -> Result<ScrapedPage> {
    let text = reqwest::get(page.url()).await?.text().await?;
    let soup = Soup::new(text.as_str());
    let mut res = ScrapedPage {
        url: page.url().clone(),
        name: page.name().map(|o| o.clone()),
        elements: vec![],
    };
    let mut scraped: Vec<ScrapedElement> = vec![];
    for element in page.elements() {
        if let Some(id) = element.identifier() {
            scraped.push(ScrapedElement {
                name: id.clone(),
                values: get_element_data(element, soup.tag("body").find().expect("No body?"))?,
            })
        }
    }
    res.elements = scraped;
    Ok(res)
}

fn get_element_data(element: &Element, node: Rc<Node>) -> Result<Vec<String>> {
    let selector = element.selector();
    match Selector::parse(selector.to_string())? {
        Selector::Tag(tag) => match element.quantifier() {
            Quantifier::Single => Ok(node.tag(tag).find_all().take(1).map(|n| n.text()).collect()),
            Quantifier::Fixed(amt) => Ok(node
                .tag(tag)
                .find_all()
                .take(*amt)
                .map(|n| n.text())
                .collect()),
            _ => Ok(node.tag(tag).find_all().map(|n| n.text()).collect()),
        },
        Selector::Attr(attrs) => match element.quantifier() {
            Quantifier::Single => Ok(node
                .attr(attrs.0, attrs.1)
                .find_all()
                .take(1)
                .map(|n| n.text())
                .collect()),
            Quantifier::Fixed(amt) => Ok(node
                .attr(attrs.0, attrs.1)
                .find_all()
                .take(*amt)
                .map(|n| n.text())
                .collect()),
            _ => Ok(node
                .attr(attrs.0, attrs.1)
                .find_all()
                .map(|n| n.text())
                .collect()),
        },
        Selector::Both(tag, attrs) => match element.quantifier() {
            Quantifier::Single => Ok(node
                .tag(tag)
                .attr(attrs.0, attrs.1)
                .find_all()
                .take(1)
                .map(|n| n.text())
                .collect()),
            Quantifier::Fixed(amt) => Ok(node
                .tag(tag)
                .attr(attrs.0, attrs.1)
                .find_all()
                .take(*amt)
                .map(|n| n.text())
                .collect()),
            _ => Ok(node
                .tag(tag)
                .attr(attrs.0, attrs.1)
                .find_all()
                .map(|n| n.text())
                .collect()),
        },
    }
}