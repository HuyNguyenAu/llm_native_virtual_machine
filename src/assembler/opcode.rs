#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    // Data movement.
    LoadString = 0x00,
    LoadFile = 0x01,
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
    Morph = 0x0B,
    Project = 0x0C,
    // Cognitive operations.
    Distill = 0x0D,
    Correlate = 0x0E,
    // Guardrails operations.
    Audit = 0x0F,
    Similarity = 0x10,
    // Context operations.
    ContextClear = 0x11,
    ContextSnapshot = 0x12,
    ContextRestore = 0x13,
    ContextPush = 0x14,
    ContextPop = 0x15,
    ContextDrop = 0x16,
    ContextSetRole = 0x17,
    // Misc.
    Decrement = 0x18,
    // Misc.
    NoOp = 0xFF,
}

impl TryFrom<u32> for OpCode {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(OpCode::LoadString),
            0x01 => Ok(OpCode::LoadFile),
            0x02 => Ok(OpCode::LoadImmediate),
            0x03 => Ok(OpCode::Move),
            0x04 => Ok(OpCode::BranchEqual),
            0x05 => Ok(OpCode::BranchLessEqual),
            0x06 => Ok(OpCode::BranchLess),
            0x07 => Ok(OpCode::BranchGreaterEqual),
            0x08 => Ok(OpCode::BranchGreater),
            0x09 => Ok(OpCode::Exit),
            0x0A => Ok(OpCode::Out),
            0x0B => Ok(OpCode::Morph),
            0x0C => Ok(OpCode::Project),
            0x0D => Ok(OpCode::Distill),
            0x0E => Ok(OpCode::Correlate),
            0x0F => Ok(OpCode::Audit),
            0x10 => Ok(OpCode::Similarity),
            0x11 => Ok(OpCode::ContextClear),
            0x12 => Ok(OpCode::ContextSnapshot),
            0x13 => Ok(OpCode::ContextRestore),
            0x14 => Ok(OpCode::ContextPush),
            0x15 => Ok(OpCode::ContextPop),
            0x16 => Ok(OpCode::ContextDrop),
            0x17 => Ok(OpCode::ContextSetRole),
            0x18 => Ok(OpCode::Decrement),
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
