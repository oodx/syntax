# Writing Parsers for `syntax::Template` (ELI5)

This crate gives you a tiny building block for string templating:
- A Template is just a list of segments:
  - `Lit(String)` → plain text
  - `Var(String)` → ask your VariableResolver for a value
  - `Func { name, args }` → call your FuncResolver to produce a value

A “parser” is any function that reads your favorite template notation and turns it into `Template(Vec<Segment>)`.

## What exists already
- Bash-like: `Template::parse` supports
  - `${VAR}` for variables (letters, numbers, `_`, `-`)
  - `$$` for a literal `$`
- Jynx-like: `Template::parse_jynx` supports
  - `%name:arg(text)` for function segments (e.g., `%color:red("Hi")`, `%e:warn("Disk")`)
  - `${VAR}` and `$$` also work in the same string
  - Balanced parentheses in `(text)` are supported: `%pre:warn(Foo(bar))`

## ELI5: How to write your own parser
1) Start with an empty `Vec<Segment>` and an empty `String` called `lit` (for accumulating plain text).
2) Walk the input string character by character.
3) When you see the start of a special token:
   - For a variable (your choice of syntax), flush `lit` into `Lit(...)`, then push `Var(name)`.
   - For a function (your choice of syntax), flush `lit`, parse the function parts, then push `Func { name, args }`.
4) Otherwise, push the character onto `lit`.
5) At the end, flush any leftover `lit` as `Lit(...)`.
6) Return `Template(segs)`.

### Skeleton code (outline)
```rust
use syntax::tmpl::{Template, Segment};
use syntax::SyntaxError;

fn parse_my_format(input: &str) -> Result<Template, SyntaxError> {
    let mut segs = Vec::new();
    let mut lit = String::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '@' { // example: @VAR@ style
            // flush literal
            if !lit.is_empty() { segs.push(Segment::Lit(std::mem::take(&mut lit))); }
            // read name until next '@'
            let start = i + 1; let mut j = start;
            while j < bytes.len() && (bytes[j] as char) != '@' { j += 1; }
            if j >= bytes.len() { return Err(SyntaxError::ResolveError("Unclosed @VAR@".into())); }
            let name = &input[start..j];
            segs.push(Segment::Var(name.to_string()));
            i = j + 1; // past closing '@'
            continue;
        }
        // default literal path
        lit.push(ch);
        i += ch.len_utf8();
    }
    if !lit.is_empty() { segs.push(Segment::Lit(lit)); }
    Ok(Template(segs))
}
```

## Resolvers (plug in behavior)
- `VariableResolver` → how to look up variables (theme/env/context). You implement:
  ```rust
  trait VariableResolver { fn get(&self, key: &str) -> Option<String>; }
  ```
- `FuncResolver` → how to handle function calls (e.g., `%color:red("text")`). You implement:
  ```rust
  trait FuncResolver { fn call(&self, name: &str, args: &[String]) -> Result<String, SyntaxError>; }
  ```
- Render a template by passing both resolvers:
  ```rust
  let out = template.render(&my_vars, &my_funcs)?;
  ```

## Nesting and composition
- Composition (sequence): chain multiple segments in order, like:
  - `Lit("hello ")`, `Func{ name:"color", args:["red","world"] }`, `Lit("!")`
- Nesting (inside `(text)`):
  - We support nested templates via `Arg::Tpl(Template)` for function arguments and for-loops.
  - Jynx parser recursively parses function bodies; SimpleTL parser does too, and within bodies it uses the bash-like `${VAR}` form.

## Pattern limitations (today)
- `${VAR}`: var names limited to `[A-Za-z0-9_-]+`.
- `%name:arg(text)`: name uses `[A-Za-z0-9_-]+`; `arg` runs to `(`; `text` supports balanced parentheses; text is not recursively parsed into inner functions by default.
- No escape sequences inside names; quotes in text are okay (treated as regular characters).

## Tips
- Keep parsers tiny and focused on your notation; do not try to parse shell or other languages here.
- Always flush accumulated `lit` when you switch to emitting a `Var` or `Func` segment.
- Add unit tests that show both successful parses and clear error messages for unmatched delimiters.

## Where to start
- If your notation looks like Jynx’s, use `Template::parse_jynx` directly.
- If it’s different (e.g., `:var:` or `@VAR@`), copy the skeleton, produce segments, and reuse the same `render(..)` with your resolvers.

## Iteration (For-Loops)

First-class node: `Segment::For { var, list, body, sep }`.

- Jynx syntax: `%for:var(list)(body)(sep?)`
  - Example: `%for:item(a,b,c)([${item}])(,)` → `[a],[b],[c]`
  - `list`, `body`, `sep` are parsed as nested templates.

- SimpleTL syntax: `{{for:var(list)(body)(sep?)}}`
  - Example: `{{for:item(a,b,c)([${item}])(,)}}` → `[a],[b],[c]`
  - Inside bodies, prefer `${item}` form (bash-like) for variables.

List splitting order:
1) newline, 2) comma, 3) whitespace.

Per-iteration variable:
- Before each body render, we call `FuncResolver::call("set", [var, item])`. Implement `set` to write into your store. `VariableResolver` reads from the same store.

Reference resolver sketch:
```rust
use std::cell::RefCell; use std::collections::HashMap; use std::rc::Rc;
use syntax::tmpl::{VariableResolver, FuncResolver, SyntaxError};

struct Store { m: Rc<RefCell<HashMap<String,String>>> }
impl VariableResolver for Store { fn get(&self, k:&str) -> Option<String> { self.m.borrow().get(k).cloned() } }
impl FuncResolver for Store {
  fn call(&self, name:&str, a:&[String]) -> Result<String, SyntaxError> {
    match name {
      "set" => { if a.len()>=2 { self.m.borrow_mut().insert(a[0].clone(), a[1].clone()); } Ok(String::new()) }
      _ => Ok(String::new())
    }
  }
}
```

Paintbox integration:
- In your `FuncResolver`, map styling funcs like `color:red(text)` to Paintbox spans and render to ANSI (or plain when NO_COLOR).
