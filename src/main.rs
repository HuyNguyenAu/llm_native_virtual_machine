mod assembler;
mod config;
mod constants;
mod exception;
mod processor;

use std::{
    env,
    fs::{read, read_to_string, write},
    path::Path,
};

use crate::{
    config::{Config, TextModelOverrides},
    exception::{BaseException, Exception},
};

fn start_up() -> Result<(), Exception> {
    std::fs::create_dir_all(constants::BUILD_DIR).map_err(|e| {
        Exception::Program(BaseException::new(
            format!("Failed to create build directory: {}", constants::BUILD_DIR),
            Some(Box::new(e.into())),
        ))
    })
}

fn config() -> Result<Config, Exception> {
    if dotenv::dotenv().ok().is_none() {
        return Err(Exception::Program(BaseException::new(
            "Failed to load .env file".to_string(),
            None,
        )));
    }

    let text_model = env::var(constants::TEXT_MODEL_ENV).map_err(|e| {
        Exception::Program(BaseException::new(
            format!("{} must be set in the .env file", constants::TEXT_MODEL_ENV),
            Some(Box::new(format!("{:#?}", e).into())),
        ))
    })?;
    let embedding_model = env::var(constants::EMBEDDING_MODEL_ENV).map_err(|e| {
        Exception::Program(BaseException::new(
            format!(
                "{} must be set in the .env file",
                constants::EMBEDDING_MODEL_ENV
            ),
            Some(Box::new(format!("{:#?}", e).into())),
        ))
    })?;
    let debug_build = env::var(constants::DEBUG_BUILD_ENV)
        .map(|v| v == "true")
        .unwrap_or(false);
    let debug_run = env::var(constants::DEBUG_RUN_ENV)
        .map(|v| v == "true")
        .unwrap_or(false);
    let debug_chat = env::var(constants::DEBUG_CHAT_ENV)
        .map(|v| v == "true")
        .unwrap_or(false);

    let text_model_overrides = TextModelOverrides {
        stream: env::var("TEXT_MODEL_STREAM").ok().map(|v| v == "true"),
        return_progress: env::var("TEXT_MODEL_RETURN_PROGRESS")
            .ok()
            .map(|v| v == "true"),
        reasoning_format: env::var("TEXT_MODEL_REASONING_FORMAT").ok(),
        temperature: env::var("TEXT_MODEL_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse().ok()),
        dynatemp_range: env::var("TEXT_MODEL_DYNATEMP_RANGE")
            .ok()
            .and_then(|v| v.parse().ok()),
        dynatemp_exponent: env::var("TEXT_MODEL_DYNATEMP_EXPONENT")
            .ok()
            .and_then(|v| v.parse().ok()),
        top_k: env::var("TEXT_MODEL_TOP_K")
            .ok()
            .and_then(|v| v.parse().ok()),
        top_p: env::var("TEXT_MODEL_TOP_P")
            .ok()
            .and_then(|v| v.parse().ok()),
        min_p: env::var("TEXT_MODEL_MIN_P")
            .ok()
            .and_then(|v| v.parse().ok()),
        xtc_probability: env::var("TEXT_MODEL_XTC_PROBABILITY")
            .ok()
            .and_then(|v| v.parse().ok()),
        xtc_threshold: env::var("TEXT_MODEL_XTC_THRESHOLD")
            .ok()
            .and_then(|v| v.parse().ok()),
        typ_p: env::var("TEXT_MODEL_TYP_P")
            .ok()
            .and_then(|v| v.parse().ok()),
        max_tokens: env::var("TEXT_MODEL_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok()),
        repeat_last_n: env::var("TEXT_MODEL_REPEAT_LAST_N")
            .ok()
            .and_then(|v| v.parse().ok()),
        repeat_penalty: env::var("TEXT_MODEL_REPEAT_PENALTY")
            .ok()
            .and_then(|v| v.parse().ok()),
        presence_penalty: env::var("TEXT_MODEL_PRESENCE_PENALTY")
            .ok()
            .and_then(|v| v.parse().ok()),
        frequency_penalty: env::var("TEXT_MODEL_FREQUENCY_PENALTY")
            .ok()
            .and_then(|v| v.parse().ok()),
        dry_multiplier: env::var("TEXT_MODEL_DRY_MULTIPLIER")
            .ok()
            .and_then(|v| v.parse().ok()),
        dry_base: env::var("TEXT_MODEL_DRY_BASE")
            .ok()
            .and_then(|v| v.parse().ok()),
        dry_allowed_length: env::var("TEXT_MODEL_DRY_ALLOWED_LENGTH")
            .ok()
            .and_then(|v| v.parse().ok()),
        dry_penalty_last_n: env::var("TEXT_MODEL_DRY_PENALTY_LAST_N")
            .ok()
            .and_then(|v| v.parse().ok()),
        timings_per_token: env::var("TEXT_MODEL_TIMINGS_PER_TOKEN")
            .ok()
            .map(|v| v == "true"),
    };

    Ok(Config {
        text_model,
        embedding_model,
        text_model_overrides,
        debug_build,
        debug_run,
        debug_chat,
    })
}

fn build(file_path: &str, config: &Config) -> Result<(), Exception> {
    let source = read_to_string(file_path).map_err(|e| {
        Exception::Program(BaseException::new(
            "Failed to read source file.".to_string(),
            Some(Box::new(e.into())),
        ))
    })?;

    let mut compiler = assembler::Assembler::new(source);
    let byte_code = compiler.assemble().map_err(|e| {
        Exception::Program(BaseException::new(
            "Failed to assemble source file.".to_string(),
            Some(Box::new(e.to_string().into())),
        ))
    })?;

    if config.debug_build {
        println!("Assembled byte code ({} bytes):", byte_code.len());

        for (chunk_idx, chunk) in byte_code.chunks(4).enumerate() {
            let index = chunk_idx * 4;
            println!("{} {:02X} ({}): {:?}", chunk_idx, index, index, chunk);
        }

        println!();
    }

    let path = Path::new(file_path);
    let stem = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
        Exception::Program(BaseException::new(
            "Failed to determine output filename from source file.".to_string(),
            None,
        ))
    })?;

    let output_file_name = format!("{}/{}.lpu", constants::BUILD_DIR, stem);

    write(&output_file_name, byte_code).map_err(|e| {
        Exception::Program(BaseException::new(
            "Failed to write byte code to output file.".to_string(),
            Some(Box::new(e.into())),
        ))
    })?;

    println!("Build successful! Output written to {}", output_file_name);

    Ok(())
}

fn run(file_path: &str, config: &Config) -> Result<(), Exception> {
    let data = read(file_path).map_err(|e| {
        Exception::Program(BaseException::new(
            "Failed to read byte code file.".to_string(),
            Some(Box::new(e.into())),
        ))
    })?;

    let mut processor = processor::Processor::new(config.clone());

    processor.load(&data).map_err(|e| {
        Exception::Program(BaseException::new(
            "Failed to load byte code file.".to_string(),
            Some(Box::new(e)),
        ))
    })?;

    processor.run().map_err(|e| {
        Exception::Program(BaseException::new(
            "Failed to run program.".to_string(),
            Some(Box::new(e)),
        ))
    })
}

fn main() {
    match start_up() {
        Ok(_) => (),
        Err(exception) => {
            println!("Startup error: {}", exception);
            return;
        }
    }

    let config = match config() {
        Ok(config) => config,
        Err(exception) => {
            println!("Configuration error: {}", exception);
            return;
        }
    };

    let args: Vec<String> = env::args().collect();
    let command = match args.get(1) {
        Some(command) => command,
        None => {
            println!("No command provided. {}", constants::HELP_USAGE);
            return;
        }
    };
    let file_path = match args.get(2) {
        Some(file_path) => file_path,
        None => {
            println!("No file path provided. {}", constants::HELP_USAGE);
            return;
        }
    };

    match command.as_str() {
        "build" => match build(file_path, &config) {
            Ok(_) => (),
            Err(exception) => println!("Build error: {}", exception),
        },
        "run" => match run(file_path, &config) {
            Ok(_) => (),
            Err(exception) => println!("Run error: {}", exception),
        },
        unexpected_command => println!(
            "Unknown command: {}. {}",
            unexpected_command,
            constants::HELP_USAGE
        ),
    }
}
