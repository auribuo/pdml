use pdml_lib::parser::Parser;

#[tokio::main]
async fn main() {
    let mut parser = Parser::for_file("example.pdml".to_string());
    let pages = parser.parse().unwrap();
    dbg!(pages);
}
