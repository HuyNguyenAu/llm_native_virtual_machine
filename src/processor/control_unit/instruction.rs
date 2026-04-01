#[derive(Debug)]
pub struct LoadStringInstruction {
    pub destination_register: u32,
    pub value: String,
}
#[derive(Debug)]
pub struct LoadImmediateInstruction {
    pub destination_register: u32,
    pub value: u32,
}

#[derive(Debug)]
pub struct LoadContentInstruction {
    pub destination_register: u32,
    pub path: String,
}

#[derive(Debug)]
pub struct MoveInstruction {
    pub destination_register: u32,
    pub source_register: u32,
}

#[derive(Debug)]
pub struct InferenceInstruction {
    pub destination_register: u32,
    pub source_register: u32,
    pub context_register: u32,
}

#[derive(Debug)]
pub struct EvaluateInstruction {
    pub destination_register: u32,
    pub source_register: u32,
    pub context_register: u32,
}

#[derive(Debug)]
pub struct SimilarityInstruction {
    pub destination_register: u32,
    pub source_register_1: u32,
    pub source_register_2: u32,
}

#[derive(Debug)]
pub enum BranchType {
    Equal,
    LessEqual,
    Less,
    GreaterEqual,
    Greater,
    NotEqual,
}

#[derive(Debug)]
pub struct BranchInstruction {
    pub branch_type: BranchType,
    pub source_register_1: u32,
    pub source_register_2: u32,
    pub instruction_pointer_jump_index: u32,
}

#[derive(Debug)]
pub struct ContextPushInstruction {
    pub destination_context_register: u32,
    pub source_register: u32,
    pub role: String,
}

#[derive(Debug)]
pub struct ContextPopInstruction {
    pub destination_register: u32,
    pub source_context_register: u32,
}

#[derive(Debug)]
pub struct ContextDropInstruction {
    pub source_context_register: u32,
}

#[derive(Debug)]
pub struct MoveContextInstruction {
    pub destination_context_register: u32,
    pub source_context_register: u32,
}

#[derive(Debug)]
pub struct AddImmediateInstruction {
    pub destination_register: u32,
    pub value: u32,
}

#[derive(Debug)]
pub struct SubtractImmediateInstruction {
    pub destination_register: u32,
    pub value: u32,
}

#[derive(Debug)]
pub struct PrintInstruction {
    pub source_register: u32,
}

#[derive(Debug)]
pub struct PrintLineInstruction {
    pub source_register: u32,
}

#[derive(Debug)]
pub struct PrintContextInstruction {
    pub source_context_register: u32,
}

#[derive(Debug)]
pub struct ReadLineInstruction {
    pub destination_register: u32,
    pub source_register: u32,
    pub line_number_register: u32,
}

#[derive(Debug)]
pub struct CountLinesInstruction {
    pub destination_register: u32,
    pub source_register: u32,
}

#[derive(Debug)]
pub struct ExitInstruction;

#[derive(Debug)]
pub enum Instruction {
    // Data movement.
    LoadString(LoadStringInstruction),
    LoadImmediate(LoadImmediateInstruction),
    LoadContent(LoadContentInstruction),
    Move(MoveInstruction),
    // Control flow.
    Branch(BranchInstruction),
    Exit(ExitInstruction),
    // I/O.
    Print(PrintInstruction),
    PrintLine(PrintLineInstruction),
    PrintContext(PrintContextInstruction),
    // Generative operations.
    Inference(InferenceInstruction),
    // Guardrails operations.
    Evaluate(EvaluateInstruction),
    Similarity(SimilarityInstruction),
    // Context operations.
    ContextPush(ContextPushInstruction),
    ContextPop(ContextPopInstruction),
    ContextDrop(ContextDropInstruction),
    MoveContext(MoveContextInstruction),
    // Arithmetic operations.
    AddImmediate(AddImmediateInstruction),
    SubtractImmediate(SubtractImmediateInstruction),
    // Line operations.
    ReadLine(ReadLineInstruction),
    CountLines(CountLinesInstruction),
}
