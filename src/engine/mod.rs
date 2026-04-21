// src/engine/mod.rs
pub mod types;
pub mod parser;
pub mod interpolation;
pub mod events;
pub mod executor;
pub mod parallel;
pub mod merge;
pub mod unified_executor;

pub use types::*;
pub use parser::WorkflowParser;
pub use interpolation::VariableInterpolator;
pub use events::{EventBus, AgentEvent};
pub use executor::SequentialExecutor;
pub use parallel::ParallelExecutor;
pub use merge::{MergeStrategy, merge_results};
pub use unified_executor::WorkflowExecutor;
