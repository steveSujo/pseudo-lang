//TODO: add more types
// enum ErrorType {
//     Error,
//     Warning,
// }
#[derive(Default)]
struct Error {
    line: usize,
    place: String,
    message: String,
}

impl Error {
    fn report(&self) {
        println!(
            "[Line {}] Error {}: {}",
            self.line, self.place, self.message
        );
    }
}
#[derive(Default)]
pub struct ErrorSet {
    error_list: Vec<Error>,
    had_error: bool,
}

impl ErrorSet {
    pub fn error(&mut self, line: usize, message: String) {
        let error = Error {
            line,
            place: String::new(),
            message,
        };
        error.report();
        self.error_list.push(error);
        self.had_error = true;
    }

    pub fn error_where(&mut self, line: usize, place: String, message: String) {
        let error = Error {
            line,
            place,
            message,
        };
        error.report();
        self.error_list.push(error);
        self.had_error = true;
    }
}
