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
        self.renderer.render_pipe_plan(pipe)
    }
}

#[cfg(feature = "exec")]
pub struct StdExecutor;

#[cfg(feature = "exec")]
impl Executor for StdExecutor {
    fn exec(&self, pipe: &PipelineSpec) -> Result<ExecResult, SyntaxError> {
        use std::process::{Command, Stdio as PStdio};
        use crate::render::{Renderer, PosixRenderer};
        // Execute via shell using the safe-rendered pipeline string
        let r = PosixRenderer::default();
        let cmdline = r.render_pipe(pipe).map_err(|e| SyntaxError::ExecError(e.to_string()))?;
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmdline)
            .stdin(PStdio::null())
            .output()
            .map_err(|e| SyntaxError::ExecError(e.to_string()))?;
        Ok(ExecResult{
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
