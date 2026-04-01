use std::fs::read_to_string;

use crate::{
    config::Config,
    exception::{BaseException, Exception},
    processor::{
        control_unit::{
            instruction::{
                AddImmediateInstruction, BranchInstruction, BranchType, ContextDropInstruction,
                ContextPopInstruction, ContextPushInstruction, EvaluateInstruction,
                InferenceInstruction, Instruction, CountLinesInstruction, LoadContentInstruction,
                LoadImmediateInstruction, LoadStringInstruction, MoveContextInstruction,
                MoveInstruction, PrintContextInstruction, PrintInstruction, PrintLineInstruction,
                ReadLineInstruction, SimilarityInstruction, SubtractImmediateInstruction,
            },
            language_logic_unit::{
                LanguageLogicUnit, boolean_eval_params::BooleanEvalParams,
                text_generation_config::TextGenerationConfig,
            },
        },
        memory::Memory,
        registers::{ContextMessage, Registers, Value},
    },
};

pub struct Executor;

impl Executor {
    fn read_text(registers: &Registers, register_number: u32) -> Result<&String, Exception> {
        match registers.get_register(register_number).map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                format!("Failed to read r{}.", register_number),
                e,
            ))
        })? {
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
        match registers.get_register(register_number).map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                format!("Failed to read r{}.", register_number),
                e,
            ))
        })? {
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
        registers
            .set_register(instruction.destination_register, &value)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to store string in r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

        crate::debug_print!(
            debug,
            "Executed LS  : r{} = {:?}",
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
        registers
            .set_register(instruction.destination_register, &value)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to store immediate value in r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

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

        crate::debug_print!(
            debug,
            "Executed LC  : r{} = {:?}",
            instruction.destination_register,
            file_contents
        );

        registers
            .set_register(
                instruction.destination_register,
                &Value::Text(file_contents),
            )
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to store file contents in r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

        Ok(())
    }

    fn mov(
        registers: &mut Registers,
        instruction: &MoveInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = registers
            .get_register(instruction.source_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read source register r{}.",
                        instruction.source_register
                    ),
                    e,
                ))
            })?
            .clone();
        registers
            .set_register(instruction.destination_register, &value)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to write to destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

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
        let value_a = Self::read_number(registers, instruction.source_register_1).map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                "Failed to read first branch operand.",
                e,
            ))
        })?;
        let value_b = Self::read_number(registers, instruction.source_register_2).map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                "Failed to read second branch operand.",
                e,
            ))
        })?;

        let (is_true, label) = match instruction.branch_type {
            BranchType::Equal => (value_a == value_b, "BEQ"),
            BranchType::Less => (value_a < value_b, "BLT"),
            BranchType::LessEqual => (value_a <= value_b, "BLE"),
            BranchType::Greater => (value_a > value_b, "BGT"),
            BranchType::GreaterEqual => (value_a >= value_b, "BGE"),
            BranchType::NotEqual => (value_a != value_b, "BNE"),
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
        let value = registers
            .get_register(instruction.source_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!("Failed to read register r{}.", instruction.source_register),
                    e,
                ))
            })?;

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
        let value = registers
            .get_register(instruction.source_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!("Failed to read register r{}.", instruction.source_register),
                    e,
                ))
            })?;

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
        let context = registers
            .get_context(instruction.source_context_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read context register c{}.",
                        instruction.source_context_register
                    ),
                    e,
                ))
            })?;

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
        config: &Config,
    ) -> Result<(), Exception> {
        let value = Self::read_text(registers, instruction.source_register).map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                format!(
                    "Failed to read prompt from source register r{}.",
                    instruction.source_register
                ),
                e,
            ))
        })?;
        let conversation_history = registers
            .get_context(instruction.context_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read context register c{}.",
                        instruction.context_register
                    ),
                    e,
                ))
            })?;
        let text_generation_config = TextGenerationConfig {
            text_model: config.text_model.clone(),
            text_model_overrides: config.text_model_overrides.clone(),
            base_url: config.base_url.clone(),
            chat_completion_endpoint: config.chat_completion_endpoint.clone(),
            timeout_secs: config.timeout_secs,
            debug_chat: config.debug_chat,
        };
        let result =
            LanguageLogicUnit::generate_text(value, conversation_history, &text_generation_config)
                .map_err(|e| {
                    Exception::Executor(BaseException::caused_by("Text generation failed.", e))
                })?;

        crate::debug_print!(
            config.debug_run,
            "Executed INF  : r{} = '{:?}'",
            instruction.destination_register,
            result
        );

        registers
            .set_register(instruction.destination_register, &Value::Text(result))
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to write generated text to destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })
    }

    fn evaluate(
        registers: &mut Registers,
        instruction: &EvaluateInstruction,
        config: &Config,
    ) -> Result<(), Exception> {
        let value = Self::read_text(registers, instruction.source_register).map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                format!(
                    "Failed to read prompt from source register r{}.",
                    instruction.source_register
                ),
                e,
            ))
        })?;
        let micro_prompt = format!(
            "{}\nAnswer with exactly one word: YES or NO, TRUE or FALSE.\n\nAnswer only:",
            value
        );
        let true_values = vec!["YES".to_string(), "TRUE".to_string()];
        let false_values = vec!["NO".to_string(), "FALSE".to_string()];
        let conversation_history = registers
            .get_context(instruction.context_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read context register c{}.",
                        instruction.context_register
                    ),
                    e,
                ))
            })?;

        let eval_params = BooleanEvalParams {
            true_values,
            false_values,
            embedding_model: config.embedding_model.clone(),
        };
        let text_generation_config = TextGenerationConfig {
            text_model: config.text_model.clone(),
            text_model_overrides: config.text_model_overrides.clone(),
            base_url: config.base_url.clone(),
            chat_completion_endpoint: config.chat_completion_endpoint.clone(),
            timeout_secs: config.timeout_secs,
            debug_chat: config.debug_chat,
        };

        let result = LanguageLogicUnit::evaluate_boolean(
            &micro_prompt,
            &eval_params,
            conversation_history,
            &text_generation_config,
            &config.embeddings_endpoint,
        )
        .map_err(|e| {
            Exception::Executor(BaseException::caused_by("Boolean evaluation failed.", e))
        })?;

        crate::debug_print!(
            config.debug_run,
            "Executed EVAL: r{} = '{:?}'",
            instruction.destination_register,
            result
        );

        registers
            .set_register(instruction.destination_register, &Value::Number(result))
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to write evaluation result to destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })
    }

    fn similarity(
        registers: &mut Registers,
        instruction: &SimilarityInstruction,
        config: &Config,
    ) -> Result<(), Exception> {
        let value_a = Self::read_text(registers, instruction.source_register_1).map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                format!(
                    "Failed to read first operand from register r{}.",
                    instruction.source_register_1
                ),
                e,
            ))
        })?;
        let value_b = Self::read_text(registers, instruction.source_register_2).map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                format!(
                    "Failed to read second operand from register r{}.",
                    instruction.source_register_2
                ),
                e,
            ))
        })?;

        let result = LanguageLogicUnit::cosine_similarity(
            value_a,
            value_b,
            &config.embedding_model,
            &config.base_url,
            &config.embeddings_endpoint,
            config.timeout_secs,
        )
        .map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                "Cosine similarity computation failed.",
                e,
            ))
        })?;

        crate::debug_print!(
            config.debug_run,
            "Executed SIM : '{:?}' vs '{:?}' -> r{} = {}",
            value_a,
            value_b,
            instruction.destination_register,
            result
        );

        registers
            .set_register(instruction.destination_register, &Value::Number(result))
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to write similarity score to destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })
    }

    fn context_push(
        registers: &mut Registers,
        instruction: &ContextPushInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let register_value = registers
            .get_register(instruction.source_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read source register r{}.",
                        instruction.source_register
                    ),
                    e,
                ))
            })?;

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

        registers
            .push_context(
                ContextMessage::new(&instruction.role, &value),
                instruction.destination_context_register,
            )
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to push message onto context register c{}.",
                        instruction.destination_context_register
                    ),
                    e,
                ))
            })?;

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
        let context = registers
            .pop_context(instruction.source_context_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to pop message from context register c{}.",
                        instruction.source_context_register
                    ),
                    e,
                ))
            })?;

        registers
            .set_register(
                instruction.destination_register,
                &Value::Text(context.content),
            )
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to write popped message to destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

        crate::debug_print!(debug, "Executed POP : Popped value from context stack.",);

        Ok(())
    }

    fn context_drop(
        registers: &mut Registers,
        instruction: &ContextDropInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        registers
            .pop_context(instruction.source_context_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to drop message from context register c{}.",
                        instruction.source_context_register
                    ),
                    e,
                ))
            })?;

        crate::debug_print!(debug, "Executed DRP : Dropped value from context stack.",);

        Ok(())
    }

    fn move_context(
        registers: &mut Registers,
        instruction: &MoveContextInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value = registers
            .get_context(instruction.source_context_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read source context register c{}.",
                        instruction.source_context_register
                    ),
                    e,
                ))
            })?
            .to_vec();
        registers
            .set_context(instruction.destination_context_register, &value)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to write to destination context register c{}.",
                        instruction.destination_context_register
                    ),
                    e,
                ))
            })?;

        crate::debug_print!(
            debug,
            "Executed MVC : c{} = c{}",
            instruction.destination_context_register,
            instruction.source_context_register
        );

        Ok(())
    }

    fn add_immediate(
        registers: &mut Registers,
        instruction: &AddImmediateInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value =
            Self::read_number(registers, instruction.destination_register).map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

        let new_value = value.checked_add(instruction.value).ok_or_else(|| {
            Exception::Executor(BaseException::new(
                format!(
                    "Cannot add {} to register r{} because it would overflow.",
                    instruction.value, instruction.destination_register
                ),
                None,
            ))
        })?;
        let new_value = Value::Number(new_value);

        registers
            .set_register(instruction.destination_register, &new_value)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to write result to destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

        crate::debug_print!(
            debug,
            "Executed ADDI: Added {} to r{} resulting in {}.",
            instruction.value,
            instruction.destination_register,
            new_value
        );

        Ok(())
    }

    fn subtract_immediate(
        registers: &mut Registers,
        instruction: &SubtractImmediateInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let value =
            Self::read_number(registers, instruction.destination_register).map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

        let new_value = value.checked_sub(instruction.value).ok_or_else(|| {
            Exception::Executor(BaseException::new(
                format!(
                    "Cannot subtract {} from register r{} because it would underflow.",
                    instruction.value, instruction.destination_register
                ),
                None,
            ))
        })?;
        let new_value = Value::Number(new_value);

        registers
            .set_register(instruction.destination_register, &new_value)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to write result to destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

        crate::debug_print!(
            debug,
            "Executed SUBI: Subtracted {} from r{} resulting in {}.",
            instruction.value,
            instruction.destination_register,
            new_value
        );

        Ok(())
    }

    pub fn read_line(
        registers: &mut Registers,
        instruction: &ReadLineInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let csv_content = Self::read_text(registers, instruction.source_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read CSV content from source register r{}.",
                        instruction.source_register
                    ),
                    e,
                ))
            })?
            .clone();
        let line_number =
            Self::read_number(registers, instruction.line_number_register).map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read line number from register r{}.",
                        instruction.line_number_register
                    ),
                    e,
                ))
            })? as usize;

        let line = csv_content
            .lines()
            .nth(line_number)
            .ok_or_else(|| {
                Exception::Executor(BaseException::new(
                    format!(
                        "Out of bounds access when reading CSV: requested line {}, but only {} lines available.",
                        line_number,
                        csv_content.lines().count()
                    ),
                    None,
                ))
            })?;

        registers
            .set_register(
                instruction.destination_register,
                &Value::Text(line.to_string()),
            )
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to write CSV line to destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

        crate::debug_print!(
            debug,
            "Executed RLN : r{} = {:?}",
            instruction.destination_register,
            line
        );

        Ok(())
    }

    pub fn count_lines(
        registers: &mut Registers,
        instruction: &CountLinesInstruction,
        debug: bool,
    ) -> Result<(), Exception> {
        let row_count = Self::read_text(registers, instruction.source_register)
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to read content from source register r{}.",
                        instruction.source_register
                    ),
                    e,
                ))
            })?
            .lines()
            .count();
        let value = u32::try_from(row_count).map_err(|e| {
            Exception::Executor(BaseException::caused_by(
                "Failed to convert line count to u32",
                e.to_string(),
            ))
        })?;

        registers
            .set_register(instruction.destination_register, &Value::Number(value))
            .map_err(|e| {
                Exception::Executor(BaseException::caused_by(
                    format!(
                        "Failed to write line count to destination register r{}.",
                        instruction.destination_register
                    ),
                    e,
                ))
            })?;

        crate::debug_print!(
            debug,
            "Executed CLN : r{} = {}",
            instruction.destination_register,
            value
        );

        Ok(())
    }

    pub fn execute(
        memory: &mut Memory,
        registers: &mut Registers,
        instruction: &Instruction,
        config: &Config,
    ) -> Result<(), Exception> {
        let result = match instruction {
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
            Instruction::Inference(i) => Self::inference(registers, i, config),
            // Guardrails operations.
            Instruction::Evaluate(i) => Self::evaluate(registers, i, config),
            Instruction::Similarity(i) => Self::similarity(registers, i, config),
            // Context operations.
            Instruction::ContextPush(i) => Self::context_push(registers, i, config.debug_run),
            Instruction::ContextPop(i) => Self::context_pop(registers, i, config.debug_run),
            Instruction::ContextDrop(i) => Self::context_drop(registers, i, config.debug_run),
            Instruction::MoveContext(i) => Self::move_context(registers, i, config.debug_run),
            // Arithmetic operations.
            Instruction::AddImmediate(i) => Self::add_immediate(registers, i, config.debug_run),
            Instruction::SubtractImmediate(i) => {
                Self::subtract_immediate(registers, i, config.debug_run)
            }
            // Text operations.
            Instruction::ReadLine(i) => Self::read_line(registers, i, config.debug_run),
            Instruction::CountLines(i) => Self::count_lines(registers, i, config.debug_run),
        };
        result.map_err(|e| {
            Exception::Executor(BaseException::caused_by("Instruction execution failed.", e))
        })
    }
}
