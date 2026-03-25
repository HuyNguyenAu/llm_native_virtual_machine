use std::collections::HashMap;

use crate::assembler::opcode::OpCode;
use crate::assembler::scanner::Scanner;
use crate::assembler::scanner::token::{Token, TokenType};
use crate::exception::{BaseException, Exception};

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

    source: String,
    scanner: Scanner,

    previous: Option<Token>,
    current: Option<Token>,

    labels: HashMap<String, usize>,
    unresolved_labels: HashMap<String, UnresolvedLabel>,

    had_error: bool,
    panic_mode: bool,
}

impl Assembler {
    pub fn new(source: String) -> Self {
        Assembler {
            data_segment: Vec::new(),
            text_segment: Vec::new(),
            source: source.clone(),
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

    fn error_at_current(&mut self, message: &str) -> Result<(), Exception> {
        if let Some(token) = &self.current {
            let token = token.clone();
            self.error_at(&token, message);
        } else {
            return Err(Exception::Assembler(BaseException::new(
                "Unexpected end of input. No current token available for error reporting."
                    .to_string(),
                None,
            )));
        }

        Ok(())
    }

    fn error_at_previous(&mut self, message: &str) -> Result<(), Exception> {
        if let Some(token) = &self.previous {
            let token = token.clone();
            self.error_at(&token, message);
        } else {
            return Err(Exception::Assembler(BaseException::new(
                "Unexpected end of input. No previous token available for error reporting."
                    .to_string(),
                None,
            )));
        }

        Ok(())
    }

    fn advance(&mut self) -> Result<(), Exception> {
        self.previous = self.current.clone();

        let token = self.scanner.scan_token();
        self.current = Some(token.clone());

        if token.token_type() == &TokenType::Error {
            return self.error_at_current("Failed to advance to next token due to scanning error.");
        }

        Ok(())
    }

    fn previous_lexeme(&self) -> Result<&str, Exception> {
        if let Some(token) = &self.previous {
            return Ok(self.lexeme(token));
        }

        Err(Exception::Assembler(BaseException::new(
            "Failed to retrieve previous lexeme because there is no previous token.".to_string(),
            None,
        )))
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<(), Exception> {
        if let Some(current_token) = &self.current
            && current_token.token_type() == token_type
        {
            if let Err(exception) = self.advance() {
                return Err(Exception::Assembler(BaseException::new(
                    format!(
                        "Failed to advance after consuming expected token '{:?}'.",
                        token_type
                    ),
                    Some(Box::new(exception)),
                )));
            }

            return Ok(());
        }

        self.error_at_previous(message)
    }

    fn number(&mut self, message: &str) -> Result<u32, Exception> {
        self.consume(&TokenType::Number, message)?;

        let previous_lexeme = match self.previous_lexeme() {
            Ok(lexeme) => lexeme,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    message.to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        match previous_lexeme.parse() {
            Ok(value) => Ok(value),
            Err(error) => {
                let message = format!("Failed to parse number from lexeme '{}'.", previous_lexeme);

                match self.error_at_current(&message) {
                    Ok(_) => Err(Exception::Assembler(BaseException::new(
                        message.clone(),
                        Some(Box::new(error.into())),
                    ))),
                    Err(exception) => Err(Exception::Assembler(BaseException::new(
                        message.clone(),
                        Some(Box::new(exception)),
                    ))),
                }
            }
        }
    }

    fn register(&mut self, message: &str) -> Result<u32, Exception> {
        self.consume(&TokenType::Identifier, message)?;

        let lexeme = match self.previous_lexeme() {
            Ok(lexeme) => lexeme,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    message.to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        if !lexeme.to_lowercase().starts_with('x') {
            let message = format!("Invalid register format: '{}'. Expected xN (1-32).", lexeme);

            match self.error_at_previous(&message) {
                Ok(_) => return Err(Exception::Assembler(BaseException::new(message, None))),
                Err(exception) => {
                    return Err(Exception::Assembler(BaseException::new(
                        message,
                        Some(Box::new(exception)),
                    )));
                }
            };
        }

        let register_number = if let Ok(number) = lexeme[1..].parse::<u32>() {
            number
        } else {
            let message = format!("Failed to parse register number from '{}'.", lexeme);

            match self.error_at_previous(&message) {
                Ok(_) => return Err(Exception::Assembler(BaseException::new(message, None))),
                Err(exception) => {
                    return Err(Exception::Assembler(BaseException::new(
                        message,
                        Some(Box::new(exception)),
                    )));
                }
            };
        };

        if !(1..=32).contains(&register_number) {
            let message = format!("Register number {} out of range (1-32).", register_number);

            match self.error_at_previous(&message) {
                Ok(_) => return Err(Exception::Assembler(BaseException::new(message, None))),
                Err(exception) => {
                    return Err(Exception::Assembler(BaseException::new(
                        message,
                        Some(Box::new(exception)),
                    )));
                }
            }
        }

        Ok(register_number)
    }

    fn string(&mut self, message: &str) -> Result<String, Exception> {
        self.consume(&TokenType::String, message)?;

        let lexeme = match self.previous_lexeme() {
            Ok(lexeme) => lexeme,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Failed to parse string".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        // Strip quotes.
        let inner = &lexeme[1..lexeme.len() - 1];

        Ok(inner.replace("\\n", "\n").replace("\\\"", "\""))
    }

    fn identifier(&mut self, message: &str) -> Result<&str, Exception> {
        match self.consume(&TokenType::Identifier, message) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    message.to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        match self.previous_lexeme() {
            Ok(lexeme) => Ok(lexeme),
            Err(exception) => Err(Exception::Assembler(BaseException::new(
                message.to_string(),
                Some(Box::new(exception)),
            ))),
        }
    }

    fn label(&mut self) -> Result<(), Exception> {
        self.consume(&TokenType::Label, "Expected label name.")?;

        let label_name = match self.previous_lexeme() {
            Ok(name) => name,
            Err(exception) => {
                let message = "Failed to parse label name.".to_string();

                match self.error_at_current(&message) {
                    Ok(_) => {
                        return Err(Exception::Assembler(BaseException::new(
                            message,
                            Some(Box::new(exception)),
                        )));
                    }
                    Err(exception) => {
                        return Err(Exception::Assembler(BaseException::new(
                            message,
                            Some(Box::new(exception)),
                        )));
                    }
                }
            }
        };
        let value = label_name.trim_end_matches(':').to_string();
        let byte_code_index = self.text_segment.len();

        self.labels.insert(value, byte_code_index);

        Ok(())
    }

    fn upsert_unresolved_label(&mut self, key: String) -> Result<(), Exception> {
        let index = self.text_segment.len().saturating_sub(1);

        if let Some(label) = self.unresolved_labels.get_mut(&key) {
            label.indices.push(index);
        } else {
            let previous_token = if let Some(token) = self.previous.clone() {
                token
            } else {
                let message =
                    "Failed to retrieve previous token for unresolved label error reporting.";

                return match self.error_at_current(message) {
                    Ok(_) => Err(Exception::Assembler(BaseException::new(
                        message.to_string(),
                        None,
                    ))),
                    Err(exception) => Err(Exception::Assembler(BaseException::new(
                        message.to_string(),
                        Some(Box::new(exception)),
                    ))),
                };
            };

            self.unresolved_labels.insert(
                key,
                UnresolvedLabel {
                    indices: vec![index],
                    token: previous_token,
                },
            );
        }

        Ok(())
    }

    fn backpatch_labels(&mut self) -> Result<(), Exception> {
        let mut resolved_labels: Vec<String> = Vec::new();

        for (key, unresolved) in &self.unresolved_labels {
            if let Some(byte_code_index) = self.labels.get(key) {
                let index: u32 = match (*byte_code_index).try_into() {
                    Ok(value) => value,
                    Err(_) => {
                        let message = format!(
                            "Failed to convert byte code index to u32 for backpatching. Byte code index exceeds {}. Found byte code index: {}.",
                            u32::MAX,
                            byte_code_index
                        );

                        return match self.error_at_current(&message) {
                            Ok(_) => Err(Exception::Assembler(BaseException::new(
                                message.clone(),
                                None,
                            ))),
                            Err(exception) => Err(Exception::Assembler(BaseException::new(
                                message.clone(),
                                Some(Box::new(exception)),
                            ))),
                        };
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

        Ok(())
    }

    fn emit_number(&mut self, value: u32) {
        self.text_segment.push(value.to_be_bytes());
    }

    fn emit_opcode(&mut self, op_code: OpCode) {
        self.emit_number(op_code.into());
    }

    fn emit_string(&mut self, value: &str) -> Result<u32, Exception> {
        let nulled_value = format!("{}\0", value);
        let words: Vec<[u8; 4]> = nulled_value
            .bytes()
            .map(|byte| u32::from(byte).to_be_bytes())
            .collect();

        let address: u32 = match self.data_segment.len().try_into() {
            Ok(address) => address,
            Err(_) => {
                let message = format!(
                    "Failed to convert data segment length to u32 for string emission. Data segment length exceeds {}. Found data segment length: {}.",
                    u32::MAX,
                    self.data_segment.len()
                );

                return match self.error_at_current(&message) {
                    Ok(_) => Err(Exception::Assembler(BaseException::new(
                        message.clone(),
                        None,
                    ))),
                    Err(exception) => Err(Exception::Assembler(BaseException::new(
                        message.clone(),
                        Some(Box::new(exception)),
                    ))),
                };
            }
        };

        self.data_segment.extend(words);

        Ok(address)
    }

    fn emit_label(&mut self, key: String) -> Result<(), Exception> {
        self.emit_number(0); // Placeholder, will be replaced in backpatch.

        self.upsert_unresolved_label(key)
    }

    fn emit_padding(&mut self, words: usize) {
        for _ in 0..words {
            self.emit_number(0);
        }
    }

    fn expect_not_nop(&mut self, op_code: OpCode) -> Result<(), Exception> {
        if op_code == OpCode::NoOp {
            return self.error_at_current("Invalid opcode: NoOp is reserved for labels and placeholders and cannot be used in instructions.");
        }

        Ok(())
    }

    fn immediate(
        &mut self,
        token_type: &TokenType,
        op_code: OpCode,
        string_only: bool,
        number_only: bool,
    ) -> Result<(), Exception> {
        match self.expect_not_nop(op_code) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Unexpected noop opcode.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        if string_only && number_only {
            return Err(Exception::Assembler(BaseException::new(
                "An instruction cannot be both string-only and number-only.".to_string(),
                None,
            )));
        }

        if let Err(exception) =
            self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))
        {
            return Err(Exception::Assembler(BaseException::new(
                format!(
                    "Failed to consume expected keyword '{:?}' for immediate instruction.",
                    token_type
                ),
                Some(Box::new(exception)),
            )));
        };

        let destination_register = match self.register("Expected destination register.") {
            Ok(register) => register,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Failed to parse destination register for immediate instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        if let Err(exception) = self.consume(
            &TokenType::Comma,
            "Expected ',' after destination register.",
        ) {
            return Err(Exception::Assembler(BaseException::new(
                "Failed to consume ',' after destination register for immediate instruction."
                    .to_string(),
                Some(Box::new(exception)),
            )));
        }

        self.emit_opcode(op_code);
        self.emit_number(destination_register);

        if string_only {
            let string = match self.string("Expected string after ','.") {
                Ok(string) => string,
                Err(exception) => {
                    return Err(Exception::Assembler(BaseException::new(
                        "Failed to parse string immediate value.".to_string(),
                        Some(Box::new(exception)),
                    )));
                }
            };
            let pointer = match self.emit_string(&string) {
                Ok(pointer) => pointer,
                Err(exception) => {
                    return Err(Exception::Assembler(BaseException::new(
                        "Failed to emit string immediate value.".to_string(),
                        Some(Box::new(exception)),
                    )));
                }
            };

            self.emit_number(pointer);
            self.emit_padding(1);
        } else if number_only {
            let immediate = match self.number("Expected number after ','.") {
                Ok(number) => number,
                Err(exception) => {
                    return Err(Exception::Assembler(BaseException::new(
                        "Failed to parse numeric immediate value.".to_string(),
                        Some(Box::new(exception)),
                    )));
                }
            };

            self.emit_number(immediate);
            self.emit_padding(1);
        } else {
            let immediate = match self.number("Expected immediate after ','.") {
                Ok(number) => number,
                Err(exception) => {
                    return Err(Exception::Assembler(BaseException::new(
                        "Failed to parse immediate value.".to_string(),
                        Some(Box::new(exception)),
                    )));
                }
            };

            self.emit_number(immediate);
            self.emit_padding(1);
        }

        Ok(())
    }

    fn branch(&mut self, token_type: &TokenType, op_code: OpCode) -> Result<(), Exception> {
        match self.expect_not_nop(op_code) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Unexpected noop opcode.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        match self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type)) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    format!(
                        "Failed to consume expected keyword '{:?}' for branch instruction.",
                        token_type
                    ),
                    Some(Box::new(exception)),
                )));
            }
        };

        let source_register_1 =
            match self.register("Expected source register 1 after branch keyword.") {
                Ok(register) => register,
                Err(exception) => {
                    return Err(Exception::Assembler(BaseException::new(
                        "Failed to parse source register 1 for branch instruction.".to_string(),
                        Some(Box::new(exception)),
                    )));
                }
            };
        match self.consume(&TokenType::Comma, "Expected ',' after source register 1.") {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Failed to consume ',' after source register 1 for branch instruction."
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        let source_register_2 = match self.register("Expected source register 2 after ','.") {
            Ok(register) => register,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Failed to parse source register 2 for branch instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };
        match self.consume(&TokenType::Comma, "Expected ',' after source register 2.") {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Failed to consume ',' after source register 2 for branch instruction."
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        let label_name = match self.identifier("Expected label name after ','.") {
            Ok(name) => name.to_string(),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Failed to parse label name for branch instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        self.emit_opcode(op_code);
        self.emit_number(source_register_1);
        self.emit_number(source_register_2);
        match self.emit_label(label_name) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Failed to emit label for branch instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        Ok(())
    }

    fn no_register(&mut self, token_type: &TokenType, op_code: OpCode) -> Result<(), Exception> {
        match self.expect_not_nop(op_code) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Unexpected noop opcode.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        match self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type)) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    format!("Failed to consume expected keyword '{:?}'.", token_type),
                    Some(Box::new(exception)),
                )));
            }
        };

        self.emit_opcode(op_code);
        self.emit_padding(3);

        Ok(())
    }

    fn no_register_string(
        &mut self,
        token_type: &TokenType,
        op_code: OpCode,
    ) -> Result<(), Exception> {
        match self.expect_not_nop(op_code) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Unexpected noop opcode.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        match self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type)) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    format!("Failed to consume expected keyword '{:?}'.", token_type),
                    Some(Box::new(exception)),
                )));
            }
        };

        let string = match self.string("Expected string after keyword.") {
            Ok(string) => string,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Failed to parse string argument.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        if op_code == OpCode::ContextSetRole {
            if string.is_empty() {
                match self.error_at_previous("Role name cannot be empty.") {
                    Ok(_) => (),
                    Err(exception) => {
                        return Err(Exception::Assembler(BaseException::new(
                            "Role name cannot be empty.".to_string(),
                            Some(Box::new(exception)),
                        )));
                    }
                };

                return Err(Exception::Assembler(BaseException::new(
                    "Role name cannot be empty.".to_string(),
                    None,
                )));
            }

            if !matches!(
                string.to_lowercase().as_str(),
                roles::USER_ROLE | roles::ASSISTANT_ROLE
            ) {
                if let Err(exception) = self.error_at_previous(&format!(
                    "Invalid role name '{}'. Expected '{}' or '{}'.",
                    string,
                    roles::USER_ROLE,
                    roles::ASSISTANT_ROLE
                )) {
                    return Err(Exception::Assembler(BaseException::new(
                        format!(
                            "Invalid role name '{}'. Expected '{}' or '{}'.",
                            string,
                            roles::USER_ROLE,
                            roles::ASSISTANT_ROLE
                        ),
                        Some(Box::new(exception)),
                    )));
                }

                return Err(Exception::Assembler(BaseException::new(
                    format!(
                        "Invalid role name '{}'. Expected '{}' or '{}'.",
                        string,
                        roles::USER_ROLE,
                        roles::ASSISTANT_ROLE
                    ),
                    None,
                )));
            }
        }

        let pointer = match self.emit_string(&string) {
            Ok(pointer) => pointer,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Failed to emit string.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        self.emit_opcode(op_code);
        self.emit_number(pointer);
        self.emit_padding(2);

        Ok(())
    }

    fn single_register(
        &mut self,
        token_type: &TokenType,
        op_code: OpCode,
    ) -> Result<(), Exception> {
        if let Err(exception) = self.expect_not_nop(op_code) {
            return Err(Exception::Assembler(BaseException::new(
                "Unexpected noop opcode.".to_string(),
                Some(Box::new(exception)),
            )));
        }

        if let Err(exception) =
            self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))
        {
            return Err(Exception::Assembler(BaseException::new(
                format!("Expected '{:?}' keyword.", token_type),
                Some(Box::new(exception)),
            )));
        }

        let register = match self.register(&format!("Expected register after '{:?}'.", op_code)) {
            Ok(register) => register,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    format!("Expected register after '{:?}'.", op_code),
                    Some(Box::new(exception)),
                )));
            }
        };

        self.emit_opcode(op_code);
        self.emit_number(register);
        self.emit_padding(2);

        Ok(())
    }

    fn double_register(
        &mut self,
        token_type: &TokenType,
        op_code: OpCode,
    ) -> Result<(), Exception> {
        if let Err(exception) = self.expect_not_nop(op_code) {
            return Err(Exception::Assembler(BaseException::new(
                "Unexpected noop opcode.".to_string(),
                Some(Box::new(exception)),
            )));
        }

        if let Err(exception) =
            self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))
        {
            return Err(Exception::Assembler(BaseException::new(
                format!("Expected '{:?}' keyword.", token_type),
                Some(Box::new(exception)),
            )));
        }

        let destination_register = match self.register(&format!(
            "Expected destination register after '{:?}'.",
            op_code
        )) {
            Ok(register) => register,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    format!("Expected destination register after '{:?}'.", op_code),
                    Some(Box::new(exception)),
                )));
            }
        };

        if let Err(exception) = self.consume(
            &TokenType::Comma,
            "Expected ',' after destination register.",
        ) {
            return Err(Exception::Assembler(BaseException::new(
                "Expected ',' after destination register.".to_string(),
                Some(Box::new(exception)),
            )));
        }

        let source_register =
            match self.register(&format!("Expected source register after '{:?}'.", op_code)) {
                Ok(register) => register,
                Err(exception) => {
                    return Err(Exception::Assembler(BaseException::new(
                        format!("Expected source register after '{:?}'.", op_code),
                        Some(Box::new(exception)),
                    )));
                }
            };

        self.emit_opcode(op_code);
        self.emit_number(destination_register);
        self.emit_number(source_register);
        self.emit_padding(1);

        Ok(())
    }

    fn triple_register(
        &mut self,
        token_type: &TokenType,
        op_code: OpCode,
    ) -> Result<(), Exception> {
        if let Err(exception) = self.expect_not_nop(op_code) {
            return Err(Exception::Assembler(BaseException::new(
                "Unexpected noop opcode.".to_string(),
                Some(Box::new(exception)),
            )));
        }

        if let Err(exception) =
            self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))
        {
            return Err(Exception::Assembler(BaseException::new(
                format!("Expected '{:?}' keyword.", token_type),
                Some(Box::new(exception)),
            )));
        }

        let destination_register = match self.register(&format!(
            "Expected destination register after '{:?}' keyword.",
            op_code
        )) {
            Ok(register) => register,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    format!(
                        "Expected destination register after '{:?}' keyword.",
                        op_code
                    ),
                    Some(Box::new(exception)),
                )));
            }
        };
        if let Err(exception) = self.consume(
            &TokenType::Comma,
            "Expected ',' after destination register.",
        ) {
            return Err(Exception::Assembler(BaseException::new(
                "Expected ',' after destination register.".to_string(),
                Some(Box::new(exception)),
            )));
        }

        let source_register_1 = match self.register("Expected source register 1 after ','.") {
            Ok(register) => register,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Expected source register 1 after ','.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        if let Err(exception) =
            self.consume(&TokenType::Comma, "Expected ',' after source register 1.")
        {
            return Err(Exception::Assembler(BaseException::new(
                "Expected ',' after source register 1.".to_string(),
                Some(Box::new(exception)),
            )));
        }

        let source_register_2 = match self.register("Expected source register 2 after ','.") {
            Ok(register) => register,
            Err(exception) => {
                return Err(Exception::Assembler(BaseException::new(
                    "Expected source register 2 after ','.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        self.emit_opcode(op_code);
        self.emit_number(destination_register);
        self.emit_number(source_register_1);
        self.emit_number(source_register_2);

        Ok(())
    }

    pub fn assemble(&mut self) -> Result<Vec<u8>, Exception> {
        if let Err(exception) = self.advance() {
            return Err(Exception::Assembler(BaseException::new(
                "Failed to advance to the first token.".to_string(),
                Some(Box::new(exception)),
            )));
        }

        while !self.panic_mode {
            if let Some(current_token) = &self.current {
                let token_type = current_token.token_type().clone();
                let op_code: OpCode = token_type.clone().into();

                let result = match token_type {
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
                };

                result?;
            } else {
                self.error_at_current("Unexpected end of input. Expected more tokens.")?;
            }
        }

        if self.had_error {
            return Err(Exception::Assembler(BaseException::new(
                "Assembly failed due to errors.".to_string(),
                None,
            )));
        }

        self.backpatch_labels()?;

        if let Some((_, unresolved_label)) = self.unresolved_labels.iter().next() {
            let token = unresolved_label.token.clone();

            self.error_at(&token, "Undefined label referenced here.");

            return Err(Exception::Assembler(BaseException::new(
                "Assembly failed due to errors.".to_string(),
                None,
            )));
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
                ))?;

                return Err(Exception::Assembler(BaseException::new(
                    "Assembly failed due to errors.".to_string(),
                    None,
                )));
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
