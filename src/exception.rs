use std::{error::Error, fmt, num::ParseIntError};

#[derive(Debug)]
pub struct BaseException {
    pub location: String,
    pub message: String,
    pub inner_exception: Option<Box<Exception>>,
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

    #[track_caller]
    pub fn caused_by(message: impl Into<String>, cause: impl Into<Exception>) -> Self {
        BaseException::new(message.into(), Some(Box::new(cause.into())))
    }
}

impl From<Exception> for BaseException {
    fn from(exception: Exception) -> Self {
        let e = exception.into_inner();
        BaseException::new(e.message, e.inner_exception)
    }
}

#[derive(Debug)]
pub enum Exception {
    BaseException(BaseException),
    Program(BaseException),
    Assembler(BaseException),
    OpenAIChatCompletion(BaseException),
    OpenAIEmbeddings(BaseException),
    LanguageLogic(BaseException),
    ControlUnit(BaseException),
    Decoder(BaseException),
    Executor(BaseException),
    Processor(BaseException),
    Memory(BaseException),
    Register(BaseException),
    Config(BaseException),
}

impl Exception {
    fn inner(&self) -> &BaseException {
        match self {
            Self::BaseException(e)
            | Self::Program(e)
            | Self::Assembler(e)
            | Self::OpenAIChatCompletion(e)
            | Self::OpenAIEmbeddings(e)
            | Self::LanguageLogic(e)
            | Self::ControlUnit(e)
            | Self::Decoder(e)
            | Self::Executor(e)
            | Self::Processor(e)
            | Self::Memory(e)
            | Self::Register(e)
            | Self::Config(e) => e,
        }
    }

    fn into_inner(self) -> BaseException {
        match self {
            Self::BaseException(e)
            | Self::Program(e)
            | Self::Assembler(e)
            | Self::OpenAIChatCompletion(e)
            | Self::OpenAIEmbeddings(e)
            | Self::LanguageLogic(e)
            | Self::ControlUnit(e)
            | Self::Decoder(e)
            | Self::Executor(e)
            | Self::Processor(e)
            | Self::Memory(e)
            | Self::Register(e)
            | Self::Config(e) => e,
        }
    }
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base = self.inner();

        write!(f, "[{}] {}", base.location, base.message)?;

        if let Some(inner) = &base.inner_exception {
            write!(f, "\n  Caused by: {}", inner)?;
        }

        Ok(())
    }
}
