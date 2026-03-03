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
    general_purpose: [Value; 32],
    instruction_pointer: usize,
    instruction: Option<[[u8; 4]; 4]>,
    data_section_pointer: usize,
    context: Vec<ContextMessage>,
    context_role: Option<String>,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            general_purpose: [const { Value::None }; 32],
            instruction_pointer: 0,
            instruction: None,
            data_section_pointer: 0,
            context: Vec::new(),
            context_role: None,
        }
    }

    fn to_index(register_number: u32) -> Result<usize, Exception> {
        let idx = match usize::try_from(register_number) {
            Ok(value) => value,
            Err(error) => {
                return Err(Exception::Register(BaseException::new(
                    format!(
                        "Failed to convert register number to usize. Invalid register number: {}. Must be a non-negative integer.",
                        register_number
                    ),
                    Some(Box::new(format!("{:#?}", error).into())),
                )));
            }
        };

        if !(1..=32).contains(&idx) {
            return Err(Exception::Register(BaseException::new(
                format!(
                    "Failed to convert register number. Invalid register number: {}. Valid register numbers are from 1 to 32.",
                    register_number
                ),
                None,
            )));
        }

        Ok(idx - 1)
    }

    pub fn get_register(&self, register_number: u32) -> Result<&Value, Exception> {
        let idx = match Self::to_index(register_number) {
            Ok(value) => value,
            Err(error) => {
                return Err(Exception::Register(BaseException::new(
                    format!("Failed to get register: {}", error),
                    Some(Box::new(error)),
                )));
            }
        };

        Ok(&self.general_purpose[idx])
    }

    pub fn set_register(&mut self, register_number: u32, value: &Value) -> Result<(), Exception> {
        let idx = match Self::to_index(register_number) {
            Ok(value) => value,
            Err(error) => {
                return Err(Exception::Register(BaseException::new(
                    format!("Failed to set register: {}", error),
                    Some(Box::new(error)),
                )));
            }
        };
        self.general_purpose[idx] = value.clone();

        Ok(())
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

    pub fn clear_context(&mut self) {
        self.context.clear();
    }

    pub fn get_context(&self) -> &[ContextMessage] {
        &self.context
    }

    pub fn snapshot_context(&self) -> String {
        miniserde::json::to_string(&self.context)
    }

    pub fn restore_context(&mut self, snapshot: &str) -> Result<(), Exception> {
        self.context = match miniserde::json::from_str(snapshot) {
            Ok(context) => context,
            Err(error) => {
                return Err(Exception::Register(BaseException::new(
                    format!("Failed to restore context from snapshot: {}", error),
                    Some(Box::new(format!("{:#?}", error).into())),
                )));
            }
        };

        Ok(())
    }

    pub fn push_context(&mut self, message: ContextMessage) {
        self.context.push(message);
    }

    pub fn pop_context(&mut self) -> Option<ContextMessage> {
        self.context.pop()
    }

    pub fn get_context_role(&self) -> Option<String> {
        self.context_role.clone()
    }

    pub fn set_context_role(&mut self, role: &str) {
        self.context_role = Some(role.to_string());
    }
}
