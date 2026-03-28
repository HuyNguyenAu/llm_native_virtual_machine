#[derive(Debug, Clone)]
pub struct Config {
    pub text_model: String,
    pub embedding_model: String,
    pub debug_build: bool,
    pub debug_run: bool,
    pub debug_chat: bool,
}
