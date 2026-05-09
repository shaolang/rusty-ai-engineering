mod args;
mod io;
mod openai;

pub use args::*;
pub use io::*;
pub use openai::*;

pub type Result<T> = std::result::Result<T, openai_oxide::OpenAIError>;
