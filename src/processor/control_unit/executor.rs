use std::fs::read_to_string;

use crate::{
    assembler::roles,
    exception::{BaseException, Exception},
    processor::{
        control_unit::{
            instruction::{
                BranchInstruction, BranchType, ContextPopInstruction, ContextPushInstruction,
                ContextRestoreInstruction, ContextSetRoleInstruction, ContextSnapshotInstruction,
                DecrementInstruction, EvalInstruction, Instruction, LoadFileInstruction,
                LoadImmediateInstruction, LoadStringInstruction, MapInstruction, MoveInstruction,
                OutputInstruction, SimilarityInstruction,
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
        match registers.get_register(register_number)? {
            Value::Text(text) => Ok(text),
            Value::None => Err(Exception::Executor(BaseException::new(
                format!(
                    "Register r{} is uninitialised, expected text.",
                    register_number
                ),
                None,
            ))),
            other => Err(Exception::Executor(BaseException::new(
                format!(
                    "Register r{} contains {:?}, expected text.",
                    register_number, other
                ),
                None,
            ))),
        }
    }

    fn read_number(registers: &Registers, register_number: u32) -> Result<u32, Exception> {
        match registers.get_register(register_number)? {
            Value::Number(number) => Ok(*number),
            Value::None => Err(Exception::Executor(BaseException::new(
                format!(
                    "Register r{} is uninitialised, expected number.",
                    register_number
                ),
                None,
            ))),
            other => Err(Exception::Executor(BaseException::new(
                format!(
                    "Register r{} contains {:?}, expected number.",
                    register_number, other
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
        registers.set_register(instruction.destination_register, &value)?;

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
        registers.set_register(instruction.destination_register, &value)?;

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
        let file_contents = read_to_string(&instruction.file_path).map_err(|e| {
            Exception::Executor(BaseException::new(
                format!("Failed to read file '{}'", instruction.file_path),
                Some(Box::new(e.into())),
            ))
        })?;

        registers.set_register(
            instruction.destination_register,
            &Value::Text(file_contents.clone()),
        )?;

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
        let value = registers.get_register(instruction.source_register)?.clone();
        registers.set_register(instruction.destination_register, &value)?;

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
        let value_a = Self::read_number(registers, instruction.source_register_1)?;
        let value_b = Self::read_number(registers, instruction.source_register_2)?;

        let is_true = match instruction.branch_type {
            BranchType::Equal => value_a == value_b,
            BranchType::Less => value_a < value_b,
            BranchType::LessEqual => value_a <= value_b,
            BranchType::Greater => value_a > value_b,
            BranchType::GreaterEqual => value_a >= value_b,
        };

        if is_true {
            let pointer =
                usize::try_from(instruction.instruction_pointer_jump_index).map_err(|e| {
                    Exception::Executor(BaseException::new(
                        "Invalid branch jump index".to_string(),
                        Some(Box::new(e.to_string().into())),
                    ))
                })?;
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
        let value = registers.get_register(instruction.source_register)?.clone();

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

    fn map(
        registers: &mut Registers,
        instruction: &MapInstruction,
        text_model: &str,
        debug: bool,
        debug_chat: bool,
    ) -> Result<(), Exception> {
        let value = Self::read_text(registers, instruction.source_register)?.clone();
        let context = registers.get_context();
        let result = LanguageLogicUnit::string(&value, context, text_model, debug_chat)?;

        crate::debug_print!(
            debug,
            "Executed MAP : r{} = '{:?}'",
            instruction.destination_register,
            result
        );

        registers.set_register(instruction.destination_register, &Value::Text(result))
    }

    fn eval(
        registers: &mut Registers,
        instruction: &EvalInstruction,
        text_model: &str,
        embedding_model: &str,
        debug: bool,
        debug_chat: bool,
    ) -> Result<(), Exception> {
        let value = Self::read_text(registers, instruction.source_register)?.clone();
        let micro_prompt = format!(
            "{}\nAnswer with exactly one word: YES or NO, TRUE or FALSE.\n\nAnswer only:",
            value
        );
        let true_values = vec!["YES", "TRUE"];
        let false_values = vec!["NO", "FALSE"];
        let context = registers.get_context();

        let result = LanguageLogicUnit::boolean(
            &micro_prompt,
            &true_values,
            &false_values,
            context,
            text_model,
            embedding_model,
            debug_chat,
        )?;

        crate::debug_print!(
            debug,
            "Executed EVAL: r{} = '{:?}'",
            instruction.destination_register,
            result
        );

        registers.set_register(instruction.destination_register, &Value::Number(result))
    }

    fn similarity(
        registers: &mut Registers,
        instruction: &SimilarityInstruction,
        embedding_model: &str,
        debug: bool,
    ) -> Result<(), Exception> {
        let value_a = Self::read_text(registers, instruction.source_register_1)?.clone();
        let value_b = Self::read_text(registers, instruction.source_register_2)?.clone();

        let result = LanguageLogicUnit::cosine_similarity(&value_a, &value_b, embedding_model)?;

        crate::debug_print!(
            debug,
            "Executed SIM : '{:?}' vs '{:?}' -> r{} = {}",
            value_a,
            value_b,
            instruction.destination_register,
            result
        );

        registers.set_register(instruction.destination_register, &Value::Number(result))
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
        registers.set_register(instruction.destination_register, &Value::Text(snapshot))?;

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
        let snapshot = Self::read_text(registers, instruction.source_register)?.clone();
        registers.restore_context(&snapshot)?;

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
        let value = match registers.get_register(instruction.source_register)? {
            Value::Text(text) => text.clone(),
            Value::Number(number) => number.to_string(),
            Value::None => {
                return Err(Exception::Executor(BaseException::new(
                    format!(
                        "Register r{} is uninitialised, expected text or number.",
                        instruction.source_register
                    ),
                    None,
                )));
            }
        };
        let role = registers
            .get_context_role()
            .unwrap_or(roles::USER_ROLE.to_string());

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
        let context = registers.pop_context().ok_or_else(|| {
            Exception::Executor(BaseException::new(
                "Cannot pop from empty context stack.".to_string(),
                None,
            ))
        })?;

        registers.set_register(
            instruction.destination_register,
            &Value::Text(context.content.clone()),
        )?;

        crate::debug_print!(debug, "Executed POP : Popped value from context stack.",);

        Ok(())
    }

    fn context_drop(registers: &mut Registers, debug: bool) -> Result<(), Exception> {
        registers.pop_context().ok_or_else(|| {
            Exception::Executor(BaseException::new(
                "Cannot drop from empty context stack.".to_string(),
                None,
            ))
        })?;

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
        let value = Self::read_number(registers, instruction.source_register)?;

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
        registers.set_register(instruction.source_register, &new_value)?;

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
        debug_chat: bool,
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
            Instruction::Map(i) => Self::map(registers, i, text_model, debug, debug_chat),
            // Guardrails operations.
            Instruction::Eval(i) => {
                Self::eval(registers, i, text_model, embedding_model, debug, debug_chat)
            }
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
