use std::fs::read_to_string;

use crate::{
    config::{Config, TextModelOverrides},
    exception::{BaseException, Exception},
    processor::{
        control_unit::{
            instruction::{
                BranchInstruction, BranchType, ContextDropInstruction, ContextPopInstruction,
                ContextPushInstruction, EvalulateInstruction, InferenceInstruction, Instruction,
                LoadContentInstruction, LoadImmediateInstruction, LoadStringInstruction,
                MoveContextInstruction, MoveInstruction, PrintContextInstruction, PrintInstruction,
                PrintLineInstruction, SimilarityInstruction, SubtractImmediateInstruction,
            },
            language_logic_unit::{BooleanEvalParams, LanguageLogicUnit},
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

    fn load_content(
        registers: &mut Registers,
        instruction: &LoadContentInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let file_contents = read_to_string(&instruction.path).map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                format!("Failed to read file '{}'", instruction.path),
                e,
            ))
        })?;

        registers.set_register(
            instruction.destination_register,
            &Value::Text(file_contents.clone()),
        )?;

        crate::debug_print!(
            debug,
            "Executed LC  : r{} = {:?}",
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
            "Executed MV  : r{} = {}",
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
                    Exception::Executor(BaseException::caused_by(
                        "Invalid branch jump index",
                        e.to_string(),
                    ))
                })?;
            registers.set_instruction_pointer(pointer);
        }

        let label = match instruction.branch_type {
            BranchType::Equal => "BEQ",
            BranchType::Less => "BLT",
            BranchType::LessEqual => "BLE",
            BranchType::Greater => "BGT",
            BranchType::GreaterEqual => "BGE",
        };

        crate::debug_print!(
            debug,
            "Executed {} : {:?} {:?} -> {} jump {}",
            label,
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

    fn print(
        registers: &Registers,
        instruction: &PrintInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = registers.get_register(instruction.source_register)?.clone();

        crate::debug_print!(
            debug,
            "Executed PUT : r{} = {:?}",
            instruction.source_register,
            value
        );

        if !debug {
            print!("{}", value);
        }

        Ok(())
    }

    fn print_line(
        registers: &Registers,
        instruction: &PrintLineInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = registers.get_register(instruction.source_register)?.clone();

        crate::debug_print!(
            debug,
            "Executed PLN : r{} = {:?}",
            instruction.source_register,
            value
        );

        if !debug {
            println!("{}", value);
        }

        Ok(())
    }

    fn print_context(
        registers: &Registers,
        instruction: &PrintContextInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let context = registers.get_context(instruction.source_context_register)?;

        crate::debug_print!(
            debug,
            "Executed PCX : c{} = {:?}",
            instruction.source_context_register,
            context
        );

        if !debug {
            let context_json = miniserde::json::to_string(&context);
            println!("{}", context_json);
        }

        Ok(())
    }

    fn inference(
        registers: &mut Registers,
        instruction: &InferenceInstruction,
        text_model: &str,
        text_model_overrides: &TextModelOverrides,
        debug: bool,
        debug_chat: bool,
    ) -> Result<(), Exception> {
        let value = Self::read_text(registers, instruction.source_register)?.clone();
        let context = registers.get_context(instruction.context_register)?;
        let result = LanguageLogicUnit::string(
            &value,
            context,
            text_model,
            text_model_overrides,
            debug_chat,
        )?;

        crate::debug_print!(
            debug,
            "Executed INF : r{} = '{:?}'",
            instruction.destination_register,
            result
        );

        registers.set_register(instruction.destination_register, &Value::Text(result))
    }

    fn evaluate(
        registers: &mut Registers,
        instruction: &EvalulateInstruction,
        text_model: &str,
        embedding_model: &str,
        text_model_overrides: &TextModelOverrides,
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
        let context = registers.get_context(instruction.context_register)?;

        let eval_params = BooleanEvalParams {
            true_values: &true_values,
            false_values: &false_values,
            embedding_model,
        };

        let result = LanguageLogicUnit::boolean(
            &micro_prompt,
            &eval_params,
            context,
            text_model,
            text_model_overrides,
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

    fn context_push(
        registers: &mut Registers,
        instruction: &ContextPushInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let register_value = registers.get_register(instruction.source_register)?;

        let value = match register_value {
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

        registers.push_context(
            ContextMessage::new(&instruction.role, &value),
            instruction.destination_context_register,
        )?;

        crate::debug_print!(
            debug,
            "Executed PSH : Pushed value from r{} onto context stack with role '{}'.",
            instruction.source_register,
            instruction.role
        );

        Ok(())
    }

    fn context_pop(
        registers: &mut Registers,
        instruction: &ContextPopInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let context = registers.pop_context(instruction.source_context_register)?;

        registers.set_register(
            instruction.destination_register,
            &Value::Text(context.content.clone()),
        )?;

        crate::debug_print!(debug, "Executed POP : Popped value from context stack.",);

        Ok(())
    }

    fn context_drop(
        registers: &mut Registers,
        instruction: &ContextDropInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        registers.pop_context(instruction.source_context_register)?;

        crate::debug_print!(debug, "Executed DRP : Dropped value from context stack.",);

        Ok(())
    }

    fn move_context(
        registers: &mut Registers,
        instruction: &MoveContextInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = registers
            .get_context(instruction.source_context_register)?
            .to_vec();
        registers.set_context(instruction.destination_context_register, &value)?;

        crate::debug_print!(
            debug,
            "Executed MVC : c{} = c{}",
            instruction.destination_context_register,
            instruction.source_context_register
        );

        Ok(())
    }

    fn subtract_immediate(
        registers: &mut Registers,
        instruction: &SubtractImmediateInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = Self::read_number(registers, instruction.source_register)?;

        if value < instruction.value {
            return Err(Exception::Executor(BaseException::new(
                format!(
                    "Cannot subtract {} from register r{} because it would result in a negative value.",
                    instruction.value, instruction.source_register
                ),
                None,
            )));
        }

        let new_value = Value::Number(value - instruction.value);
        registers.set_register(instruction.source_register, &new_value)?;

        crate::debug_print!(
            debug,
            "Executed SUBI: Subtracted {} from r{} resulting in {}.",
            instruction.value,
            instruction.source_register,
            new_value
        );

        Ok(())
    }

    pub fn execute(
        memory: &mut Memory,
        registers: &mut Registers,
        instruction: &Instruction,
        config: &Config,
    ) -> Result<(), Exception> {
        match instruction {
            // Data movement operations.
            Instruction::LoadString(i) => Self::load_string(registers, i, config.debug_run),
            Instruction::LoadImmediate(i) => Self::load_immediate(registers, i, config.debug_run),
            Instruction::LoadContent(i) => Self::load_content(registers, i, config.debug_run),
            Instruction::Move(i) => Self::mov(registers, i, config.debug_run),
            // Control flow operations.
            Instruction::Branch(i) => Self::branch(registers, i, config.debug_run),
            Instruction::Exit(_) => {
                Self::exit(memory, registers, config.debug_run);
                Ok(())
            }
            // I/O operations.
            Instruction::Print(i) => Self::print(registers, i, config.debug_run),
            Instruction::PrintLine(i) => Self::print_line(registers, i, config.debug_run),
            Instruction::PrintContext(i) => Self::print_context(registers, i, config.debug_run),
            // Generative operations.
            Instruction::Inference(i) => Self::inference(
                registers,
                i,
                &config.text_model,
                &config.text_model_overrides,
                config.debug_run,
                config.debug_chat,
            ),
            // Guardrails operations.
            Instruction::Evaluate(i) => Self::evaluate(
                registers,
                i,
                &config.text_model,
                &config.embedding_model,
                &config.text_model_overrides,
                config.debug_run,
                config.debug_chat,
            ),
            Instruction::Similarity(i) => Self::similarity(registers, i, &config.embedding_model, config.debug_run),
            // Context operations.
            Instruction::ContextPush(i) => Self::context_push(registers, i, config.debug_run),
            Instruction::ContextPop(i) => Self::context_pop(registers, i, config.debug_run),
            Instruction::ContextDrop(i) => Self::context_drop(registers, i, config.debug_run),
            Instruction::MoveContext(i) => Self::move_context(registers, i, config.debug_run),
            // Arithmetic operations.
            Instruction::SubtractImmediate(i) => Self::subtract_immediate(registers, i, config.debug_run),
        }
    }
}
