use std::{
    error::Error,
    fmt::{self},
};

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
}

impl From<Exception> for BaseException {
    fn from(exception: Exception) -> Self {
        match exception {
            Exception::BaseException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            Exception::ProgramException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // OpenAI client exceptions.
            Exception::OpenAIChatCompletionException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            Exception::OpenAIEmbeddingsException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // Language logic unit exceptions.
            Exception::LanguageLogicException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // Control unit exceptions.
            Exception::ControlUnitException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            Exception::DecoderException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            Exception::ExecutorException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // Processor exceptions.
            Exception::ProcessorException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // Memory exceptions.
            Exception::MemoryException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
            // Register exceptions.
            Exception::RegisterException(exception) => {
                BaseException::new(exception.message, exception.inner_exception)
            }
        }
    }
}

#[derive(Debug)]
pub enum Exception {
    BaseException(BaseException),
    ProgramException(BaseException),
    // OpenAI client exceptions.
    OpenAIChatCompletionException(BaseException),
    OpenAIEmbeddingsException(BaseException),
    // Language logic unit exceptions.
    LanguageLogicException(BaseException),
    // Control unit exceptions.
    ControlUnitException(BaseException),
    DecoderException(BaseException),
    ExecutorException(BaseException),
    // Processor exceptions.
    ProcessorException(BaseException),
    // Memory exceptions.
    MemoryException(BaseException),
    // Register exceptions.
    RegisterException(BaseException),
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

impl fmt::Display for Exception {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            _ => write!(formatter, "{:#?}", self),
        }
    }
}
