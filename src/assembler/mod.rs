use std::collections::HashMap;

use crate::assembler::opcode::OpCode;
use crate::assembler::scanner::Scanner;
use crate::assembler::scanner::token::{Token, TokenType};

pub mod opcode;
pub mod roles;
mod scanner;

const HEADER_SIZE: u32 = 2;

impl From<TokenType> for OpCode {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            // Data movement.
            TokenType::LoadString => OpCode::LoadString,
            TokenType::LoadImmediate => OpCode::LoadImmediate,
            TokenType::LoadFile => OpCode::LoadFile,
            TokenType::Move => OpCode::Move,
            // Control flow.
            TokenType::BranchEqual => OpCode::BranchEqual,
            TokenType::BranchLessEqual => OpCode::BranchLessEqual,
            TokenType::BranchLess => OpCode::BranchLess,
            TokenType::BranchGreaterEqual => OpCode::BranchGreaterEqual,
            TokenType::BranchGreater => OpCode::BranchGreater,
            TokenType::Exit => OpCode::Exit,
            // I/O.
            TokenType::Out => OpCode::Out,
            // Generative operations.
            TokenType::Morph => OpCode::Morph,
            TokenType::Project => OpCode::Project,
            // Cognitive operations.
            TokenType::Distill => OpCode::Distill,
            TokenType::Correlate => OpCode::Correlate,
            // Guardrails operations.
            TokenType::Audit => OpCode::Audit,
            TokenType::Similarity => OpCode::Similarity,
            // Context operations.
            TokenType::ContextClear => OpCode::ContextClear,
            TokenType::ContextSnapshot => OpCode::ContextSnapshot,
            TokenType::ContextRestore => OpCode::ContextRestore,
            TokenType::ContextPush => OpCode::ContextPush,
            TokenType::ContextPop => OpCode::ContextPop,
            TokenType::ContextDrop => OpCode::ContextDrop,
            TokenType::ContextSetRole => OpCode::ContextSetRole,
            // Misc operations.
            TokenType::Decrement => OpCode::Decrement,
            // Misc.
            TokenType::Comma
            | TokenType::Identifier
            | TokenType::String
            | TokenType::Number
            | TokenType::Label
            | TokenType::Eof
            | TokenType::Error => OpCode::NoOp,
        }
    }
}

struct UnresolvedLabel {
    indices: Vec<usize>,
    token: Token,
}

pub struct Assembler {
    data_segment: Vec<[u8; 4]>,
    text_segment: Vec<[u8; 4]>,

    source: &'static str,
    scanner: Scanner,

    previous: Option<Token>,
    current: Option<Token>,

    labels: HashMap<String, usize>,
    unresolved_labels: HashMap<String, UnresolvedLabel>,

    had_error: bool,
    panic_mode: bool,
}

impl Assembler {
    pub fn new(source: &'static str) -> Self {
        Assembler {
            data_segment: Vec::new(),
            text_segment: Vec::new(),
            source,
            scanner: Scanner::new(source),
            previous: None,
            current: None,
            labels: HashMap::new(),
            unresolved_labels: HashMap::new(),
            had_error: false,
            panic_mode: false,
        }
    }

    fn lexeme(&self, token: &Token) -> &str {
        &self.source[token.start()..token.end()]
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;

        eprint!("[Line {}:{}] Error:", token.line(), token.column());

        if token.token_type() == &TokenType::Error
            && let Some(error) = token.error()
        {
            eprint!(" {}", error);
        }

        eprint!(" at '{}'.", self.lexeme(token));
        eprintln!(" {}", message);

        self.had_error = true;
    }

    fn error_at_current(&mut self, message: &str) {
        if let Some(token) = &self.current {
            let token = token.clone();
            self.error_at(&token, message);
        } else {
            panic!(
                "Failed to handle error at current token.\nError: {}",
                message
            );
        }
    }

    fn error_at_previous(&mut self, message: &str) {
        if let Some(token) = &self.previous {
            let token = token.clone();
            self.error_at(&token, message);
        } else {
            panic!(
                "Failed to handle error at previous token.\nError: {}",
                message
            );
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        let token = self.scanner.scan_token();
        self.current = Some(token.clone());

        if token.token_type() == &TokenType::Error {
            self.error_at_current("Failed to advance to next token due to scanning error.");
        }
    }

    fn previous_lexeme(&self) -> &str {
        if let Some(token) = &self.previous {
            return self.lexeme(token);
        }

        panic!("Expected previous token to be present, but it is None.");
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) {
        if let Some(current_token) = &self.current
            && current_token.token_type() == token_type
        {
            self.advance();
            return;
        }

        self.error_at_previous(message);
    }

    fn number(&mut self, message: &str) -> u32 {
        self.consume(&TokenType::Number, message);

        match self.previous_lexeme().parse() {
            Ok(value) => value,
            Err(_) => {
                self.error_at_current(&format!(
                    "Failed to parse number from lexeme '{}'.",
                    self.previous_lexeme()
                ));
                0 // Return a default value on error, though the error handling above should prevent this from being used.
            }
        }
    }

    fn register(&mut self, message: &str) -> u32 {
        self.consume(&TokenType::Identifier, message);

        let lexeme = self.previous_lexeme();

        if !lexeme.to_lowercase().starts_with('x') {
            self.error_at_previous(&format!(
                "Invalid register format: '{}'. Expected xN (1-32).",
                lexeme
            ));
            return 0; // Return a default value on error, though the error handling above should prevent this from being used.
        }

        let register_number = if let Ok(number) = lexeme[1..].parse::<u32>() {
            number
        } else {
            self.error_at_previous(&format!(
                "Failed to parse register number from '{}'.",
                lexeme
            ));
            return 0; // Return a default value on error, though the error handling above should prevent this from being used.
        };

        if !(1..=32).contains(&register_number) {
            self.error_at_previous(&format!(
                "Register number {} out of range (1-32).",
                register_number
            ));
            return 0; // Return a default value on error, though the error handling above should prevent this from being used.
        }

        register_number
    }

    fn string(&mut self, message: &str) -> String {
        self.consume(&TokenType::String, message);

        let lexeme = self.previous_lexeme();

        // Strip quotes.
        let inner = &lexeme[1..lexeme.len() - 1];

        inner.replace("\\n", "\n").replace("\\\"", "\"")
    }

    fn identifier(&mut self, message: &str) -> &str {
        self.consume(&TokenType::Identifier, message);
        self.previous_lexeme()
    }

    fn label(&mut self) {
        self.consume(&TokenType::Label, "Expected label name.");

        let label_name = self.previous_lexeme();
        let value = label_name.trim_end_matches(':').to_string();
        let byte_code_index = self.text_segment.len();

        self.labels.insert(value, byte_code_index);
    }

    fn upsert_unresolved_label(&mut self, key: String) {
        let index = self.text_segment.len().saturating_sub(1);

        if let Some(label) = self.unresolved_labels.get_mut(&key) {
            label.indices.push(index);
        } else {
            let previous_token = if let Some(token) = self.previous.clone() {
                token
            } else {
                self.error_at_current("Missing token for unresolved label");
                return;
            };

            self.unresolved_labels.insert(
                key,
                UnresolvedLabel {
                    indices: vec![index],
                    token: previous_token,
                },
            );
        }
    }

    fn backpatch_labels(&mut self) {
        let mut resolved_labels: Vec<String> = Vec::new();

        for (key, unresolved) in &self.unresolved_labels {
            if let Some(byte_code_index) = self.labels.get(key) {
                let index: u32 = match (*byte_code_index).try_into() {
                    Ok(value) => value,
                    Err(_) => {
                        self.error_at_current(&format!(
                            "Failed to convert byte code index to u32 for backpatching. Byte code index exceeds {}. Found byte code index: {}.",
                            u32::MAX,
                            byte_code_index
                        ));
                        return;
                    }
                };

                let bytes = (HEADER_SIZE + index).to_be_bytes();

                for idx in &unresolved.indices {
                    self.text_segment[*idx] = bytes;
                }

                resolved_labels.push(key.clone());
            }
        }

        for key in resolved_labels {
            self.unresolved_labels.remove(&key);
        }
    }

    fn emit_number(&mut self, value: u32) {
        self.text_segment.push(value.to_be_bytes());
    }

    fn emit_opcode(&mut self, op_code: OpCode) {
        self.emit_number(op_code.into());
    }

    fn emit_string(&mut self, value: &str) -> u32 {
        let nulled_value = format!("{}\0", value);
        let words: Vec<[u8; 4]> = nulled_value
            .bytes()
            .map(|byte| u32::from(byte).to_be_bytes())
            .collect();

        let address: u32 = match self.data_segment.len().try_into() {
            Ok(address) => address,
            Err(_) => {
                self.error_at_current(&format!(
                    "Failed to convert data segment length to u32. Data segment length exceeds {}. Found data segment length: {}.",
                    u32::MAX,
                    self.data_segment.len()
                ));
                return 0; // Return a default value on error, though the error handling above should prevent this from being used.
            }
        };

        self.data_segment.extend(words);

        address
    }

    fn emit_label(&mut self, key: String) {
        self.emit_number(0); // Placeholder, will be replaced in backpatch.

        self.upsert_unresolved_label(key);
    }

    fn emit_padding(&mut self, words: usize) {
        for _ in 0..words {
            self.emit_number(0);
        }
    }

    fn expect_not_nop(&mut self, op_code: OpCode) {
        if op_code == OpCode::NoOp {
            self.error_at_current("Invalid opcode: NoOp is reserved for labels and placeholders and cannot be used in instructions.");
        }
    }

    fn immediate(
        &mut self,
        token_type: &TokenType,
        op_code: OpCode,
        string_only: bool,
        number_only: bool,
    ) {
        self.expect_not_nop(op_code);

        if string_only && number_only {
            self.error_at_current(
                "Invalid opcode configuration: cannot be both string-only and number-only.",
            );
            return;
        }

        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type));

        let destination_register = self.register("Expected destination register.");
        self.consume(
            &TokenType::Comma,
            "Expected ',' after destination register.",
        );

        self.emit_opcode(op_code);
        self.emit_number(destination_register);

        if string_only {
            let string = self.string("Expected string after ','.");
            let pointer = self.emit_string(&string);

            self.emit_number(pointer);
            self.emit_padding(1);
        } else if number_only {
            let immediate = self.number("Expected number after ','.");

            self.emit_number(immediate);
            self.emit_padding(1);
        } else {
            let immediate = self.number("Expected immediate after ','.");

            self.emit_number(immediate);
            self.emit_padding(1);
        }
    }

    fn branch(&mut self, token_type: &TokenType, op_code: OpCode) {
        self.expect_not_nop(op_code);

        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type));

        let source_register_1 = self.register("Expected source register 1 after branch keyword.");
        self.consume(&TokenType::Comma, "Expected ',' after source register 1.");

        let source_register_2 = self.register("Expected source register 2 after ','.");
        self.consume(&TokenType::Comma, "Expected ',' after source register 2.");

        let label_name = self
            .identifier("Expected label name after ','.")
            .to_string();

        self.emit_opcode(op_code);
        self.emit_number(source_register_1);
        self.emit_number(source_register_2);
        self.emit_label(label_name);
    }

    fn no_register(&mut self, token_type: &TokenType, op_code: OpCode) {
        self.expect_not_nop(op_code);

        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type));

        self.emit_opcode(op_code);
        self.emit_padding(3);
    }

    fn no_register_string(&mut self, token_type: &TokenType, op_code: OpCode) {
        self.expect_not_nop(op_code);

        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type));

        let string = self.string("Expected string after keyword.");

        match op_code {
            OpCode::ContextSetRole => {
                if string.is_empty() {
                    self.error_at_previous("Role name cannot be empty.");
                    return;
                }

                if !matches!(
                    string.to_lowercase().as_str(),
                    roles::USER_ROLE | roles::ASSISTANT_ROLE
                ) {
                    self.error_at_previous(&format!(
                        "Invalid role name '{}'. Expected '{}' or '{}'.",
                        string,
                        roles::USER_ROLE,
                        roles::ASSISTANT_ROLE
                    ));
                    return;
                }
            }
            _ => {}
        }

        let pointer = self.emit_string(&string);

        self.emit_opcode(op_code);
        self.emit_number(pointer);
        self.emit_padding(2);
    }

    fn single_register(&mut self, token_type: &TokenType, op_code: OpCode) {
        self.expect_not_nop(op_code);

        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type));

        let register = self.register(&format!("Expected register after '{:?}'.", op_code));

        self.emit_opcode(op_code);
        self.emit_number(register);
        self.emit_padding(2);
    }

    fn double_register(&mut self, token_type: &TokenType, op_code: OpCode) {
        self.expect_not_nop(op_code);

        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type));

        let destination_register = self.register(&format!(
            "Expected destination register after '{:?}'.",
            op_code
        ));
        self.consume(
            &TokenType::Comma,
            "Expected ',' after destination register.",
        );

        let source_register =
            self.register(&format!("Expected source register after '{:?}'.", op_code));

        self.emit_opcode(op_code);
        self.emit_number(destination_register);
        self.emit_number(source_register);
        self.emit_padding(1);
    }

    fn triple_register(&mut self, token_type: &TokenType, op_code: OpCode) {
        self.expect_not_nop(op_code);

        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type));

        let destination_register = self.register(&format!(
            "Expected destination register after '{:?}' keyword.",
            op_code
        ));
        self.consume(
            &TokenType::Comma,
            "Expected ',' after destination register.",
        );

        let source_register_1 = self.register("Expected source register 1 after ','.");
        self.consume(&TokenType::Comma, "Expected ',' after source register 1.");

        let source_register_2 = self.register("Expected source register 2 after ','.");

        self.emit_opcode(op_code);
        self.emit_number(destination_register);
        self.emit_number(source_register_1);
        self.emit_number(source_register_2);
    }

    pub fn assemble(&mut self) -> Result<Vec<u8>, &'static str> {
        self.advance();

        while !self.panic_mode {
            if let Some(current_token) = &self.current {
                let token_type = current_token.token_type().clone();
                let op_code: OpCode = token_type.clone().into();

                match token_type {
                    // Data movement.
                    TokenType::LoadImmediate => self.immediate(&token_type, op_code, false, false),
                    TokenType::LoadString | TokenType::LoadFile => {
                        self.immediate(&token_type, op_code, true, false)
                    }
                    TokenType::Move => self.double_register(&token_type, op_code),
                    // Control flow.
                    TokenType::BranchEqual
                    | TokenType::BranchLess
                    | TokenType::BranchLessEqual
                    | TokenType::BranchGreater
                    | TokenType::BranchGreaterEqual => self.branch(&token_type, op_code),
                    TokenType::Exit => self.no_register(&token_type, op_code),
                    TokenType::Label => self.label(),
                    // I/O.
                    TokenType::Out => self.single_register(&token_type, op_code),
                    // Generative, cognitive, and guardrails operations.
                    TokenType::Morph
                    | TokenType::Project
                    | TokenType::Distill
                    | TokenType::Correlate
                    | TokenType::Audit => self.double_register(&token_type, op_code),
                    TokenType::Similarity => self.triple_register(&token_type, op_code),
                    // Context operations.
                    TokenType::ContextClear | TokenType::ContextDrop => {
                        self.no_register(&token_type, op_code)
                    }
                    TokenType::ContextSnapshot
                    | TokenType::ContextRestore
                    | TokenType::ContextPush
                    | TokenType::ContextPop => self.single_register(&token_type, op_code),
                    TokenType::ContextSetRole => self.no_register_string(&token_type, op_code),
                    // Misc operations.
                    TokenType::Decrement => self.immediate(&token_type, op_code, false, true),
                    // Misc.
                    TokenType::Eof => break,
                    _ => self.error_at_current("Unexpected keyword."),
                }
            } else {
                self.error_at_current("Unexpected end of input. Expected more tokens.");
            }
        }

        if self.had_error {
            return Err("Assembly failed due to errors.");
        }

        self.backpatch_labels();

        if let Some((_, unresolved_label)) = self.unresolved_labels.iter().next() {
            let token = unresolved_label.token.clone();

            self.error_at(&token, "Undefined label referenced here.");

            return Err("Assembly failed due to errors.");
        }

        let mut byte_code: Vec<[u8; 4]> = Vec::new();

        // Text segment starts after the header.
        byte_code.push(HEADER_SIZE.to_be_bytes());

        // Data segment starts after the header and text segment.
        let text_segment_size: u32 = match self.text_segment.len().try_into() {
            Ok(size) => size,
            Err(_) => {
                self.error_at_current(&format!(
                    "Failed to convert text segment size to u32. Text segment size exceeds {}. Found text segment size: {}",
                    u32::MAX,
                    self.text_segment.len()
                ));
                return Err("Assembly failed due to errors.");
            }
        };

        byte_code.push((HEADER_SIZE + text_segment_size).to_be_bytes());

        // Append the text segment.
        byte_code.extend(&self.text_segment);

        // Append the data segment after the text segment.
        byte_code.extend(&self.data_segment);

        Ok(byte_code.into_iter().flatten().collect())
    }
}
