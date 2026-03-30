#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    // Data movement.
    LoadString = 0x00,
    LoadContent = 0x01,
    LoadImmediate = 0x02,
    Move = 0x03,
    // Control flow.
    BranchEqual = 0x04,
    BranchLessEqual = 0x05,
    BranchLess = 0x06,
    BranchGreaterEqual = 0x07,
    BranchGreater = 0x08,
    Exit = 0x09,
    // I/O.
    Out = 0x0A,
    // Generative operations.
    Inference = 0x0B,
    // Guardrails operations.
    Evaluate = 0x0C,
    Similarity = 0x0D,
    // Context operations.
    ContextPush = 0x0E,
    ContextPop = 0x0F,
    ContextDrop = 0x10,
    // Misc.
    Decrement = 0x11,
    // Misc.
    NoOp = 0xFF,
}

impl TryFrom<u32> for OpCode {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(OpCode::LoadString),
            0x01 => Ok(OpCode::LoadContent),
            0x02 => Ok(OpCode::LoadImmediate),
            0x03 => Ok(OpCode::Move),
            0x04 => Ok(OpCode::BranchEqual),
            0x05 => Ok(OpCode::BranchLessEqual),
            0x06 => Ok(OpCode::BranchLess),
            0x07 => Ok(OpCode::BranchGreaterEqual),
            0x08 => Ok(OpCode::BranchGreater),
            0x09 => Ok(OpCode::Exit),
            0x0A => Ok(OpCode::Out),
            0x0B => Ok(OpCode::Inference),
            0x0C => Ok(OpCode::Evaluate),
            0x0D => Ok(OpCode::Similarity),
            0x0E => Ok(OpCode::ContextPush),
            0x0F => Ok(OpCode::ContextPop),
            0x10 => Ok(OpCode::ContextDrop),
            0x11 => Ok(OpCode::Decrement),
            _ => Err(format!("Unknown opcode value: 0x{:02X}", value)),
        }
    }
}

impl From<OpCode> for u32 {
    fn from(op: OpCode) -> u32 {
        op as u32
    }
}

impl OpCode {
    pub fn to_be_bytes(self) -> [u8; 4] {
        (self as u32).to_be_bytes()
    }

    pub fn from_be_bytes(bytes: [u8; 4]) -> Result<OpCode, String> {
        let value = u32::from_be_bytes(bytes);
        OpCode::try_from(value)
    }
}
