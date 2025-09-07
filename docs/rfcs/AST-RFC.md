# Syntax: Command and Template AST for CLI Tools

Status: Draft
Owner: RSB maintainers
Scope: `syntax` crate (standalone), imported by RSB and others

## 1. Summary
`syntax` provides a small, safe AST and renderer for building CLI commands and constructing strings/templates without brittle concatenation. It decouples:
- Command modeling (program, args, env, cwd, redirections, pipelines, background/timeout)
- Template strings (literal + variable/function nodes with pluggable expansion)
- Rendering for POSIX/Windows shells (quoting/escaping) and a dry-run planner
- Execution via an `Executor` trait so RSB can plug in its OS layer

## 2. Goals
- Safe composition: no ad hoc quoting/escaping in user code
- Cross-platform rendering: POSIX (`sh -c`) and Windows (`cmd.exe`/PowerShell) strategies
- Testable plans: render pipelines to human-readable plans without executing
- Reusable template engine: accept different variable sources (RSB context, env, etc.)
- Minimal dependency footprint; feature-gated execution

## 3. Non-Goals
- Full shell parser/emulator
- TUI/styled text (lives in `paintbox`)
- Owning RSB’s variable context; use traits so callers supply it

## 4. Architecture Overview
- `cmd`: `CommandSpec`, `Redir`, `PipelineSpec`, `Stdio`, `{Timeout, Retry}` options
- `tmpl`: `Template`, `Segment::{Lit, Var, Func}`, `VariableResolver`, `FuncResolver`
- `render`: `ShellRenderer` (posix), `WinRenderer`, `QuotePolicy` (strict/loose)
- `exec`: `Executor` trait; `StdExecutor` (feature = `exec`), `Planner` for dry runs
- `error`: unified `SyntaxError`

## 5. Core Types (Sketch)
```rust
pub struct CommandSpec { pub program: String, pub args: Vec<String>, pub env: BTreeMap<String,String>, pub cwd: Option<String>, pub stdin: Stdio, pub stdout: Stdio, pub stderr: Stdio, pub flags: CmdFlags }
pub struct PipelineSpec(pub Vec<CommandSpec>);
pub enum Stdio { Inherit, Null, File{path:String, append:bool}, Pipe }
pub struct CmdFlags { pub background: bool, pub timeout_ms: Option<u64>, pub retries: u8 }

pub struct Template(pub Vec<Segment>);
pub enum Segment { Lit(String), Var(String), Func{ name:String, args:Vec<String> } }

pub trait VariableResolver { fn get(&self, key: &str) -> Option<String>; }
pub trait FuncResolver { fn call(&self, name:&str, args:&[String]) -> Result<String, SyntaxError>; }

pub trait Renderer { fn render_cmd(&self, c:&CommandSpec) -> String; fn render_pipe(&self, p:&PipelineSpec) -> String; fn render_tmpl<T:VariableResolver, F:FuncResolver>(&self, t:&Template, vars:&T, funcs:&F) -> Result<String,SyntaxError>; }
pub trait Executor { fn exec(&self, pipe:&PipelineSpec) -> ExecResult; }
```

## 6. Rendering Rules (POSIX)
- Args are never concatenated with spaces; each arg is quoted independently when needed
- Env `KEY=VAL` are prefixed safely; `cwd` handled by wrapper (`cd` + `&&` or `Command::current_dir`)
- Redirections: `>`, `>>`, `<`, `2>&1` modeled explicitly
- Pipelines: render with `|` and proper grouping; background with `&`
- Quote policy: prefer single quotes; escape `'` by closing/opening `'` boundaries

## 7. Windows Rendering
- Two strategies behind features: `cmd.exe` and `powershell`
- Separate quoting/escaping rules (careful with `^`, `%VAR%`, quoting spaces)
- Same AST, different renderer

## 8. Templates
- `Template` is a small AST for string construction
- Resolvers supplied by caller; RSB can pass its context; env-based resolver for generic use
- Optional helpers for `${VAR}` parsing to build `Template` from strings

## 9. Execution & Planning
- `Executor` trait for dependency inversion; default `StdExecutor` uses `std::process`
- `Planner` pretty-prints planned commands/pipelines for dry-runs/tests
- Integrates with RSB’s `mock_cmd!` if desired via an adapter executor

## 10. Security & Safety
- No shell string concatenation in user code; all args and redirections are modeled
- Renderers are the sole place where quoting/escaping is applied
- Templates are resolved via explicit resolvers (whitelisting possible)

## 11. Integration
- RSB: use Command/Pipeline + Renderer + Executor; keep macros string-first
- Paintbox: unrelated; remains focused on styled text and layout

## 12. Milestones
- M1: CommandSpec/PipelineSpec, POSIX renderer, Planner, basic tests
- M2: Template AST + resolvers; parsing helpers; tests
- M3: Windows renderer(s); edge-case quoting; docs/examples

