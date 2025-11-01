mod lexer;
mod parser;
mod token;
mod error;
mod cursor;

pub use parser::{parse};
pub use error::{ParseResult, ParseError};

#[cfg(test)]
mod tests {
    mod lexer;
    mod parser;
}
