//! Rendering strategies for commands and templates.

use crate::cmd::{CommandSpec, PipelineSpec, Stdio};
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
    fn render_cmd_plan(&self, cmd: &CommandSpec) -> Result<String, SyntaxError> { self.render_cmd(cmd) }
    fn render_pipe_plan(&self, pipe: &PipelineSpec) -> Result<String, SyntaxError> {
        let mut parts = Vec::new();
        for c in &pipe.0 { parts.push(self.render_cmd_plan(c)?); }
        Ok(parts.join(" | "))
    }
}

#[derive(Debug, Clone)]
pub struct PosixRenderer { pub quote: QuotePolicy }

impl Default for PosixRenderer { fn default() -> Self { Self { quote: QuotePolicy::Strict } } }

impl Renderer for PosixRenderer {
    fn render_cmd(&self, cmd: &CommandSpec) -> Result<String, SyntaxError> {
        if cmd.program.is_empty() {
            return Err(SyntaxError::RenderError("program empty".into()));
        }

        // Compose core command with env + program + args + redirections
        let mut parts: Vec<String> = Vec::new();

        // env assignments
        for (k, v) in &cmd.env {
            parts.push(format!("{}={}", k, quote_sh(v)));
        }

        // program
        parts.push(quote_prog(&cmd.program));

        // args
        for a in &cmd.args {
            parts.push(quote_sh(a));
        }

        // redirections
        // stdin
        if let Some(r) = render_redir(0, &cmd.stdin) { parts.push(r); }
        // stdout
        if let Some(r) = render_redir(1, &cmd.stdout) { parts.push(r); }
        // stderr
        if let Some(r) = render_redir(2, &cmd.stderr) { parts.push(r); }

        let mut cmd_str = parts.join(" ");

        // cwd via cd &&
        if let Some(dir) = &cmd.cwd {
            cmd_str = format!("cd {} && {}", quote_sh(dir), cmd_str);
        }

        Ok(cmd_str)
    }

    fn render_cmd_plan(&self, cmd: &CommandSpec) -> Result<String, SyntaxError> {
        let mut s = <Self as Renderer>::render_cmd(self, cmd)?;
        let mut metas = Vec::new();
        if let Some(ms) = cmd.flags.timeout_ms { metas.push(format!("timeout={}ms", ms)); }
        if cmd.flags.retries > 0 { metas.push(format!("retries={}", cmd.flags.retries)); }
        if !metas.is_empty() { s.push_str(&format!("  # {}", metas.join(", "))); }
        Ok(s)
    }
    fn render_pipe_plan(&self, pipe: &PipelineSpec) -> Result<String, SyntaxError> {
        let mut parts = Vec::new();
        for c in &pipe.0 { parts.push(self.render_cmd_plan(c)?); }
        let mut s = parts.join(" | ");
        if let Some(last) = pipe.0.last() { if last.flags.background { s.push_str(" &"); } }
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

impl PosixRenderer { }

fn render_redir(fd: u8, io: &Stdio) -> Option<String> {
    match (fd, io) {
        (0, Stdio::Inherit) | (1, Stdio::Inherit) | (2, Stdio::Inherit) => None,
        (0, Stdio::Null) => Some("< /dev/null".to_string()),
        (1, Stdio::Null) => Some("> /dev/null".to_string()),
        (2, Stdio::Null) => Some("2> /dev/null".to_string()),
        (0, Stdio::File { path, .. }) => Some(format!("< {}", quote_sh(path))),
        (1, Stdio::File { path, append }) => Some(format!("{} {}", if *append { ">>" } else { ">" }, quote_sh(path))),
        (2, Stdio::File { path, append }) => Some(format!("2{} {}", if *append { ">>" } else { ">" }, quote_sh(path))),
        (_, Stdio::Pipe) => None,
        _ => None,
    }
}

fn quote_prog(p: &str) -> String {
    // Allow bare if simple, else quote
    if is_simple_word(p) { p.to_string() } else { quote_sh(p) }
}

fn is_simple_word(s: &str) -> bool {
    s.chars().all(|c| matches!(c,
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | '/' | ':' | '+' | '%' | '@' | '=' | ','))
}

fn quote_sh(s: &str) -> String {
    if s.is_empty() { return "''".to_string(); }
    let escaped = s.replace("'", "'\"'\"'");
    format!("'{}'", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::{CommandSpec, Stdio, PipelineSpec};
    use std::collections::BTreeMap;

    #[test]
    fn quote_sh_basic() {
        assert_eq!(quote_sh(""), "''");
        assert_eq!(quote_sh("hello"), "'hello'");
        assert_eq!(quote_sh("hello world"), "'hello world'");
        assert_eq!(quote_sh("foo'bar"), "'foo'\"'\"'bar'");
    }

    #[test]
    fn render_cmd_env_cwd_args() {
        let mut env = BTreeMap::new();
        env.insert("FOO".to_string(), "bar baz".to_string());
        let cmd = CommandSpec {
            program: "echo".to_string(),
            args: vec!["hi".to_string()],
            env,
            cwd: Some("/tmp".to_string()),
            stdin: Stdio::Inherit,
            stdout: Stdio::Inherit,
            stderr: Stdio::Inherit,
            flags: Default::default(),
        };
        let r = PosixRenderer::default();
        let got = r.render_cmd(&cmd).unwrap();
        assert_eq!(got, "cd '/tmp' && FOO='bar baz' echo 'hi'");
    }

    #[test]
    fn render_cmd_redirections() {
        let cmd = CommandSpec {
            program: "/bin/cat".to_string(),
            args: vec!["file.txt".to_string()],
            env: BTreeMap::new(),
            cwd: None,
            stdin: Stdio::Null,
            stdout: Stdio::File { path: "out.txt".into(), append: true },
            stderr: Stdio::File { path: "err.txt".into(), append: false },
            flags: Default::default(),
        };
        let r = PosixRenderer::default();
        let got = r.render_cmd(&cmd).unwrap();
        assert_eq!(got, "/bin/cat 'file.txt' < /dev/null >> 'out.txt' 2> 'err.txt'");
    }

    #[test]
    fn render_pipe_plan_with_flags() {
        let mut p = PipelineSpec::new();
        let mut c1 = CommandSpec { program: "echo".into(), args: vec!["a".into()], ..Default::default() };
        c1.flags.timeout_ms = Some(500);
        let mut c2 = CommandSpec { program: "grep".into(), args: vec!["b".into()], ..Default::default() };
        c2.flags.retries = 2;
        c2.flags.background = true;
        p.push(c1);
        p.push(c2);
        let r = PosixRenderer::default();
        let got = r.render_pipe_plan(&p).unwrap();
        assert_eq!(got, "echo 'a'  # timeout=500ms | grep 'b'  # retries=2 &");
    }
}
