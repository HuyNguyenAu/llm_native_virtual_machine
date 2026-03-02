use crate::exception::{BaseException, Exception};

pub struct Memory {
    data: Vec<[u8; 4]>,
}

impl Memory {
    pub fn new() -> Self {
        Memory { data: Vec::new() }
    }

    pub fn load(&mut self, byte_code: Vec<[u8; 4]>) {
        self.data = byte_code;
    }

    pub fn read(&self, address: usize) -> Result<&[u8; 4], Exception> {
        match self.data.get(address) {
            Some(instruction) => Ok(instruction),
            None => Err(Exception::MemoryException(BaseException::new(
                format!("Address out of bounds: {}", address),
                None,
            ))),
        }
    }

    pub fn length(&self) -> usize {
        self.data.len()
    }
}
