//! Rendering strategies for commands and templates.

use crate::cmd::{CommandSpec, PipelineSpec};
use crate::error::SyntaxError;

#[derive(Debug, Clone, Copy)]
pub enum QuotePolicy { Strict, Loose }

pub trait Renderer {
    fn render_cmd(&self, cmd: &CommandSpec) -> Result<String, SyntaxError>;
    fn render_pipe(&self, pipe: &PipelineSpec) -> Result<String, SyntaxError> {
        let mut parts = Vec::new();
        for c in &pipe.0 { parts.push(self.render_cmd(c)?); }
        Ok(parts.join(" | "))
    }
}

#[derive(Debug, Clone)]
pub struct PosixRenderer { pub quote: QuotePolicy }

impl Default for PosixRenderer { fn default() -> Self { Self { quote: QuotePolicy::Strict } } }

impl Renderer for PosixRenderer {
    fn render_cmd(&self, cmd: &CommandSpec) -> Result<String, SyntaxError> {
        // Minimal placeholder implementation (no actual escaping yet)
        if cmd.program.is_empty() { return Err(SyntaxError::RenderError("program empty".into())); }
        let mut s = String::new();
        // env prefix
        if !cmd.env.is_empty() {
            for (k,v) in &cmd.env { s.push_str(&format!("{}={} ", k, v)); }
        }
        s.push_str(&cmd.program);
        for a in &cmd.args { s.push(' '); s.push_str(a); }
        Ok(s)
    }
}

#[derive(Debug, Clone)]
pub struct WinRenderer { pub quote: QuotePolicy }

impl Default for WinRenderer { fn default() -> Self { Self { quote: QuotePolicy::Strict } } }

impl Renderer for WinRenderer {
    fn render_cmd(&self, cmd: &CommandSpec) -> Result<String, SyntaxError> {
        // Placeholder: same as Posix for now
        PosixRenderer::default().render_cmd(cmd)
    }
}

