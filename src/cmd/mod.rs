//! Command modeling for safe CLI construction.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub cwd: Option<String>,
    pub stdin: Stdio,
    pub stdout: Stdio,
    pub stderr: Stdio,
    pub flags: CmdFlags,
}

#[derive(Debug, Clone, Default)]
pub struct CmdFlags {
    pub background: bool,
    pub timeout_ms: Option<u64>,
    pub retries: u8,
}

#[derive(Debug, Clone)]
pub enum Stdio {
    Inherit,
    Null,
    File { path: String, append: bool },
    Pipe,
}

impl Default for Stdio {
    fn default() -> Self { Stdio::Inherit }
}

#[derive(Debug, Clone, Default)]
pub struct PipelineSpec(pub Vec<CommandSpec>);

impl PipelineSpec {
    pub fn new() -> Self { PipelineSpec(Vec::new()) }
    pub fn push(&mut self, cmd: CommandSpec) { self.0.push(cmd); }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}

