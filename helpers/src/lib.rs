mod args;
mod history;
mod io;
mod openai;
pub mod vectordb;

pub use args::*;
pub use history::*;
pub use io::*;
pub use openai::*;
pub use vectordb::*;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
