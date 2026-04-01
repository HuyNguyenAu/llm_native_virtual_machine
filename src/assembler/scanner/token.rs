#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single-character.
    Comma,
    // Literals.
    Identifier,
    String,
    Number,
    // Data movement keywords.
    LoadString,
    LoadImmediate,
    LoadContent,
    Move,
    // Control flow keywords.
    BranchEqual,
    BranchLessEqual,
    BranchLess,
    BranchGreaterEqual,
    BranchGreater,
    BranchNotEqual,
    Exit,
    // I/O keywords.
    Print,
    PrintLine,
    PrintContext,
    // Generative operations keywords.
    Inference,
    // Guardrails operations keywords.
    Evaluate,
    Similarity,
    // Context operations keywords.
    ContextPush,
    ContextPop,
    ContextDrop,
    MoveContext,
    // Arithmetic operations keywords.
    AddImmediate,
    SubtractImmediate,
    // Line operations keywords.
    ReadLine,
    CountLines,
    // Misc keywords.
    Label,
    Eof,
    Error,
}

impl TryFrom<&str> for TokenType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, <TokenType as TryFrom<&str>>::Error> {
        match value {
            // Data movement.
            "ls" => Ok(TokenType::LoadString),
            "lc" => Ok(TokenType::LoadContent),
            "li" => Ok(TokenType::LoadImmediate),
            "mv" => Ok(TokenType::Move),
            // Control flow.
            "beq" => Ok(TokenType::BranchEqual),
            "ble" => Ok(TokenType::BranchLessEqual),
            "blt" => Ok(TokenType::BranchLess),
            "bge" => Ok(TokenType::BranchGreaterEqual),
            "bgt" => Ok(TokenType::BranchGreater),
            "bne" => Ok(TokenType::BranchNotEqual),
            "exit" => Ok(TokenType::Exit),
            // I/O.
            "put" => Ok(TokenType::Print),
            "pln" => Ok(TokenType::PrintLine),
            "pcx" => Ok(TokenType::PrintContext),
            // Generative operations.
            "inf" => Ok(TokenType::Inference),
            // Guardrails operations.
            "eval" => Ok(TokenType::Evaluate),
            "sim" => Ok(TokenType::Similarity),
            // Context operations.
            "psh" => Ok(TokenType::ContextPush),
            "pop" => Ok(TokenType::ContextPop),
            "drp" => Ok(TokenType::ContextDrop),
            "mvc" => Ok(TokenType::MoveContext),
            // Arithmetic operations.
            "addi" => Ok(TokenType::AddImmediate),
            "subi" => Ok(TokenType::SubtractImmediate),
            // Line operations.
            "rln" => Ok(TokenType::ReadLine),
            "cln" => Ok(TokenType::CountLines),
            _ => Err("String does not correspond to any known token type.".to_string()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    token_type: TokenType,
    start: usize,
    end: usize,
    line: usize,
    column: usize,
    error: Option<String>,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        start: usize,
        end: usize,
        line: usize,
        column: usize,
        error: Option<String>,
    ) -> Token {
        Token {
            token_type,
            start,
            end,
            line,
            column,
            error,
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}
