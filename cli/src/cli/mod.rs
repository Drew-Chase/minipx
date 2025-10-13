// CLI module
//
// This module contains command-line interface functionality:
// - arguments: Command-line argument parsing and handling (renamed from command_line_arguments.rs)

pub mod arguments;

// Re-export main types for backward compatibility
pub use arguments::MinipxArguments;
