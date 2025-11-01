mod lexer;
mod parser;
mod token;
mod error;
mod cursor;

pub use error::{ParseError, ParseResult};
pub use parser::parse;

#[cfg(test)]
mod tests {
    mod lexer;
    mod parser;
}
