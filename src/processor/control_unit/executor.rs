use std::fs::read_to_string;

use crate::{
    assembler::roles,
    exception::{BaseException, Exception},
    processor::{
        control_unit::{
            instruction::{
                AuditInstruction, BranchInstruction, BranchType, ContextPopInstruction,
                ContextPushInstruction, ContextRestoreInstruction, ContextSetRoleInstruction,
                ContextSnapshotInstruction, CorrelateInstruction, DecrementInstruction,
                DistillInstruction, Instruction, LoadFileInstruction, LoadImmediateInstruction,
                LoadStringInstruction, MorphInstruction, MoveInstruction, OutputInstruction,
                ProjectInstruction, SimilarityInstruction,
            },
            language_logic_unit::LanguageLogicUnit,
        },
        memory::Memory,
        registers::{ContextMessage, Registers, Value},
    },
};

pub struct Executor;

impl Executor {
    fn read_text(registers: &Registers, register_number: u32) -> Result<&String, Exception> {
        let value = match registers.get_register(register_number) {
            Ok(value) => value,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read text from register.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        match value {
            Value::Text(text) => Ok(text),
            Value::None => Err(Exception::Executor(BaseException::new(
                format!(
                    "Register r{} is uninitialised, expected text.",
                    register_number
                ),
                None,
            ))),
            _ => Err(Exception::Executor(BaseException::new(
                format!(
                    "Register r{} contains {:?}, expected text.",
                    register_number, value
                ),
                None,
            ))),
        }
    }

    fn read_number(registers: &Registers, register_number: u32) -> Result<u32, Exception> {
        let value = match registers.get_register(register_number) {
            Ok(value) => value,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read number from register.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        match value {
            Value::Number(number) => Ok(*number),
            Value::None => Err(Exception::Executor(BaseException::new(
                format!(
                    "Register r{} is uninitialised, expected number.",
                    register_number
                ),
                None,
            ))),
            _ => Err(Exception::Executor(BaseException::new(
                format!(
                    "Register r{} contains {:?}, expected number.",
                    register_number, value
                ),
                None,
            ))),
        }
    }

    fn load_string(
        registers: &mut Registers,
        instruction: &LoadStringInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = Value::Text(instruction.value.clone());

        match registers.set_register(instruction.destination_register, &value) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to set destination register for load string instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        crate::debug_print!(
            debug,
            "Executed LI  : r{} = {:?}",
            instruction.destination_register,
            value
        );

        Ok(())
    }

    fn load_immediate(
        registers: &mut Registers,
        instruction: &LoadImmediateInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = Value::Number(instruction.value);

        match registers.set_register(instruction.destination_register, &value) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to set destination register for load immediate instruction."
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        crate::debug_print!(
            debug,
            "Executed LI  : r{} = {:?}",
            instruction.destination_register,
            value
        );

        Ok(())
    }

    fn load_file(
        registers: &mut Registers,
        instruction: &LoadFileInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let file_contents = match read_to_string(&instruction.file_path) {
            Ok(contents) => contents,
            Err(error) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read file for load file instruction.".to_string(),
                    Some(Box::new(error.into())),
                )));
            }
        };

        match registers.set_register(
            instruction.destination_register,
            &Value::Text(file_contents.clone()),
        ) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to set destination register for load file instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        crate::debug_print!(
            debug,
            "Executed LF  : r{} = {:?}",
            instruction.destination_register,
            file_contents
        );

        Ok(())
    }

    fn mov(
        registers: &mut Registers,
        instruction: &MoveInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = match registers.get_register(instruction.source_register) {
            Ok(value) => value.clone(),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read source register for move instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        match registers.set_register(instruction.destination_register, &value) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to set destination register for move instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        crate::debug_print!(
            debug,
            "Executed MOV : r{} = {}",
            instruction.destination_register,
            value
        );

        Ok(())
    }

    fn branch(
        registers: &mut Registers,
        instruction: &BranchInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value_a = match Self::read_number(registers, instruction.source_register_1) {
            Ok(value) => value,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read number from source register 1 for branch instruction."
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };
        let value_b = match Self::read_number(registers, instruction.source_register_2) {
            Ok(value) => value,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read number from source register 2 for branch instruction."
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        let is_true = match instruction.branch_type {
            BranchType::Equal => value_a == value_b,
            BranchType::Less => value_a < value_b,
            BranchType::LessEqual => value_a <= value_b,
            BranchType::Greater => value_a > value_b,
            BranchType::GreaterEqual => value_a >= value_b,
        };

        if is_true {
            let pointer = match usize::try_from(instruction.instruction_pointer_jump_index) {
                Ok(index) => index,
                Err(error) => {
                    return Err(Exception::Executor(BaseException::new(
                        "Failed to convert instruction pointer jump index to usize for branch instruction."
                            .to_string(),
                        Some(Box::new(error.to_string().into())),
                    )));
                }
            };

            registers.set_instruction_pointer(pointer);
        }

        crate::debug_print!(
            debug,
            "Executed {} : {:?} {:?} -> {} jump {}",
            match instruction.branch_type {
                BranchType::Equal => "BEQ",
                BranchType::Less => "BLT",
                BranchType::LessEqual => "BLE",
                BranchType::Greater => "BGT",
                BranchType::GreaterEqual => "BGE",
            },
            value_a,
            value_b,
            is_true,
            instruction.instruction_pointer_jump_index
        );

        Ok(())
    }

    fn exit(memory: &Memory, registers: &mut Registers, debug: bool) {
        crate::debug_print!(debug, "Executed EXIT: Halting execution.");
        registers.set_instruction_pointer(memory.length());
    }

    fn output(
        registers: &Registers,
        instruction: &OutputInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = match registers.get_register(instruction.source_register) {
            Ok(value) => value.clone(),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read source register for output instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        crate::debug_print!(
            debug,
            "Executed OUT : r{} = {:?}",
            instruction.source_register,
            value
        );

        if !debug {
            println!("{}", value);
        }

        Ok(())
    }

    fn morph(
        registers: &mut Registers,
        instruction: &MorphInstruction,
        text_model: &str,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = match Self::read_text(registers, instruction.source_register) {
            Ok(value) => value.clone(),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read text from source register for morph instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };
        let micro_prompt = format!(
            "Rewrite to exactly match this template:\n{}\n\nAnswer only:",
            value
        );
        let context = registers.get_context();

        let result = match LanguageLogicUnit::string(&micro_prompt, context, text_model) {
            Ok(result) => result,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to perform morph operation.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        crate::debug_print!(
            debug,
            "Executed MRF : r{} = '{:?}'",
            instruction.destination_register,
            result
        );

        match registers.set_register(instruction.destination_register, &Value::Text(result)) {
            Ok(_) => Ok(()),
            Err(exception) => Err(Exception::Executor(BaseException::new(
                "Failed to set destination register for morph instruction.".to_string(),
                Some(Box::new(exception)),
            ))),
        }
    }

    fn project(
        registers: &mut Registers,
        instruction: &ProjectInstruction,
        text_model: &str,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = match Self::read_text(registers, instruction.source_register) {
            Ok(value) => value.clone(),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read text from source register for project instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };
        let micro_prompt = format!("What happens next if:\n{}\n\nPrediction only:", value);
        let context = registers.get_context();

        let result = match LanguageLogicUnit::string(&micro_prompt, context, text_model) {
            Ok(result) => result,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to perform project operation.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        crate::debug_print!(
            debug,
            "Executed PRJ : r{} = '{:?}'",
            instruction.destination_register,
            result
        );

        match registers.set_register(instruction.destination_register, &Value::Text(result)) {
            Ok(_) => Ok(()),
            Err(exception) => Err(Exception::Executor(BaseException::new(
                "Failed to set destination register for project instruction.".to_string(),
                Some(Box::new(exception)),
            ))),
        }
    }

    fn distill(
        registers: &mut Registers,
        instruction: &DistillInstruction,
        text_model: &str,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = match Self::read_text(registers, instruction.source_register) {
            Ok(value) => value.clone(),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read text from source register for distill instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };
        let micro_prompt = format!(
            "Extract only the exact information here:\n{}\n\nShort answer only:",
            value
        );
        let context = registers.get_context();

        let result = match LanguageLogicUnit::string(&micro_prompt, context, text_model) {
            Ok(result) => result,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to perform distill operation.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        crate::debug_print!(
            debug,
            "Executed DST : r{} = '{:?}'",
            instruction.destination_register,
            result
        );

        match registers.set_register(instruction.destination_register, &Value::Text(result)) {
            Ok(_) => Ok(()),
            Err(exception) => Err(Exception::Executor(BaseException::new(
                "Failed to set destination register for distill instruction.".to_string(),
                Some(Box::new(exception)),
            ))),
        }
    }

    fn correlate(
        registers: &mut Registers,
        instruction: &CorrelateInstruction,
        text_model: &str,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = match Self::read_text(registers, instruction.source_register) {
            Ok(value) => value.clone(),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read text from source register for correlate instruction."
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };
        let micro_prompt = format!(
            "Compare with:\n{}\nHow are they similar or different?\n\nAnswer only:",
            value
        );
        let context = registers.get_context();

        let result = match LanguageLogicUnit::string(&micro_prompt, context, text_model) {
            Ok(result) => result,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to perform correlate operation.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        crate::debug_print!(
            debug,
            "Executed CORR : r{} = '{:?}'",
            instruction.destination_register,
            result
        );

        match registers.set_register(instruction.destination_register, &Value::Text(result)) {
            Ok(_) => Ok(()),
            Err(exception) => Err(Exception::Executor(BaseException::new(
                "Failed to set destination register for correlate instruction.".to_string(),
                Some(Box::new(exception)),
            ))),
        }
    }

    fn audit(
        registers: &mut Registers,
        instruction: &AuditInstruction,
        text_model: &str,
        embedding_model: &str,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = match Self::read_text(registers, instruction.source_register) {
            Ok(value) => value.clone(),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read text from source register for audit instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };
        let micro_prompt = format!(
            "Does the it follow the rule:\n{}\nAnswer with exactly one word: YES or NO.\n\nAnswer only:",
            value
        );
        let true_values = vec!["YES"];
        let false_values = vec!["NO"];
        let context = registers.get_context();

        let result = match LanguageLogicUnit::boolean(
            &micro_prompt,
            &true_values,
            &false_values,
            context,
            text_model,
            embedding_model,
        ) {
            Ok(result) => result,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to perform audit operation.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        crate::debug_print!(
            debug,
            "Executed AUD : r{} = '{:?}'",
            instruction.destination_register,
            result
        );

        match registers.set_register(instruction.destination_register, &Value::Number(result)) {
            Ok(_) => Ok(()),
            Err(exception) => Err(Exception::Executor(BaseException::new(
                "Failed to set destination register for audit instruction.".to_string(),
                Some(Box::new(exception)),
            ))),
        }
    }

    fn similarity(
        registers: &mut Registers,
        instruction: &SimilarityInstruction,
        embedding_model: &str,
        debug: bool,
    ) -> Result<(), Exception> {
        let value_a = match Self::read_text(registers, instruction.source_register_1) {
            Ok(value) => value.clone(),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read text from source register 1 for similarity instruction."
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };
        let value_b = match Self::read_text(registers, instruction.source_register_2) {
            Ok(value) => value.clone(),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read text from source register 2 for similarity instruction."
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        let result = match LanguageLogicUnit::cosine_similarity(&value_a, &value_b, embedding_model)
        {
            Ok(result) => result,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to perform similarity operation.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        crate::debug_print!(
            debug,
            "Executed SIM : '{:?}' vs '{:?}' -> r{} = {}",
            value_a,
            value_b,
            instruction.destination_register,
            result
        );

        match registers.set_register(instruction.destination_register, &Value::Number(result)) {
            Ok(_) => Ok(()),
            Err(exception) => Err(Exception::Executor(BaseException::new(
                "Failed to set destination register for similarity instruction.".to_string(),
                Some(Box::new(exception)),
            ))),
        }
    }

    fn context_clear(registers: &mut Registers, debug: bool) {
        registers.clear_context();

        crate::debug_print!(debug, "Executed CLR : Cleared context stack.");
    }

    fn context_snapshot(
        registers: &mut Registers,
        instruction: &ContextSnapshotInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let snapshot = registers.snapshot_context();

        match registers.set_register(instruction.destination_register, &Value::Text(snapshot)) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to set destination register for context snapshot instruction."
                        .to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        crate::debug_print!(
            debug,
            "Executed SNP : Snapshotted context stack into r{}.",
            instruction.destination_register
        );

        Ok(())
    }

    fn context_restore(
        registers: &mut Registers,
        instruction: &ContextRestoreInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let snapshot = match Self::read_text(registers, instruction.source_register) {
            Ok(value) => value.clone(),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read source register for context restore instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        match registers.restore_context(&snapshot) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to restore context from snapshot.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        crate::debug_print!(
            debug,
            "Executed RST : Restored context stack from snapshot in r{}.",
            instruction.source_register
        );

        Ok(())
    }

    fn context_push(
        registers: &mut Registers,
        instruction: &ContextPushInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = match registers.get_register(instruction.source_register) {
            Ok(Value::Text(text)) => text.clone(),
            Ok(Value::Number(number)) => number.to_string(),
            Ok(Value::None) => {
                return Err(Exception::Executor(BaseException::new(
                    format!(
                        "Register r{} is uninitialised, expected text or number.",
                        instruction.source_register
                    ),
                    None,
                )));
            }
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    format!(
                        "Failed to read register r{} for context push instruction.",
                        instruction.source_register
                    ),
                    Some(Box::new(exception)),
                )));
            }
        };
        let role = registers
            .get_context_role()
            .unwrap_or(roles::USER_ROLE.to_string())
            .to_string();

        registers.push_context(ContextMessage::new(&role, &value));

        crate::debug_print!(
            debug && registers.get_context_role().is_none(),
            "Defaulting context role to '{}' for CONTEXT_PUSH since no role is currently set.",
            roles::USER_ROLE
        );
        crate::debug_print!(
            debug,
            "Executed PSH : Pushed value from r{} onto context stack.",
            instruction.source_register
        );

        Ok(())
    }

    fn context_pop(
        registers: &mut Registers,
        instruction: &ContextPopInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let context = match registers.pop_context() {
            Some(ctx) => ctx,
            None => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to pop context because context stack is uninitialised.".to_string(),
                    None,
                )));
            }
        };

        match registers.set_register(
            instruction.destination_register,
            &Value::Text(context.content.clone()),
        ) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to set register for CONTEXT_POP instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        crate::debug_print!(debug, "Executed POP : Popped value from context stack.",);

        Ok(())
    }

    fn context_drop(registers: &mut Registers, debug: bool) -> Result<(), Exception> {
        match registers.pop_context() {
            Some(_) => (),
            None => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to pop context because context stack is uninitialised.".to_string(),
                    None,
                )));
            }
        }

        crate::debug_print!(debug, "Executed DRP : Dropped value from context stack.",);

        Ok(())
    }

    fn context_set_role(
        registers: &mut Registers,
        instruction: &ContextSetRoleInstruction,
        debug: bool,
    ) {
        registers.set_context_role(&instruction.role);

        crate::debug_print!(
            debug,
            "Executed SRL : Set context role to '{}'.",
            instruction.role
        );
    }

    fn decrement(
        registers: &mut Registers,
        instruction: &DecrementInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = match Self::read_number(registers, instruction.source_register) {
            Ok(value) => value,
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to read number from register for decrement instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        if value < instruction.value {
            return Err(Exception::Executor(BaseException::new(
                format!(
                    "Cannot decrement register r{} by {} because it would result in a negative value.",
                    instruction.source_register, instruction.value
                ),
                None,
            )));
        }

        let new_value = Value::Number(value - instruction.value);

        match registers.set_register(instruction.source_register, &new_value) {
            Ok(_) => (),
            Err(exception) => {
                return Err(Exception::Executor(BaseException::new(
                    "Failed to set register for decrement instruction.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        }

        crate::debug_print!(
            debug,
            "Executed DEC : Decremented r{} from {} to {}.",
            instruction.source_register,
            value,
            new_value
        );

        Ok(())
    }

    pub fn execute(
        memory: &mut Memory,
        registers: &mut Registers,
        instruction: &Instruction,
        text_model: &str,
        embedding_model: &str,
        debug: bool,
    ) -> Result<(), Exception> {
        match instruction {
            // Data movement operations.
            Instruction::LoadString(i) => Self::load_string(registers, i, debug),
            Instruction::LoadImmediate(i) => Self::load_immediate(registers, i, debug),
            Instruction::LoadFile(i) => Self::load_file(registers, i, debug),
            Instruction::Move(i) => Self::mov(registers, i, debug),
            // Control flow operations.
            Instruction::Branch(i) => Self::branch(registers, i, debug),
            Instruction::Exit(_) => {
                Self::exit(memory, registers, debug);
                Ok(())
            }
            // I/O operations.
            Instruction::Output(i) => Self::output(registers, i, debug),
            // Generative operations.
            Instruction::Morph(i) => Self::morph(registers, i, text_model, debug),
            Instruction::Project(i) => Self::project(registers, i, text_model, debug),
            // Cognitive operations.
            Instruction::Distill(i) => Self::distill(registers, i, text_model, debug),
            Instruction::Correlate(i) => Self::correlate(registers, i, text_model, debug),
            // Guardrails operations.
            Instruction::Audit(i) => Self::audit(registers, i, text_model, embedding_model, debug),
            Instruction::Similarity(i) => Self::similarity(registers, i, embedding_model, debug),
            // Context operations.
            Instruction::ContextClear(_) => {
                Self::context_clear(registers, debug);
                Ok(())
            }
            Instruction::ContextSnapshot(i) => Self::context_snapshot(registers, i, debug),
            Instruction::ContextRestore(i) => Self::context_restore(registers, i, debug),
            Instruction::ContextPush(i) => Self::context_push(registers, i, debug),
            Instruction::ContextPop(i) => Self::context_pop(registers, i, debug),
            Instruction::ContextDrop(_) => Self::context_drop(registers, debug),
            Instruction::ContextSetRole(i) => {
                Self::context_set_role(registers, i, debug);
                Ok(())
            }
            // Misc operations.
            Instruction::Decrement(i) => Self::decrement(registers, i, debug),
        }
    }
}
