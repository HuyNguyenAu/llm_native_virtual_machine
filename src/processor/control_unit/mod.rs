use crate::exception::{BaseException, Exception};
use crate::processor::control_unit::decoder::Decoder;
use crate::processor::control_unit::executor::Executor;
use crate::processor::{memory::Memory, registers::Registers};

use crate::processor::control_unit::instruction::Instruction;

mod decoder;
mod executor;
mod instruction;
mod language_logic_unit;
mod utils;

pub struct ControlUnit {
    memory: Memory,
    registers: Registers,
}

impl ControlUnit {
    pub fn new() -> Self {
        ControlUnit {
            memory: Memory::new(),
            registers: Registers::new(),
        }
    }

    fn read_instruction(&self) -> Result<[[u8; 4]; 4], Exception> {
        let instruction_pointer = self.registers.get_instruction_pointer();
        let mut buffer = [[0u8; 4]; 4];

        for (i, slot) in buffer.iter_mut().enumerate() {
            *slot = match self.memory.read(instruction_pointer + i) {
                Ok(bytes) => *bytes,
                Err(exception) => {
                    return Err(Exception::ControlUnit(BaseException::new(
                        format!(
                            "Failed to read instruction at {}: no data found",
                            instruction_pointer + i
                        ),
                        Some(Box::new(exception)),
                    )));
                }
            }
        }

        Ok(buffer)
    }

    fn header_pointer(&self, index: usize, byte_code: &[[u8; 4]]) -> Result<usize, Exception> {
        let pointer_bytes = match byte_code.get(index) {
            Some(bytes) => bytes,
            None => {
                return Err(Exception::ControlUnit(BaseException::new(
                    format!(
                        "Failed to read header pointer at index {}: no data found",
                        index
                    ),
                    None,
                )));
            }
        };

        match u32::from_be_bytes(*pointer_bytes).try_into() {
            Ok(pointer) => Ok(pointer),
            Err(error) => Err(Exception::ControlUnit(BaseException::new(
                format!(
                    "Failed to read header pointer at index {}: invalid pointer value",
                    index
                ),
                Some(Box::new(error.to_string().into())),
            ))),
        }
    }

    pub fn load(&mut self, byte_code: &[[u8; 4]]) -> Result<(), Exception> {
        let instruction_section_pointer = match self.header_pointer(0, byte_code) {
            Ok(pointer) => pointer,
            Err(exception) => {
                return Err(Exception::ControlUnit(BaseException::new(
                    "Control Unit failed to load byte code: invalid instruction section pointer"
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };
        let data_section_pointer = match self.header_pointer(1, byte_code) {
            Ok(pointer) => pointer,
            Err(exception) => {
                return Err(Exception::ControlUnit(BaseException::new(
                    "Control Unit failed to load byte code: invalid data section pointer"
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        self.memory.load(byte_code);

        self.registers
            .set_instruction_pointer(instruction_section_pointer);
        self.registers.set_instruction(None);
        self.registers
            .set_data_section_pointer(data_section_pointer);

        Ok(())
    }

    pub fn fetch(&mut self) -> Result<bool, Exception> {
        if self.registers.get_instruction_pointer() >= self.registers.get_data_section_pointer() {
            return Ok(false);
        }

        let instruction_bytes = match self.read_instruction() {
            Ok(bytes) => bytes,
            Err(exception) => {
                return Err(Exception::ControlUnit(BaseException::new(
                    "Control Unit failed to fetch instruction: unable to read instruction bytes"
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        self.registers.set_instruction(Some(instruction_bytes));
        self.registers.advance_instruction_pointer(4);

        Ok(true)
    }

    pub fn decode(&self) -> Result<Instruction, Exception> {
        let bytes = match self.registers.get_instruction() {
            Some(bytes) => bytes,
            None => {
                return Err(Exception::ControlUnit(BaseException::new(
                    "Control Unit failed to decode instruction: no instruction bytes found"
                        .to_string(),
                    None,
                )));
            }
        };

        match Decoder::decode(&self.memory, &self.registers, bytes) {
            Ok(instruction) => Ok(instruction),
            Err(exception) => Err(Exception::ControlUnit(BaseException::new(
                "Control Unit failed to decode instruction.".to_string(),
                Some(Box::new(exception)),
            ))),
        }
    }

    pub fn execute(
        &mut self,
        instruction: Instruction,
        text_model: &str,
        embedding_model: &str,
        debug: bool,
    ) -> Result<(), Exception> {
        match Executor::execute(
            &mut self.memory,
            &mut self.registers,
            &instruction,
            text_model,
            embedding_model,
            debug,
        ) {
            Ok(_) => Ok(()),
            Err(exception) => Err(Exception::ControlUnit(BaseException::new(
                "Control Unit failed to execute instruction.".to_string(),
                Some(Box::new(exception)),
            ))),
        }
    }
}
