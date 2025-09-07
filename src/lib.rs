//! syntax: Command and Template AST for safe CLI construction.

pub mod error;
pub mod cmd;
pub mod tmpl;
pub mod render;
pub mod exec;

pub use error::SyntaxError;

