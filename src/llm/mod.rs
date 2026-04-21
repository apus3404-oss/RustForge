pub mod ollama;
pub mod openai;
pub mod registry;
pub mod traits;
pub mod types;

pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use registry::LLMRegistry;
pub use traits::LLMProvider;
pub use types::{CompletionOptions, Message, MessageRole};
