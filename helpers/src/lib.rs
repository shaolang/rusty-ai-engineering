mod args;
mod io;
mod openai;
mod vectordb;

pub use args::*;
pub use io::*;
pub use openai::*;
pub use vectordb::*;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
