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

    fn try_from(value: u32) -> Result<Self, <OpCode as TryFrom<u32>>::Error> {
        match value {
            // Data movement.
            x if x == OpCode::LoadString as u32 => Ok(OpCode::LoadString),
            x if x == OpCode::LoadFile as u32 => Ok(OpCode::LoadFile),
            x if x == OpCode::LoadImmediate as u32 => Ok(OpCode::LoadImmediate),
            x if x == OpCode::Move as u32 => Ok(OpCode::Move),
            // Control flow.
            x if x == OpCode::BranchEqual as u32 => Ok(OpCode::BranchEqual),
            x if x == OpCode::BranchLessEqual as u32 => Ok(OpCode::BranchLessEqual),
            x if x == OpCode::BranchLess as u32 => Ok(OpCode::BranchLess),
            x if x == OpCode::BranchGreaterEqual as u32 => Ok(OpCode::BranchGreaterEqual),
            x if x == OpCode::BranchGreater as u32 => Ok(OpCode::BranchGreater),
            x if x == OpCode::Exit as u32 => Ok(OpCode::Exit),
            // I/O.
            x if x == OpCode::Out as u32 => Ok(OpCode::Out),
            // Generative operations.
            x if x == OpCode::Morph as u32 => Ok(OpCode::Morph),
            x if x == OpCode::Project as u32 => Ok(OpCode::Project),
            // Cognitive operations.
            x if x == OpCode::Distill as u32 => Ok(OpCode::Distill),
            x if x == OpCode::Correlate as u32 => Ok(OpCode::Correlate),
            // Guardrails operations.
            x if x == OpCode::Audit as u32 => Ok(OpCode::Audit),
            x if x == OpCode::Similarity as u32 => Ok(OpCode::Similarity),
            // Context operations.
            x if x == OpCode::ContextClear as u32 => Ok(OpCode::ContextClear),
            x if x == OpCode::ContextSnapshot as u32 => Ok(OpCode::ContextSnapshot),
            x if x == OpCode::ContextRestore as u32 => Ok(OpCode::ContextRestore),
            x if x == OpCode::ContextPush as u32 => Ok(OpCode::ContextPush),
            x if x == OpCode::ContextPop as u32 => Ok(OpCode::ContextPop),
            x if x == OpCode::ContextDrop as u32 => Ok(OpCode::ContextDrop),
            x if x == OpCode::ContextSetRole as u32 => Ok(OpCode::ContextSetRole),
            // Misc.
            x if x == OpCode::Decrement as u32 => Ok(OpCode::Decrement),
            _ => Err("Byte value does not correspond to any known opcode.".to_string()),
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
