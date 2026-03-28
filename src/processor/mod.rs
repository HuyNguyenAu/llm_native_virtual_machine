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
                    "Invalid byte code length: {}. Must be a multiple of 4 bytes.",
                    data.len()
                ),
                None,
            )));
        }

        let byte_code: Vec<[u8; 4]> = data
            .chunks(4)
            .map(|chunk| {
                chunk.try_into().map_err(|e| {
                    Exception::Processor(BaseException::new(
                        "Byte code chunks must be exactly 4 bytes.".to_string(),
                        Some(Box::new(format!("{:#?}", e).into())),
                    ))
                })
            })
            .collect::<Result<_, _>>()?;

        self.control_unit.load(&byte_code).map_err(|e| {
            Exception::Processor(BaseException::new(
                "Failed to load byte code into control unit.".to_string(),
                Some(Box::new(e)),
            ))
        })
    }

    pub fn run(&mut self) -> Result<(), Exception> {
        loop {
            if !self.control_unit.fetch().map_err(|e| {
                Exception::Processor(BaseException::new(
                    "Failed to fetch instruction.".to_string(),
                    Some(Box::new(e)),
                ))
            })? {
                return Ok(());
            }

            let instruction = self.control_unit.decode().map_err(|e| {
                Exception::Processor(BaseException::new(
                    "Failed to decode instruction.".to_string(),
                    Some(Box::new(e)),
                ))
            })?;

            self.control_unit
                .execute(
                    instruction,
                    &self.config.text_model,
                    &self.config.embedding_model,
                    &self.config.text_model_overrides,
                    self.config.debug_run,
                    self.config.debug_chat,
                )
                .map_err(|e| {
                    Exception::Processor(BaseException::new(
                        "Failed to execute instruction.".to_string(),
                        Some(Box::new(e)),
                    ))
                })?;
        }
    }
}
