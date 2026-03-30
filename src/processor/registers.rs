use std::fmt;

use miniserde::{Deserialize, Serialize};

use crate::exception::{BaseException, Exception};

#[derive(Debug, Clone)]
pub enum Value {
    Text(String),
    Number(u32),
    None,
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Text(text) => write!(formatter, "{}", text),
            Value::Number(number) => write!(formatter, "{}", number),
            Value::None => write!(formatter, ""),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
}

impl ContextMessage {
    pub fn new(role: &str, content: &str) -> Self {
        ContextMessage {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

pub struct Registers {
    general_purpose: [Value; 33],
    context: [Vec<ContextMessage>; 33],
    instruction_pointer: usize,
    instruction: Option<[[u8; 4]; 4]>,
    data_section_pointer: usize,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            general_purpose: [const { Value::None }; 33],
            context: [const { Vec::new() }; 33],
            instruction_pointer: 0,
            instruction: None,
            data_section_pointer: 0,
        }
    }

    fn to_index(register_number: u32) -> Result<usize, Exception> {
        let idx = usize::try_from(register_number).map_err(|_| {
            Exception::Register(BaseException::new(
                format!("Invalid register number: {}", register_number),
                None,
            ))
        })?;

        if !(0..=32).contains(&idx) {
            return Err(Exception::Register(BaseException::new(
                format!("Register number {} out of range (0-32).", register_number),
                None,
            )));
        }

        Ok(idx)
    }

    pub fn get_register(&self, register_number: u32) -> Result<&Value, Exception> {
        let idx = Self::to_index(register_number)?;
        Ok(&self.general_purpose[idx])
    }

    pub fn set_register(&mut self, register_number: u32, value: &Value) -> Result<(), Exception> {
        let idx = Self::to_index(register_number)?;

        if idx == 0 {
            return Err(Exception::Register(BaseException::new(
                "Cannot write to register 0 (reserved for none value).".to_string(),
                None,
            )));
        }

        self.general_purpose[idx] = value.clone();
        Ok(())
    }

    pub fn get_context(&self, register_number: u32) -> Result<&[ContextMessage], Exception> {
        let idx = Self::to_index(register_number)?;
        Ok(&self.context[idx])
    }

    pub fn set_context(
        &mut self,
        register_number: u32,
        messages: &[ContextMessage],
    ) -> Result<(), Exception> {
        let idx = Self::to_index(register_number)?;

        if idx == 0 {
            return Err(Exception::Register(BaseException::new(
                "Cannot write to context register 0 (reserved for empty value).".to_string(),
                None,
            )));
        }

        self.context[idx] = messages.to_vec();
        Ok(())
    }

    pub fn push_context(
        &mut self,
        message: ContextMessage,
        register_number: u32,
    ) -> Result<(), Exception> {
        let idx = Self::to_index(register_number)?;

        if idx == 0 {
            return Err(Exception::Register(BaseException::new(
                "Cannot write to context register 0 (reserved for empty value).".to_string(),
                None,
            )));
        }

        self.context[idx].push(message);
        Ok(())
    }

    pub fn pop_context(&mut self, register_number: u32) -> Result<ContextMessage, Exception> {
        let idx = Self::to_index(register_number)?;

        if idx == 0 {
            return Err(Exception::Register(BaseException::new(
                "Cannot read from context register 0 (reserved for empty value).".to_string(),
                None,
            )));
        }

        self.context[idx].pop().ok_or_else(|| {
            Exception::Register(BaseException::new(
                format!("Context stack for register {} is empty.", register_number),
                None,
            ))
        })
    }

    pub fn get_instruction_pointer(&self) -> usize {
        self.instruction_pointer
    }

    pub fn set_instruction_pointer(&mut self, address: usize) {
        self.instruction_pointer = address;
    }

    pub fn advance_instruction_pointer(&mut self, offset: usize) {
        self.instruction_pointer += offset;
    }

    pub fn get_instruction(&self) -> Option<[[u8; 4]; 4]> {
        self.instruction
    }

    pub fn set_instruction(&mut self, be_bytes: Option<[[u8; 4]; 4]>) {
        self.instruction = be_bytes;
    }

    pub fn get_data_section_pointer(&self) -> usize {
        self.data_section_pointer
    }

    pub fn set_data_section_pointer(&mut self, address: usize) {
        self.data_section_pointer = address;
    }
}
