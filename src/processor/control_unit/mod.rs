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
        let ip = self.registers.get_instruction_pointer();
        let mut buffer = [[0u8; 4]; 4];

        for (i, slot) in buffer.iter_mut().enumerate() {
            *slot = *self.memory.read(ip + i).map_err(|e| {
                Exception::ControlUnit(BaseException::new(
                    format!("Failed to read instruction at {}", ip + i),
                    Some(Box::new(e)),
                ))
            })?;
        }

        Ok(buffer)
    }

    fn header_pointer(&self, index: usize, byte_code: &[[u8; 4]]) -> Result<usize, Exception> {
        let pointer_bytes = byte_code.get(index).ok_or_else(|| {
            Exception::ControlUnit(BaseException::new(
                format!("Header pointer at index {} not found", index),
                None,
            ))
        })?;

        u32::from_be_bytes(*pointer_bytes).try_into().map_err(|e| {
            Exception::ControlUnit(BaseException::new(
                format!("Invalid header pointer value at index {}", index),
                Some(Box::new(format!("{}", e).into())),
            ))
        })
    }

    pub fn load(&mut self, byte_code: &[[u8; 4]]) -> Result<(), Exception> {
        let instruction_section_pointer = self.header_pointer(0, byte_code).map_err(|e| {
            Exception::ControlUnit(BaseException::new(
                "Invalid instruction section pointer".to_string(),
                Some(Box::new(e)),
            ))
        })?;
        let data_section_pointer = self.header_pointer(1, byte_code).map_err(|e| {
            Exception::ControlUnit(BaseException::new(
                "Invalid data section pointer".to_string(),
                Some(Box::new(e)),
            ))
        })?;

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

        let instruction_bytes = self.read_instruction().map_err(|e| {
            Exception::ControlUnit(BaseException::new(
                "Failed to fetch instruction".to_string(),
                Some(Box::new(e)),
            ))
        })?;

        self.registers.set_instruction(Some(instruction_bytes));
        self.registers.advance_instruction_pointer(4);

        Ok(true)
    }

    pub fn decode(&self) -> Result<Instruction, Exception> {
        let bytes = self.registers.get_instruction().ok_or_else(|| {
            Exception::ControlUnit(BaseException::new(
                "No instruction bytes to decode".to_string(),
                None,
            ))
        })?;

        Decoder::decode(&self.memory, &self.registers, bytes).map_err(|e| {
            Exception::ControlUnit(BaseException::new(
                "Failed to decode instruction".to_string(),
                Some(Box::new(e)),
            ))
        })
    }

    pub fn execute(
        &mut self,
        instruction: Instruction,
        text_model: &str,
        embedding_model: &str,
        debug: bool,
        debug_chat: bool,
    ) -> Result<(), Exception> {
        Executor::execute(
            &mut self.memory,
            &mut self.registers,
            &instruction,
            text_model,
            embedding_model,
            debug,
            debug_chat,
        )
        .map_err(|e| {
            Exception::ControlUnit(BaseException::new(
                "Failed to execute instruction".to_string(),
                Some(Box::new(e)),
            ))
        })
    }
}
