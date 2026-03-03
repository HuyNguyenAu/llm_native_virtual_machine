use crate::{
    config::Config,
    exception::{BaseException, Exception},
    processor::control_unit::ControlUnit,
};

mod control_unit;
mod memory;
mod registers;

pub struct Processor {
    config: Config,
    control_unit: ControlUnit,
}

impl Processor {
    pub fn new(config: Config) -> Self {
        Processor {
            config,
            control_unit: ControlUnit::new(),
        }
    }

    pub fn load(&mut self, data: &[u8]) -> Result<(), Exception> {
        if !data.len().is_multiple_of(4) {
            return Err(Exception::Processor(BaseException::new(
                format!(
                    "Processor failed to load byte code. Invalid byte code length: {}. Byte code must be a multiple of 4 bytes.",
                    data.len()
                ),
                None,
            )));
        }

        let chunks = data.chunks(4);
        let mut byte_code: Vec<[u8; 4]> = Vec::new();

        for chunk in chunks {
            match chunk.try_into() {
                Ok(bytes) => byte_code.push(bytes),
                Err(error) => {
                    return Err(Exception::Processor(BaseException::new(
                        "Processor failed to load byte code. Byte code chunks must be exactly 4 bytes.".to_string(),
                        Some(Box::new(format!("{:#?}", error).into())),
                    )));
                }
            }
        }

        match self.control_unit.load(&byte_code) {
            Ok(_) => Ok(()),
            Err(exception) => Err(Exception::Processor(BaseException::new(
                "Processor failed to load byte code into control unit.".to_string(),
                Some(Box::new(exception)),
            ))),
        }
    }

    pub fn run(&mut self) -> Result<(), Exception> {
        loop {
            match self.control_unit.fetch() {
                Ok(instruction_fetched) => {
                    if !instruction_fetched {
                        return Ok(());
                    }
                }
                Err(exception) => {
                    return Err(Exception::Processor(BaseException::new(
                        "Processor failed to fetch instruction.".to_string(),
                        Some(Box::new(exception)),
                    )));
                }
            }

            let instruction = match self.control_unit.decode() {
                Ok(instruction) => instruction,
                Err(exception) => {
                    return Err(Exception::Processor(BaseException::new(
                        "Processor failed to decode instruction.".to_string(),
                        Some(Box::new(exception)),
                    )));
                }
            };

            match self.control_unit.execute(
                instruction,
                &self.config.text_model,
                &self.config.embedding_model,
                self.config.debug_run,
            ) {
                Ok(_) => (),
                Err(exception) => {
                    return Err(Exception::Processor(BaseException::new(
                        "Processor failed to execute instruction.".to_string(),
                        Some(Box::new(exception)),
                    )));
                }
            }
        }
    }
}
