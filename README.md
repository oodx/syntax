# syntax

Command, template AST, and parsers for safe, cross‑platform CLI construction.

- Safe modeling: programs, args, env, cwd, redirections, pipelines
- Cross-platform renderers: POSIX and Windows strategies
- Dry-run planner for testable output without execution
- Template AST with pluggable resolvers (env/context/custom)
- Parsers: bash‑like (${VAR}, $$), Jynx (%name:arg(text)), SimpleTL ({{var}}, {{func:arg(text)}})
- Nesting (Arg::Tpl) and first‑class for‑loops

Docs: `docs/rfcs/AST-RFC.md`, `docs/PARSERS.md`.

## Easy Mode (UX‑first)

Don’t wire traits by hand. Use the grab‑and‑go helpers/macros:

```rust
use syntax::prelude::*; // imports Template, resolvers, and helpers
use syntax::{sx_bash, sx_jynx, sx_simple, sx_store};

// 1) Bash‑like vars with defaults
let out = sx_bash!("release-${USER}.tar.gz")?;

// 2) Provide your own store
let vars = sx_store! { "name" => "world" };
let out = sx_bash!("hello ${name}", with: vars)?; // "hello world"

// 3) Jynx syntax (functions supported by your FuncResolver)
let out = sx_jynx!("%pre:warn(ERROR)")?; // With default NoFunc this returns "<pre:warn:ERROR>" in tests
```

## Paintbox Lens (optional)

Enable feature `paintbox` to render terminal UX directly via Paintbox:

```toml
[features]
paintbox = ["dep:paintbox"]

[dependencies.paintbox]
path = "../paintbox"
optional = true
```

```rust
use syntax::lens::{UiTheme, render_jynx_with_theme};
use paintbox::prelude::*;

let yaml = r#"
palette: { info: 39 }
classes: { info: { fg: "info", bold: true } }
"#;
let theme = Theme::from_yaml(yaml).unwrap();

let ui = UiTheme { theme, color_mode: ColorMode::Auto };
let out = render_jynx_with_theme("%color:info(Hello) %box:rounded(Status)(OK)", ui)?;
println!("{}", out);
```

## Symbol Table (Variables)

The crate stays generic. You provide a “symbol table” (variable store) via resolvers:

- VariableResolver: reads variables from your store
- FuncResolver("set"): writes variables into your store

A simple store is an `Rc<RefCell<HashMap<String,String>>>` shared by both resolvers.

## Examples

### 1) Basic variables (bash‑like)
```rust
use syntax::tmpl::{Template, VariableResolver, FuncResolver};
use syntax::SyntaxError;

struct Env;
impl VariableResolver for Env { fn get(&self, k:&str) -> Option<String> { std::env::var(k).ok() } }
struct NoFunc; impl FuncResolver for NoFunc { fn call(&self,_:&str,_:&[String])->Result<String,SyntaxError>{Ok(String::new())}}

let t = Template::parse("release-${USER}.tar.gz")?;
let s = t.render(&Env, &NoFunc)?;
```

### 2) Jynx functions and Paintbox (stub)
```rust
use syntax::tmpl::{Template, VariableResolver, FuncResolver};
use syntax::SyntaxError;

struct Vars; impl VariableResolver for Vars { fn get(&self,_:&str)->Option<String>{None} }
struct JynxFuncs; impl FuncResolver for JynxFuncs {
  fn call(&self, name:&str, a:&[String]) -> Result<String, SyntaxError> {
    match name {
      "color" => { let style=&a.get(0).cloned().unwrap_or_default(); let text=&a.get(1).cloned().unwrap_or_default(); Ok(format!("<color:{}:{}>",style,text)) }
      "e" => { let key=&a.get(0).cloned().unwrap_or_default(); let text=&a.get(1).cloned().unwrap_or_default(); let emoji = if key=="warn" {"⚠️"} else {"•"}; Ok(format!("{} {}",emoji,text)) }
      _ => Ok(String::new())
    }
  }
}

let t = Template::parse_jynx(r#"%color:red("Alert") %e:warn("Disk")"#)?;
let out = t.render(&Vars, &JynxFuncs)?;
```

### 3) For‑loop with a shared symbol table
```rust
use std::cell::RefCell; use std::rc::Rc; use std::collections::HashMap;
use syntax::tmpl::{Template, VariableResolver, FuncResolver};
use syntax::SyntaxError;

#[derive(Clone)]
struct Store { m: Rc<RefCell<HashMap<String,String>>> }
impl VariableResolver for Store { fn get(&self, k:&str)->Option<String>{ self.m.borrow().get(k).cloned() } }
impl FuncResolver for Store {
  fn call(&self, name:&str, a:&[String]) -> Result<String, SyntaxError> {
    match name {
      "set" => { if a.len()>=2 { self.m.borrow_mut().insert(a[0].clone(), a[1].clone()); } Ok(String::new()) }
      _ => Ok(String::new())
    }
  }
}

let st = Store { m: Rc::new(RefCell::new(HashMap::new())) };
let tpl = Template::parse_jynx("%for:item(a,b,c)([${item}])(,)")?;
let out = tpl.render(&st, &st)?; // "[a],[b],[c]"
```

Run more examples in `examples/`.
