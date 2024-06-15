use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use thiserror::Error;

pub struct CharReader {
    reader: BufReader<File>,
}

type Result<T> = std::result::Result<T, ReaderError>;

impl CharReader {}

impl CharReader {
    pub fn from_file(path: &String) -> Result<Self> {
        let file = File::open(&path)?;
        Ok(Self {
            reader: BufReader::new(file),
        })
    }

    pub fn reader(&self) -> &BufReader<File> {
        &self.reader
    }

    pub fn next_char(&mut self) -> Result<char> {
        let mut buf: [u8; 1] = [0];
        if self.reader.read(&mut buf)? == 0 {
            return Err(ReaderError::EOF);
        }
        Ok(char::from(buf[0]))
    }

    pub fn next_chars(&mut self, amt: usize) -> Result<Vec<char>> {
        let mut buf: Vec<u8> = vec![0; amt];
        let read_bytes = self.reader.read(&mut buf)?;
        if read_bytes <= 0 {
            return Err(ReaderError::EOF);
        }
        if amt != read_bytes {
            return Err(ReaderError::ReadError(format!(
                "Read wrong amount of bytes. Expected {}, read {}",
                amt, read_bytes
            )));
        }
        Ok(buf.iter().map(|u| char::from(*u)).collect())
    }

    pub fn peek(&mut self) -> Result<char> {
        return match self.reader.fill_buf() {
            Ok(buf) => {
                return if buf.len() == 0 {
                    Err(ReaderError::EOF)
                } else {
                    Ok(char::from(buf[0]))
                }
            }
            Err(_) => Err(ReaderError::ReadError("Could not peek".to_string())),
        };
    }

    pub fn peek_many(&mut self, amt: usize) -> Result<Vec<char>> {
        return match self.reader.fill_buf() {
            Ok(buf) => {
                return if buf.len() == 0 {
                    Err(ReaderError::EOF)
                } else {
                    Ok(buf.iter().take(amt).map(|u| char::from(*u)).collect())
                }
            }
            Err(_) => Err(ReaderError::ReadError("Could not peek".to_string())),
        };
    }

    pub fn advance(&mut self, amt: usize) {
        self.reader.consume(amt)
    }
}

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("An unexpected io error occurred: {}", .0)]
    IoError(String),

    #[error("Error while reading from buffer: {}", .0)]
    ReadError(String),

    #[error("Reader reached eof")]
    EOF,
}

impl From<std::io::Error> for ReaderError {
    fn from(value: std::io::Error) -> Self {
        ReaderError::IoError(value.to_string())
    }
}
