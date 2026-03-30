use crate::{
    assembler::opcode::OpCode,
    exception::{BaseException, Exception},
    processor::{
        control_unit::instruction::{
            BranchInstruction, BranchType, ContextDropInstruction, ContextPopInstruction,
            ContextPushInstruction, EvalulateInstruction, ExitInstruction, InferenceInstruction,
            Instruction, LoadContentInstruction, LoadImmediateInstruction, LoadStringInstruction,
            MoveContextInstruction, MoveInstruction, PrintContextInstruction, PrintInstruction,
            PrintLineInstruction, SimilarityInstruction, SubtractImmediateInstruction,
        },
        memory::Memory,
        registers::Registers,
    },
};

pub struct Decoder;

impl Decoder {
    fn op_code(bytes: &[u8; 4]) -> Result<OpCode, Exception> {
        let value = u32::from_be_bytes(*bytes);
        OpCode::try_from(value).map_err(|e| {
            Exception::Decoder(BaseException::caused_by(
                format!("Failed to decode opcode: 0x{:08X}", value),
                e,
            ))
        })
    }

    fn string(
        memory: &Memory,
        registers: &Registers,
        pointer: usize,
        context: &str,
    ) -> Result<String, Exception> {
        let mut bytes = Vec::new();
        let mut address = pointer + registers.get_data_section_pointer();

        loop {
            let word = memory.read(address).map_err(|e| {
                Exception::Decoder(BaseException::caused_by(
                    format!("{}: failed to read byte at address {}", context, address),
                    e,
                ))
            })?;
            let value: u8 = u32::from_be_bytes(*word).try_into().map_err(|e| {
                Exception::Decoder(BaseException::caused_by(
                    format!(
                        "{}: value at address {} does not fit in a byte",
                        context, address
                    ),
                    format!("{}", e),
                ))
            })?;

            if value == 0 {
                return String::from_utf8(bytes).map_err(|e| {
                    Exception::Decoder(BaseException::caused_by(
                        format!("{}: invalid UTF-8 at address {}", context, address),
                        e.to_string(),
                    ))
                });
            }

            bytes.push(value);
            address += 1;
        }
    }

    fn immediate(
        memory: &Memory,
        registers: &Registers,
        op_code: OpCode,
        instruction_bytes: [[u8; 4]; 4],
    ) -> Result<Instruction, Exception> {
        let register = u32::from_be_bytes(instruction_bytes[1]);

        match op_code {
            OpCode::LoadString | OpCode::LoadContent => {
                let string_pointer = u32::from_be_bytes(instruction_bytes[2]) as usize;
                let string = Self::string(
                    memory,
                    registers,
                    string_pointer,
                    &format!("Decoding string for {:?}", op_code),
                )?;

                if op_code == OpCode::LoadString {
                    Ok(Instruction::LoadString(LoadStringInstruction {
                        destination_register: register,
                        value: string,
                    }))
                } else {
                    Ok(Instruction::LoadContent(LoadContentInstruction {
                        destination_register: register,
                        path: string,
                    }))
                }
            }
            OpCode::LoadImmediate => Ok(Instruction::LoadImmediate(LoadImmediateInstruction {
                destination_register: register,
                value: u32::from_be_bytes(instruction_bytes[2]),
            })),
            OpCode::Move => Ok(Instruction::Move(MoveInstruction {
                destination_register: register,
                source_register: u32::from_be_bytes(instruction_bytes[2]),
            })),
            OpCode::SubtractImmediate => Ok(Instruction::SubtractImmediate(
                SubtractImmediateInstruction {
                    source_register: register,
                    value: u32::from_be_bytes(instruction_bytes[2]),
                },
            )),
            _ => Err(Exception::Decoder(BaseException::new(
                format!(
                    "Failed to decode immediate instruction: invalid opcode '{:?}'.",
                    op_code
                ),
                None,
            ))),
        }
    }

    fn branch(op_code: OpCode, instruction_bytes: [[u8; 4]; 4]) -> Result<Instruction, Exception> {
        let source_register_1 = u32::from_be_bytes(instruction_bytes[1]);
        let source_register_2 = u32::from_be_bytes(instruction_bytes[2]);
        let instruction_pointer_jump_index = u32::from_be_bytes(instruction_bytes[3]);

        let branch_type = match op_code {
            OpCode::BranchEqual => BranchType::Equal,
            OpCode::BranchLess => BranchType::Less,
            OpCode::BranchLessEqual => BranchType::LessEqual,
            OpCode::BranchGreater => BranchType::Greater,
            OpCode::BranchGreaterEqual => BranchType::GreaterEqual,
            _ => {
                return Err(Exception::Decoder(BaseException::new(
                    format!(
                        "Failed to decode branch instruction: invalid opcode '{:?}'.",
                        op_code
                    ),
                    None,
                )));
            }
        };

        Ok(Instruction::Branch(BranchInstruction {
            branch_type,
            source_register_1,
            source_register_2,
            instruction_pointer_jump_index,
        }))
    }

    fn no_register(op_code: OpCode) -> Result<Instruction, Exception> {
        match op_code {
            // Control flow.
            OpCode::Exit => Ok(Instruction::Exit(ExitInstruction)),
            _ => Err(Exception::Decoder(BaseException::new(
                format!(
                    "Failed to decode zero-register instruction: invalid opcode '{:?}'.",
                    op_code
                ),
                None,
            ))),
        }
    }

    fn single_register(
        op_code: OpCode,
        instruction_bytes: [[u8; 4]; 4],
    ) -> Result<Instruction, Exception> {
        let register = u32::from_be_bytes(instruction_bytes[1]);

        match op_code {
            // I/O.
            OpCode::Print => Ok(Instruction::Print(PrintInstruction {
                source_register: register,
            })),
            OpCode::PrintLine => Ok(Instruction::PrintLine(PrintLineInstruction {
                source_register: register,
            })),
            OpCode::PrintContext => Ok(Instruction::PrintContext(PrintContextInstruction {
                source_context_register: register,
            })),
            // Context operations.
            OpCode::ContextDrop => Ok(Instruction::ContextDrop(ContextDropInstruction {
                source_context_register: register,
            })),
            _ => Err(Exception::Decoder(BaseException::new(
                format!(
                    "Failed to decode single-register instruction: invalid opcode '{:?}'.",
                    op_code
                ),
                None,
            ))),
        }
    }

    fn double_register(
        op_code: OpCode,
        instruction_bytes: [[u8; 4]; 4],
    ) -> Result<Instruction, Exception> {
        let destination_register = u32::from_be_bytes(instruction_bytes[1]);
        let source_register = u32::from_be_bytes(instruction_bytes[2]);

        match op_code {
            OpCode::ContextPop => Ok(Instruction::ContextPop(ContextPopInstruction {
                destination_register,
                source_context_register: source_register,
            })),
            OpCode::MoveContext => Ok(Instruction::MoveContext(MoveContextInstruction {
                destination_context_register: destination_register,
                source_context_register: source_register,
            })),
            _ => Err(Exception::Decoder(BaseException::new(
                format!(
                    "Failed to decode double-register instruction: invalid opcode '{:?}'.",
                    op_code
                ),
                None,
            ))),
        }
    }

    fn double_register_string(
        memory: &Memory,
        registers: &Registers,
        op_code: OpCode,
        instruction_bytes: [[u8; 4]; 4],
    ) -> Result<Instruction, Exception> {
        let destination_context_register = u32::from_be_bytes(instruction_bytes[1]);
        let source_register = u32::from_be_bytes(instruction_bytes[2]);
        let string_pointer = u32::from_be_bytes(instruction_bytes[3]) as usize;

        let string = Self::string(
            memory,
            registers,
            string_pointer,
            &format!("Decoding role string for {:?}", op_code),
        )?;

        match op_code {
            OpCode::ContextPush => Ok(Instruction::ContextPush(ContextPushInstruction {
                destination_context_register,
                source_register,
                role: string,
            })),
            _ => Err(Exception::Decoder(BaseException::new(
                format!(
                    "Failed to decode double-register-string instruction: invalid opcode '{:?}'.",
                    op_code
                ),
                None,
            ))),
        }
    }

    fn triple_register(
        op_code: OpCode,
        instruction_bytes: [[u8; 4]; 4],
    ) -> Result<Instruction, Exception> {
        let destination_register = u32::from_be_bytes(instruction_bytes[1]);
        let source_register_1 = u32::from_be_bytes(instruction_bytes[2]);
        let source_register_2 = u32::from_be_bytes(instruction_bytes[3]);

        match op_code {
            OpCode::Inference => Ok(Instruction::Inference(InferenceInstruction {
                destination_register,
                source_register: source_register_1,
                context_register: source_register_2,
            })),
            OpCode::Evaluate => Ok(Instruction::Evaluate(EvalulateInstruction {
                destination_register,
                source_register: source_register_1,
                context_register: source_register_2,
            })),
            OpCode::Similarity => Ok(Instruction::Similarity(SimilarityInstruction {
                destination_register,
                source_register_1,
                source_register_2,
            })),
            _ => Err(Exception::Decoder(BaseException::new(
                format!(
                    "Failed to decode triple-register instruction: invalid opcode '{:?}'.",
                    op_code
                ),
                None,
            ))),
        }
    }

    pub fn decode(
        memory: &Memory,
        registers: &Registers,
        instruction_bytes: [[u8; 4]; 4],
    ) -> Result<Instruction, Exception> {
        let op_code = Self::op_code(&instruction_bytes[0])?;

        if op_code == OpCode::NoOp {
            return Err(Exception::Decoder(BaseException::new(
                "NoOp is not a valid instruction and should not be decoded.".to_string(),
                None,
            )));
        }

        match op_code {
            // Data movement.
            OpCode::LoadString
            | OpCode::LoadImmediate
            | OpCode::LoadContent
            | OpCode::Move
            | OpCode::SubtractImmediate => {
                Self::immediate(memory, registers, op_code, instruction_bytes)
            }
            // Control flow.
            OpCode::BranchEqual
            | OpCode::BranchLess
            | OpCode::BranchLessEqual
            | OpCode::BranchGreater
            | OpCode::BranchGreaterEqual => Self::branch(op_code, instruction_bytes),
            OpCode::Exit => Self::no_register(op_code),
            // I/O.
            OpCode::Print | OpCode::PrintLine | OpCode::PrintContext | OpCode::ContextDrop => {
                Self::single_register(op_code, instruction_bytes)
            }
            // Context operations.
            OpCode::ContextPush => {
                Self::double_register_string(memory, registers, op_code, instruction_bytes)
            }
            OpCode::ContextPop | OpCode::MoveContext => {
                Self::double_register(op_code, instruction_bytes)
            }
            // Generative, cognitive, and guardrails operations.
            OpCode::Inference | OpCode::Evaluate | OpCode::Similarity => {
                Self::triple_register(op_code, instruction_bytes)
            }
            OpCode::NoOp => unreachable!(),
        }
    }
}
