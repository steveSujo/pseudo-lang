use std::{error::Error, fmt::Display, fs::write, num::ParseFloatError};

//TODO: add more types
// enum ErrorType {
//     Error,
//     Warning,
// }
// #[derive(Default)]
// struct Error {
//     line: usize,
//     place: String,
//     message: String,
// }

// impl Error {
//     fn report(&self) {
//         println!(
//             "[Line {}] Error {}: {}",
//             self.line, self.place, self.message
//         );
//     }
// }
// #[derive(Default)]
// pub struct ErrorSet {
//     error_list: Vec<Error>,
//     had_error: bool,
// }

// impl ErrorSet {
//     pub fn error(&mut self, line: usize, message: String) {
//         let error = Error {
//             line,
//             place: String::new(),
//             message,
//         };
//         error.report();
//         self.error_list.push(error);
//         self.had_error = true;
//     }

//     pub fn error_where(&mut self, line: usize, place: String, message: String) {
//         let error = Error {
//             line,
//             place,
//             message,
//         };
//         error.report();
//         self.error_list.push(error);
//         self.had_error = true;
//     }
// }
#[derive(Debug)]
pub enum Errors {
    //add line data
    UntermitedGroup,
    UntermitedString,
    UnexpectedChar,
    NonPrimaryToken,
    ParseFloatError(ParseFloatError),
}

impl Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::UntermitedGroup => write!(f, "A Group Expression should close with (\")\")"),
            Errors::UntermitedString => write!(f, "A String should close with (\") "),
            Errors::UnexpectedChar => write!(f, "Unknown Character by the Lexer for found"),
            Errors::NonPrimaryToken => write!(f, "Unknown Token for Primary Exprission"),
            Errors::ParseFloatError(..) => write!(f, "NUMER Token Parse Error"),
        }
    }
}

impl Error for Errors {}

impl From<ParseFloatError> for Errors {
    fn from(value: ParseFloatError) -> Self {
        Errors::ParseFloatError(value)
    }
}
