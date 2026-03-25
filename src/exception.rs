use std::{
    error::Error,
    fmt::{self},
    num::ParseIntError,
};

#[derive(Debug)]
pub struct BaseException {
    pub location: String,
    pub message: String,
    pub inner_exception: Option<Box<Exception>>,
}

impl std::ops::Deref for BaseException {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.location
    }
}

impl BaseException {
    #[track_caller]
    pub fn new(message: String, inner_exception: Option<Box<Exception>>) -> Self {
        let caller = std::panic::Location::caller();

        BaseException {
            message,
            inner_exception,
            location: format!("{}:{}:{}", caller.file(), caller.line(), caller.column()),
        }
    }
}

impl From<Exception> for BaseException {
    fn from(exception: Exception) -> Self {
        match exception {
            Exception::BaseException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            Exception::Program(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            Exception::Assembler(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // OpenAI client exceptions.
            Exception::OpenAIChatCompletion(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            Exception::OpenAIEmbeddings(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // Language logic unit exceptions.
            Exception::LanguageLogic(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // Control unit exceptions.
            Exception::ControlUnit(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            Exception::Decoder(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            Exception::Executor(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // Processor exceptions.
            Exception::Processor(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // Memory exceptions.
            Exception::Memory(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // Register exceptions.
            Exception::Register(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
        }
    }
}

#[derive(Debug)]
pub enum Exception {
    BaseException(BaseException),
    Program(BaseException),
    Assembler(BaseException),
    // OpenAI client exceptions.
    OpenAIChatCompletion(BaseException),
    OpenAIEmbeddings(BaseException),
    // Language logic unit exceptions.
    LanguageLogic(BaseException),
    // Control unit exceptions.
    ControlUnit(BaseException),
    Decoder(BaseException),
    Executor(BaseException),
    // Processor exceptions.
    Processor(BaseException),
    // Memory exceptions.
    Memory(BaseException),
    // Register exceptions.
    Register(BaseException),
}

impl From<std::io::Error> for Exception {
    fn from(error: std::io::Error) -> Self {
        Exception::BaseException(BaseException::new(format!("{}", error), None))
    }
}

impl From<&dyn Error> for Exception {
    fn from(error: &dyn Error) -> Self {
        Exception::BaseException(BaseException::new(format!("{}", error), None))
    }
}

impl From<minreq::Error> for Exception {
    fn from(error: minreq::Error) -> Self {
        Exception::BaseException(BaseException::new(format!("{}", error), None))
    }
}

impl From<miniserde::Error> for Exception {
    fn from(error: miniserde::Error) -> Self {
        Exception::BaseException(BaseException::new(format!("{}", error), None))
    }
}

impl From<String> for Exception {
    fn from(message: String) -> Self {
        Exception::BaseException(BaseException::new(message, None))
    }
}

impl From<ParseIntError> for Exception {
    fn from(error: ParseIntError) -> Self {
        Exception::BaseException(BaseException::new(format!("{}", error), None))
    }
}

impl fmt::Display for Exception {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:#?}", self)
    }
}
