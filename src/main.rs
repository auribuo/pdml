fn main() {
    let reader = pdml_lib::reader::CharReader::from_file("example.pdml".to_string()).unwrap();
    let mut lexer = pdml_lib::lexer::Lexer::new(reader);
    lexer.tokenize().unwrap().iter().for_each(|t|println!("{:?}", t))
}
