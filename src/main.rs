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
        Exception::Program(BaseException::caused_by(
            format!("Failed to create build directory: {}", constants::BUILD_DIR),
            e,
        ))
    })
}

fn env_required(key: &str) -> Result<String, Exception> {
    env::var(key).map_err(|e| {
        Exception::Program(BaseException::caused_by(
            format!("{} must be set in the .env file", key),
            format!("{:#?}", e),
        ))
    })
}

fn env_bool(key: &str) -> bool {
    env::var(key).map(|v| v == "true").unwrap_or(false)
}

fn env_opt_bool(key: &str) -> Option<bool> {
    env::var(key).ok().map(|v| v == "true")
}

fn env_opt<T: std::str::FromStr>(key: &str) -> Option<T> {
    env::var(key).ok().and_then(|v| v.parse().ok())
}

fn config() -> Result<Config, Exception> {
    if dotenv::dotenv().ok().is_none() {
        return Err(Exception::Program(BaseException::new(
            "Failed to load .env file".to_string(),
            None,
        )));
    }

    Ok(Config {
        text_model: env_required(constants::TEXT_MODEL_ENV)?,
        embedding_model: env_required(constants::EMBEDDING_MODEL_ENV)?,
        debug_build: env_bool(constants::DEBUG_BUILD_ENV),
        debug_run: env_bool(constants::DEBUG_RUN_ENV),
        debug_chat: env_bool(constants::DEBUG_CHAT_ENV),
        text_model_overrides: TextModelOverrides {
            stream: env_opt_bool(constants::TEXT_MODEL_STREAM_ENV),
            return_progress: env_opt_bool(constants::TEXT_MODEL_RETURN_PROGRESS_ENV),
            reasoning_format: env::var(constants::TEXT_MODEL_REASONING_FORMAT_ENV).ok(),
            temperature: env_opt(constants::TEXT_MODEL_TEMPERATURE_ENV),
            dynatemp_range: env_opt(constants::TEXT_MODEL_DYNATEMP_RANGE_ENV),
            dynatemp_exponent: env_opt(constants::TEXT_MODEL_DYNATEMP_EXPONENT_ENV),
            top_k: env_opt(constants::TEXT_MODEL_TOP_K_ENV),
            top_p: env_opt(constants::TEXT_MODEL_TOP_P_ENV),
            min_p: env_opt(constants::TEXT_MODEL_MIN_P_ENV),
            xtc_probability: env_opt(constants::TEXT_MODEL_XTC_PROBABILITY_ENV),
            xtc_threshold: env_opt(constants::TEXT_MODEL_XTC_THRESHOLD_ENV),
            typ_p: env_opt(constants::TEXT_MODEL_TYP_P_ENV),
            max_tokens: env_opt(constants::TEXT_MODEL_MAX_TOKENS_ENV),
            repeat_last_n: env_opt(constants::TEXT_MODEL_REPEAT_LAST_N_ENV),
            repeat_penalty: env_opt(constants::TEXT_MODEL_REPEAT_PENALTY_ENV),
            presence_penalty: env_opt(constants::TEXT_MODEL_PRESENCE_PENALTY_ENV),
            frequency_penalty: env_opt(constants::TEXT_MODEL_FREQUENCY_PENALTY_ENV),
            dry_multiplier: env_opt(constants::TEXT_MODEL_DRY_MULTIPLIER_ENV),
            dry_base: env_opt(constants::TEXT_MODEL_DRY_BASE_ENV),
            dry_allowed_length: env_opt(constants::TEXT_MODEL_DRY_ALLOWED_LENGTH_ENV),
            dry_penalty_last_n: env_opt(constants::TEXT_MODEL_DRY_PENALTY_LAST_N_ENV),
            timings_per_token: env_opt_bool(constants::TEXT_MODEL_TIMINGS_PER_TOKEN_ENV),
        },
    })
}

fn build(file_path: &str, config: &Config) -> Result<(), Exception> {
    let source = read_to_string(file_path).map_err(|e| {
        Exception::Program(BaseException::caused_by("Failed to read source file.", e))
    })?;

    let mut compiler = assembler::Assembler::new(source);
    let byte_code = compiler.assemble().map_err(|e| {
        Exception::Program(BaseException::caused_by(
            "Failed to assemble source file.",
            e.to_string(),
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
        Exception::Program(BaseException::caused_by(
            "Failed to write byte code to output file.",
            e,
        ))
    })?;

    println!("Build successful! Output written to {}", output_file_name);

    Ok(())
}

fn run(file_path: &str, config: &Config) -> Result<(), Exception> {
    let data = read(file_path).map_err(|e| {
        Exception::Program(BaseException::caused_by(
            "Failed to read byte code file.",
            e,
        ))
    })?;

    let mut processor = processor::Processor::new(config.clone());

    processor.load(&data).map_err(|e| {
        Exception::Program(BaseException::caused_by(
            "Failed to load byte code file.",
            e,
        ))
    })?;

    processor
        .run()
        .map_err(|e| Exception::Program(BaseException::caused_by("Failed to run program.", e)))
}

fn main() {
    if let Err(e) = start_up() {
        println!("Startup error: {}", e);
        return;
    }

    let config = match config() {
        Ok(config) => config,
        Err(e) => {
            println!("Configuration error: {}", e);
            return;
        }
    };

    let args: Vec<String> = env::args().collect();

    let Some(command) = args.get(1) else {
        println!("No command provided. {}", constants::HELP_USAGE);
        return;
    };
    let Some(file_path) = args.get(2) else {
        println!("No file path provided. {}", constants::HELP_USAGE);
        return;
    };

    let result = match command.as_str() {
        "build" => build(file_path, &config),
        "run" => run(file_path, &config),
        other => {
            println!("Unknown command: {}. {}", other, constants::HELP_USAGE);
            return;
        }
    };

    if let Err(e) = result {
        println!("{} error: {}", command, e);
    }
}
