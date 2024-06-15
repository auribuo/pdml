use pdml_lib::{parser::Parser, scrape::{ParserExt, ScrapeBindable}};

#[tokio::main]
async fn main() {
    let mut parser = Parser::for_file("example.pdml".to_string());
    let _ = parser.scrape::<Res>().await.unwrap();
}

struct Res {

}

impl ScrapeBindable for Res {
    fn bind(page: &pdml_lib::scrape::ScrapedPage) -> Self {
        dbg!(page);
        Self{}
    }
}