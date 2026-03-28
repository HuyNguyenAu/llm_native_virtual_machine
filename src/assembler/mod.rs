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
            TokenType::Map => OpCode::Map,
            // Cognitive operations.
            TokenType::Eval => OpCode::Eval,
            // Guardrails operations.
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
        let scanner = Scanner::new(source.clone());

        Assembler {
            data_segment: Vec::new(),
            text_segment: Vec::new(),
            source,
            scanner,
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
        let token = self
            .current
            .as_ref()
            .ok_or_else(|| {
                Exception::Assembler(BaseException::new(
                    "Unexpected end of input. No current token available for error reporting."
                        .to_string(),
                    None,
                ))
            })?
            .clone();

        self.error_at(&token, message);
        Ok(())
    }

    fn error_at_previous(&mut self, message: &str) -> Result<(), Exception> {
        let token = self
            .previous
            .as_ref()
            .ok_or_else(|| {
                Exception::Assembler(BaseException::new(
                    "Unexpected end of input. No previous token available for error reporting."
                        .to_string(),
                    None,
                ))
            })?
            .clone();

        self.error_at(&token, message);
        Ok(())
    }

    fn advance(&mut self) -> Result<(), Exception> {
        self.previous = self.current.clone();

        let token = self.scanner.scan_token();
        self.current = Some(token.clone());

        if token.token_type() == &TokenType::Error {
            self.error_at_current("Failed to advance to next token due to scanning error.")?;
            return Err(Exception::Assembler(BaseException::new(
                "Failed to advance to next token due to scanning error.".to_string(),
                None,
            )));
        }

        Ok(())
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<(), Exception> {
        if self
            .current
            .as_ref()
            .map(|token| token.token_type() == token_type)
            .unwrap_or(false)
        {
            self.advance()?;
            Ok(())
        } else {
            self.error_at_previous(message)
        }
    }

    fn previous_lexeme(&self) -> Result<&str, Exception> {
        let token = self.previous.as_ref().ok_or_else(|| {
            Exception::Assembler(BaseException::new(
                "Failed to retrieve previous lexeme because there is no previous token."
                    .to_string(),
                None,
            ))
        })?;

        Ok(self.lexeme(token))
    }

    fn number(&mut self, message: &str) -> Result<u32, Exception> {
        self.consume(&TokenType::Number, message)?;
        let previous_lexeme = self.previous_lexeme()?;

        match previous_lexeme.parse::<u32>() {
            Ok(value) => Ok(value),
            Err(error) => {
                let message = format!("Failed to parse number from lexeme '{}'.", previous_lexeme);
                let _ = self.error_at_current(&message);
                Err(Exception::Assembler(BaseException::caused_by(
                    message, error,
                )))
            }
        }
    }

    fn register(&mut self, message: &str) -> Result<u32, Exception> {
        self.consume(&TokenType::Identifier, message)?;
        let lexeme = self.previous_lexeme()?;

        if !lexeme.to_lowercase().starts_with('x') {
            let err = format!("Invalid register format: '{}'. Expected xN (1-32).", lexeme);
            self.error_at_previous(&err)?;
            return Err(Exception::Assembler(BaseException::new(err, None)));
        }

        let register_number = match lexeme[1..].parse::<u32>() {
            Ok(v) => v,
            Err(_) => {
                let err = format!("Failed to parse register number from '{}'.", lexeme);
                let _ = self.error_at_previous(&err);
                return Err(Exception::Assembler(BaseException::new(err, None)));
            }
        };

        if !(1..=32).contains(&register_number) {
            let err = format!("Register number {} out of range (1-32).", register_number);
            self.error_at_previous(&err)?;
            return Err(Exception::Assembler(BaseException::new(err, None)));
        }

        Ok(register_number)
    }

    fn string(&mut self, message: &str) -> Result<String, Exception> {
        self.consume(&TokenType::String, message)?;
        let lexeme = self.previous_lexeme()?;

        let inner = &lexeme[1..lexeme.len() - 1];
        Ok(inner.replace("\\n", "\n").replace("\\\"", "\""))
    }

    fn identifier(&mut self, message: &str) -> Result<&str, Exception> {
        self.consume(&TokenType::Identifier, message)?;
        self.previous_lexeme()
    }

    fn label(&mut self) -> Result<(), Exception> {
        self.consume(&TokenType::Label, "Expected label name.")?;
        let label_name = self.previous_lexeme()?.trim_end_matches(':').to_string();
        let byte_code_index = self.text_segment.len();
        self.labels.insert(label_name, byte_code_index);
        Ok(())
    }

    fn upsert_unresolved_label(&mut self, key: String) -> Result<(), Exception> {
        let index = self.text_segment.len().saturating_sub(1);

        if let Some(label) = self.unresolved_labels.get_mut(&key) {
            label.indices.push(index);
            return Ok(());
        }

        let token = self.previous.clone().ok_or_else(|| {
            Exception::Assembler(BaseException::new(
                "Failed to retrieve previous token for unresolved label error reporting."
                    .to_string(),
                None,
            ))
        })?;

        self.unresolved_labels.insert(
            key,
            UnresolvedLabel {
                indices: vec![index],
                token,
            },
        );

        Ok(())
    }

    fn backpatch_labels(&mut self) -> Result<(), Exception> {
        let mut resolved_keys = Vec::with_capacity(self.unresolved_labels.len());

        for (key, unresolved) in &self.unresolved_labels {
            if let Some(&byte_code_index) = self.labels.get(key) {
                let index = match u32::try_from(byte_code_index) {
                    Ok(v) => v,
                    Err(_) => {
                        let message = format!(
                            "Failed to convert byte code index to u32 for backpatching. Byte code index exceeds {}. Found byte code index: {}.",
                            u32::MAX,
                            byte_code_index
                        );
                        let _ = self.error_at_current(&message);
                        return Err(Exception::Assembler(BaseException::new(message, None)));
                    }
                };

                let bytes = (HEADER_SIZE + index).to_be_bytes();

                for &idx in &unresolved.indices {
                    self.text_segment[idx] = bytes;
                }

                resolved_keys.push(key.clone());
            }
        }

        for key in resolved_keys {
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

        let address = u32::try_from(self.data_segment.len()).map_err(|_| {
            let message = format!(
                "Failed to convert data segment length to u32 for string emission. Data segment length exceeds {}. Found data segment length: {}.",
                u32::MAX,
                self.data_segment.len()
            );
            let _ = self.error_at_current(&message);
            Exception::Assembler(BaseException::new(message, None))
        })?;

        self.data_segment.extend(words);
        Ok(address)
    }

    fn emit_label(&mut self, key: String) -> Result<(), Exception> {
        self.emit_number(0);
        self.upsert_unresolved_label(key)
    }

    fn emit_padding(&mut self, words: usize) {
        for _ in 0..words {
            self.emit_number(0);
        }
    }

    fn validate_op_code(&mut self, op_code: OpCode) -> Result<(), Exception> {
        if op_code == OpCode::NoOp {
            self.error_at_current("Invalid opcode: NoOp is reserved for labels and placeholders and cannot be used in instructions.")?;
            Err(Exception::Assembler(BaseException::new(
                "Invalid opcode: NoOp is reserved for labels and placeholders and cannot be used in instructions.".to_string(),
                None,
            )))
        } else {
            Ok(())
        }
    }

    fn validate_role(&mut self, role: &str) -> Result<(), Exception> {
        if role.is_empty() {
            self.error_at_previous("Role name cannot be empty.")?;
            return Err(Exception::Assembler(BaseException::new(
                "Role name cannot be empty.".to_string(),
                None,
            )));
        }

        let lower = role.to_lowercase();
        if !matches!(lower.as_str(), roles::USER_ROLE | roles::ASSISTANT_ROLE) {
            let message = format!(
                "Invalid role name '{}'. Expected '{}' or '{}'.",
                role,
                roles::USER_ROLE,
                roles::ASSISTANT_ROLE
            );
            self.error_at_previous(&message)?;
            return Err(Exception::Assembler(BaseException::new(message, None)));
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
        self.validate_op_code(op_code)?;

        if string_only && number_only {
            return Err(Exception::Assembler(BaseException::new(
                "An instruction cannot be both string-only and number-only.".to_string(),
                None,
            )));
        }

        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))?;

        let destination_register = self.register("Expected destination register.")?;
        self.consume(
            &TokenType::Comma,
            "Expected ',' after destination register.",
        )?;

        self.emit_opcode(op_code);
        self.emit_number(destination_register);

        if string_only {
            let string = self.string("Expected string after ','.")?;
            let pointer = self.emit_string(&string)?;
            self.emit_number(pointer);
            self.emit_padding(1);
        } else {
            let immediate = self.number("Expected number after ','.")?;
            self.emit_number(immediate);
            self.emit_padding(1);
        }

        Ok(())
    }

    fn branch(&mut self, token_type: &TokenType, op_code: OpCode) -> Result<(), Exception> {
        self.validate_op_code(op_code)?;

        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))?;

        let source_register_1 =
            self.register("Expected source register 1 after branch keyword.")?;
        self.consume(&TokenType::Comma, "Expected ',' after source register 1.")?;

        let source_register_2 = self.register("Expected source register 2 after ','.")?;
        self.consume(&TokenType::Comma, "Expected ',' after source register 2.")?;

        let label_name = self
            .identifier("Expected label name after ','.")?
            .to_string();

        self.emit_opcode(op_code);
        self.emit_number(source_register_1);
        self.emit_number(source_register_2);
        self.emit_label(label_name)
    }

    fn no_register(&mut self, token_type: &TokenType, op_code: OpCode) -> Result<(), Exception> {
        self.validate_op_code(op_code)?;
        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))?;

        self.emit_opcode(op_code);
        self.emit_padding(3);

        Ok(())
    }

    fn no_register_string(
        &mut self,
        token_type: &TokenType,
        op_code: OpCode,
    ) -> Result<(), Exception> {
        self.validate_op_code(op_code)?;
        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))?;

        let string = self.string("Expected string after keyword.")?;

        if op_code == OpCode::ContextSetRole {
            self.validate_role(&string)?;
        }

        let pointer = self.emit_string(&string)?;

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
        self.validate_op_code(op_code)?;
        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))?;

        let register = self.register(&format!("Expected register after '{:?}'.", op_code))?;

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
        self.validate_op_code(op_code)?;
        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))?;

        let destination_register = self.register(&format!(
            "Expected destination register after '{:?}'.",
            op_code
        ))?;
        self.consume(
            &TokenType::Comma,
            "Expected ',' after destination register.",
        )?;

        let source_register =
            self.register(&format!("Expected source register after '{:?}'.", op_code))?;

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
        self.validate_op_code(op_code)?;
        self.consume(token_type, &format!("Expected '{:?}' keyword.", token_type))?;

        let destination_register = self.register(&format!(
            "Expected destination register after '{:?}' keyword.",
            op_code
        ))?;
        self.consume(
            &TokenType::Comma,
            "Expected ',' after destination register.",
        )?;

        let source_register_1 = self.register("Expected source register 1 after ','.")?;
        self.consume(&TokenType::Comma, "Expected ',' after source register 1.")?;

        let source_register_2 = self.register("Expected source register 2 after ','.")?;

        self.emit_opcode(op_code);
        self.emit_number(destination_register);
        self.emit_number(source_register_1);
        self.emit_number(source_register_2);

        Ok(())
    }

    fn parse_instruction(&mut self, token_type: &TokenType) -> Result<(), Exception> {
        let op_code: OpCode = token_type.clone().into();

        match token_type {
            // Data movement.
            TokenType::LoadImmediate => self.immediate(token_type, op_code, false, false),
            TokenType::LoadString | TokenType::LoadFile => {
                self.immediate(token_type, op_code, true, false)
            }
            TokenType::Move => self.double_register(token_type, op_code),
            // Control flow.
            TokenType::BranchEqual
            | TokenType::BranchLess
            | TokenType::BranchLessEqual
            | TokenType::BranchGreater
            | TokenType::BranchGreaterEqual => self.branch(token_type, op_code),
            TokenType::Exit => self.no_register(token_type, op_code),
            TokenType::Label => self.label(),
            // I/O.
            TokenType::Out => self.single_register(token_type, op_code),
            // Generative, cognitive, and guardrails operations.
            TokenType::Map | TokenType::Eval => self.double_register(token_type, op_code),
            TokenType::Similarity => self.triple_register(token_type, op_code),
            // Context operations.
            TokenType::ContextClear | TokenType::ContextDrop => {
                self.no_register(token_type, op_code)
            }
            TokenType::ContextSnapshot
            | TokenType::ContextRestore
            | TokenType::ContextPush
            | TokenType::ContextPop => self.single_register(token_type, op_code),
            TokenType::ContextSetRole => self.no_register_string(token_type, op_code),
            // Misc operations.
            TokenType::Decrement => self.immediate(token_type, op_code, false, true),
            _ => self.error_at_current("Unexpected keyword."),
        }
    }

    pub fn assemble(&mut self) -> Result<Vec<u8>, Exception> {
        self.advance()?;

        while !self.panic_mode {
            let token_type = self
                .current
                .as_ref()
                .map(|token| token.token_type().clone())
                .unwrap_or(TokenType::Eof);

            if token_type == TokenType::Eof {
                break;
            }

            self.parse_instruction(&token_type)?;
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
        byte_code.push(HEADER_SIZE.to_be_bytes());

        let text_segment_size = u32::try_from(self.text_segment.len()).map_err(|_| {
            let message = format!(
                "Failed to convert text segment size to u32. Text segment size exceeds {}. Found text segment size: {}",
                u32::MAX,
                self.text_segment.len()
            );
            let _ = self.error_at_current(&message);
            Exception::Assembler(BaseException::new(message, None))
        })?;

        byte_code.push((HEADER_SIZE + text_segment_size).to_be_bytes());

        // Append the text segment.
        byte_code.extend(&self.text_segment);

        // Append the data segment after the text segment.
        byte_code.extend(&self.data_segment);

        Ok(byte_code.into_iter().flatten().collect())
    }
}
