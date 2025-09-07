//! Execution and planning interfaces.

use crate::cmd::PipelineSpec;
use crate::error::SyntaxError;
use crate::render::Renderer;

#[derive(Debug, Clone, Default)]
pub struct ExecResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait Executor {
    fn exec(&self, pipe: &PipelineSpec) -> Result<ExecResult, SyntaxError>;
}

/// A simple planner that renders a pipeline using the given renderer
/// and returns the planned string (no execution).
pub struct Planner<'a, R: Renderer> { pub renderer: &'a R }

impl<'a, R: Renderer> Planner<'a, R> {
    pub fn plan(&self, pipe: &PipelineSpec) -> Result<String, SyntaxError> {
        self.renderer.render_pipe(pipe)
    }
}

#[cfg(feature = "exec")]
pub struct StdExecutor;

#[cfg(feature = "exec")]
impl Executor for StdExecutor {
    fn exec(&self, _pipe: &PipelineSpec) -> Result<ExecResult, SyntaxError> {
        // Placeholder: to be implemented
        Err(SyntaxError::ExecError("StdExecutor not implemented".into()))
    }
}

