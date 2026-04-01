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
    BranchNotEqual = 0x09,
    Exit = 0x0A,
    // I/O.
    Print = 0x0B,
    PrintLine = 0x0C,
    PrintContext = 0x0D,
    // Generative operations.
    Inference = 0x0E,
    // Guardrails operations.
    Evaluate = 0x0F,
    Similarity = 0x10,
    // Context operations.
    ContextPush = 0x11,
    ContextPop = 0x12,
    ContextDrop = 0x13,
    MoveContext = 0x14,
    // Arithmetic operations.
    AddImmediate = 0x15,
    SubtractImmediate = 0x16,
    // Text operations.
    ReadLine = 0x17,
    CountLines = 0x18,
    // Misc.
    NoOp = 0xFF,
}

impl OpCode {
    const ALL: &[OpCode] = &[
        // Data movement.
        OpCode::LoadString,
        OpCode::LoadContent,
        OpCode::LoadImmediate,
        OpCode::Move,
        // Control flow.
        OpCode::BranchEqual,
        OpCode::BranchLessEqual,
        OpCode::BranchLess,
        OpCode::BranchGreaterEqual,
        OpCode::BranchGreater,
        OpCode::BranchNotEqual,
        // I/O.
        OpCode::Exit,
        OpCode::Print,
        OpCode::PrintLine,
        OpCode::PrintContext,
        // Generative operations.
        OpCode::Inference,
        // Guardrails operations.
        OpCode::Evaluate,
        OpCode::Similarity,
        // Context operations.
        OpCode::ContextPush,
        OpCode::ContextPop,
        OpCode::ContextDrop,
        OpCode::MoveContext,
        // Arithmetic operations.
        OpCode::AddImmediate,
        OpCode::SubtractImmediate,
        // Text operations.
        OpCode::ReadLine,
        OpCode::CountLines,
        // Misc.
        OpCode::NoOp,
    ];

    pub fn to_be_bytes(self) -> [u8; 4] {
        (self as u32).to_be_bytes()
    }

    pub fn from_be_bytes(bytes: [u8; 4]) -> Result<OpCode, String> {
        let value = u32::from_be_bytes(bytes);
        OpCode::try_from(value)
    }
}

impl TryFrom<u32> for OpCode {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        OpCode::ALL
            .iter()
            .find(|&&op| op as u32 == value)
            .copied()
            .ok_or_else(|| format!("Unknown opcode value: 0x{:02X}", value))
    }
}

impl From<OpCode> for u32 {
    fn from(op: OpCode) -> u32 {
        op as u32
    }
}
