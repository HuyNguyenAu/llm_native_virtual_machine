mod assembler;
mod config;
mod constants;
mod env;
mod exception;
mod processor;

use std::{
    fs::{read, read_to_string, write},
    path::Path,
};

use crate::{
    config::{Config, TextModelOverrides},
    constants::{
        BUILD_DIR, DEBUG_BUILD_ENV, DEBUG_CHAT_ENV, DEBUG_RUN_ENV, EMBEDDING_MODEL_ENV, HELP_USAGE,
        OPENAI_BASE_URL_DEFAULT, OPENAI_BASE_URL_ENV, OPENAI_CHAT_COMPLETION_ENDPOINT_DEFAULT,
        OPENAI_CHAT_COMPLETION_ENDPOINT_ENV, OPENAI_EMBEDDINGS_ENDPOINT_DEFAULT,
        OPENAI_EMBEDDINGS_ENDPOINT_ENV, OPENAI_TIMEOUT_SECS_DEFAULT, OPENAI_TIMEOUT_SECS_ENV,
        TEXT_MODEL_ENV,
    },
    exception::{BaseException, Exception},
};

fn start_up() -> Result<(), Exception> {
    std::fs::create_dir_all(BUILD_DIR).map_err(|e| {
        Exception::Program(BaseException::caused_by(
            format!("Failed to create build directory: {}", BUILD_DIR),
            e,
        ))
    })
}

fn load_config() -> Result<Config, Exception> {
    if dotenv::dotenv().ok().is_none() {
        return Err(Exception::Program(BaseException::new(
            "Failed to load .env file".to_string(),
            None,
        )));
    }

    let text_model = env::required(TEXT_MODEL_ENV).map_err(|e| {
        Exception::Config(BaseException::caused_by(
            "Failed to load text model configuration from environment.".to_string(),
            e,
        ))
    })?;
    let embedding_model = env::required(EMBEDDING_MODEL_ENV).map_err(|e| {
        Exception::Config(BaseException::caused_by(
            "Failed to load embedding model configuration from environment.".to_string(),
            e,
        ))
    })?;

    Ok(Config {
        text_model,
        embedding_model,
        base_url: env::with_default(OPENAI_BASE_URL_ENV, OPENAI_BASE_URL_DEFAULT),
        chat_completion_endpoint: env::with_default(
            OPENAI_CHAT_COMPLETION_ENDPOINT_ENV,
            OPENAI_CHAT_COMPLETION_ENDPOINT_DEFAULT,
        ),
        embeddings_endpoint: env::with_default(
            OPENAI_EMBEDDINGS_ENDPOINT_ENV,
            OPENAI_EMBEDDINGS_ENDPOINT_DEFAULT,
        ),
        timeout_secs: env::u64_with_default(OPENAI_TIMEOUT_SECS_ENV, OPENAI_TIMEOUT_SECS_DEFAULT),
        debug_build: env::bool(DEBUG_BUILD_ENV),
        debug_run: env::bool(DEBUG_RUN_ENV),
        debug_chat: env::bool(DEBUG_CHAT_ENV),
        text_model_overrides: TextModelOverrides::from_env(),
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
            e,
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

    let output_file_name = format!("{}/{}.lpu", BUILD_DIR, stem);

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

    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            println!("Configuration error: {}", e);
            return;
        }
    };

    let args: Vec<String> = env::args();

    let result = match (args.get(1).map(String::as_str), args.get(2)) {
        (None, _) => {
            println!("No command provided. {}", HELP_USAGE);
            return;
        }
        (_, None) => {
            println!("No file path provided. {}", HELP_USAGE);
            return;
        }
        (Some("build"), Some(file_path)) => build(file_path, &config),
        (Some("run"), Some(file_path)) => run(file_path, &config),
        (Some(other), _) => {
            println!("Unknown command: {}. {}", other, HELP_USAGE);
            return;
        }
    };

    if let Err(e) = result {
        println!("Exception: {}", e);
    }
}
