//! Template AST for structured string construction.

use crate::error::SyntaxError;

#[derive(Debug, Clone, Default)]
pub struct Template(pub Vec<Segment>);

#[derive(Debug, Clone)]
pub enum Segment {
    Lit(String),
    Var(String),
    Func { name: String, args: Vec<String> },
}

pub trait VariableResolver {
    fn get(&self, key: &str) -> Option<String>;
}

pub trait FuncResolver {
    fn call(&self, name: &str, args: &[String]) -> Result<String, SyntaxError>;
}

impl Template {
    /// Parse a string containing `${VAR}` expansions into a Template.
    /// Supports:
    /// - Literal text
    /// - `${VAR_NAME}` variable segments (A-Z, a-z, 0-9, `_`, `-` allowed)
    /// Escapes:
    /// - `$$` emits a single `$`
    /// Errors on unmatched `${` with no closing `}`.
    pub fn parse(input: &str) -> Result<Self, SyntaxError> {
        let mut segs: Vec<Segment> = Vec::new();
        let mut lit = String::new();
        let bytes = input.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] as char {
                '$' => {
                    // handle $$ escape
                    if i + 1 < bytes.len() && bytes[i + 1] as char == '$' {
                        lit.push('$');
                        i += 2;
                        continue;
                    }
                    // handle ${VAR}
                    if i + 1 < bytes.len() && bytes[i + 1] as char == '{' {
                        // flush lit
                        if !lit.is_empty() {
                            segs.push(Segment::Lit(std::mem::take(&mut lit)));
                        }
                        i += 2; // skip `${`
                        let start = i;
                        let mut found = false;
                        while i < bytes.len() {
                            if bytes[i] as char == '}' {
                                found = true;
                                break;
                            }
                            i += 1;
                        }
                        if !found { return Err(SyntaxError::ResolveError("Unclosed ${ in template".into())); }
                        let var = &input[start..i];
                        // basic validation: allow [A-Za-z0-9_\-]
                        if !var.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                            return Err(SyntaxError::ResolveError(format!("Invalid var name: {}", var)));
                        }
                        segs.push(Segment::Var(var.to_string()));
                        i += 1; // skip `}`
                        continue;
                    }
                    // lone '$' treated as literal
                    lit.push('$');
                    i += 1;
                }
                ch => {
                    lit.push(ch);
                    i += ch.len_utf8();
                }
            }
        }
        if !lit.is_empty() { segs.push(Segment::Lit(lit)); }
        Ok(Template(segs))
    }
    pub fn render<V: VariableResolver, F: FuncResolver>(
        &self,
        vars: &V,
        funcs: &F,
    ) -> Result<String, SyntaxError> {
        let mut out = String::new();
        for seg in &self.0 {
            match seg {
                Segment::Lit(s) => out.push_str(s),
                Segment::Var(k) => out.push_str(&vars.get(k).unwrap_or_default()),
                Segment::Func { name, args } => out.push_str(&funcs.call(name, args)?),
            }
        }
        Ok(out)
    }

    /// Parse a Jynx-style template with function segments of the form:
    ///   %name:arg(text)
    /// Alongside existing ${VAR} and $$ rules.
    /// - name: [A-Za-z_][A-Za-z0-9_\-]*
    /// - arg: any run of non-whitespace, non-parenthesis characters up to '(' (e.g., warn, red, class-1)
    /// - text: balanced parentheses content, supports nested parentheses
    pub fn parse_jynx(input: &str) -> Result<Self, SyntaxError> {
        let mut segs: Vec<Segment> = Vec::new();
        let mut lit = String::new();
        let bytes = input.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let ch = bytes[i] as char;
            if ch == '$' {
                // Reuse ${VAR} and $$ rules
                if i + 1 < bytes.len() && bytes[i + 1] as char == '$' {
                    lit.push('$');
                    i += 2;
                    continue;
                }
                if i + 1 < bytes.len() && bytes[i + 1] as char == '{' {
                    if !lit.is_empty() { segs.push(Segment::Lit(std::mem::take(&mut lit))); }
                    i += 2; // skip ${
                    let start = i;
                    let mut found = false;
                    while i < bytes.len() {
                        if bytes[i] as char == '}' { found = true; break; }
                        i += 1;
                    }
                    if !found { return Err(SyntaxError::ResolveError("Unclosed ${ in template".into())); }
                    let var = &input[start..i];
                    if !var.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                        return Err(SyntaxError::ResolveError(format!("Invalid var name: {}", var)));
                    }
                    segs.push(Segment::Var(var.to_string()));
                    i += 1; // skip }
                    continue;
                }
                lit.push('$');
                i += 1;
                continue;
            }

            if ch == '%' {
                // Try to parse %name:arg(text)
                let name_start = i + 1;
                let mut j = name_start;
                // name
                while j < bytes.len() {
                    let c = bytes[j] as char;
                    if c.is_ascii_alphanumeric() || c == '_' || c == '-' { j += 1; } else { break; }
                }
                if j == name_start { // no name, treat as literal '%'
                    lit.push('%'); i += 1; continue;
                }
                // expect ':'
                if j >= bytes.len() || bytes[j] as char != ':' {
                    // not a function, treat as literal
                    lit.push('%'); i += 1; continue;
                }
                let name = &input[name_start..j];
                j += 1; // skip ':'
                // parse arg until '(' or whitespace
                let arg_start = j;
                while j < bytes.len() {
                    let c = bytes[j] as char;
                    if c == '(' || c.is_whitespace() { break; }
                    j += 1;
                }
                if j >= bytes.len() || bytes[j] as char != '(' {
                    // not a proper function call, treat as literal
                    lit.push('%'); i += 1; continue;
                }
                let arg = &input[arg_start..j];
                // parse balanced parentheses for text
                j += 1; // skip '('
                let text_start = j;
                let mut depth = 1;
                while j < bytes.len() {
                    let c = bytes[j] as char;
                    if c == '(' { depth += 1; }
                    else if c == ')' { depth -= 1; if depth == 0 { break; } }
                    j += 1;
                }
                if depth != 0 { return Err(SyntaxError::ResolveError("Unclosed ( in %func call".into())); }
                let text = &input[text_start..j];
                j += 1; // skip ')'
                // flush literal and emit func segment
                if !lit.is_empty() { segs.push(Segment::Lit(std::mem::take(&mut lit))); }
                segs.push(Segment::Func { name: name.to_string(), args: vec![arg.to_string(), text.to_string()] });
                i = j;
                continue;
            }

            // default literal
            lit.push(ch);
            i += ch.len_utf8();
        }
        if !lit.is_empty() { segs.push(Segment::Lit(lit)); }
        Ok(Template(segs))
    }

    /// Simple pseudo-template language ("SimpleTL")
    /// Supported tokens inside `{{ ... }}`:
    /// - Variables: `{{var}}` where var = [A-Za-z0-9_.-]+
    /// - Inline functions: `{{func:arg(text)}}` similar to Jynx, with balanced parentheses for text
    /// - Comments: `{{! anything here }}` (ignored)
    /// Everything else is treated as literal text, including unmatched braces.
    pub fn parse_simple(input: &str) -> Result<Self, SyntaxError> {
        let mut segs: Vec<Segment> = Vec::new();
        let mut lit = String::new();
        let bytes = input.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            // look for '{{'
            if bytes[i] as char == '{' && i + 1 < bytes.len() && bytes[i + 1] as char == '{' {
                // flush literal
                if !lit.is_empty() { segs.push(Segment::Lit(std::mem::take(&mut lit))); }
                i += 2; // skip '{{'
                // find closing '}}'
                let start = i;
                let mut j = i;
                let mut found = false;
                while j + 1 < bytes.len() {
                    if bytes[j] as char == '}' && bytes[j + 1] as char == '}' { found = true; break; }
                    j += 1;
                }
                if !found {
                    // no closing, treat as literal '{{' and continue
                    lit.push_str("{{");
                    i = start;
                    continue;
                }
                let inner = input[start..j].trim();
                i = j + 2; // past '}}'
                // comment
                if inner.starts_with('!') { continue; }
                // variable
                if !inner.is_empty() && inner.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' ) {
                    segs.push(Segment::Var(inner.to_string()));
                    continue;
                }
                // function func:arg(text)
                // parse name
                let mut k = 0;
                while k < inner.len() {
                    let c = inner.as_bytes()[k] as char;
                    if c.is_ascii_alphanumeric() || c == '_' || c == '-' { k += 1; } else { break; }
                }
                if k > 0 && k < inner.len() && inner.as_bytes()[k] as char == ':' {
                    let name = &inner[..k];
                    let mut p = k + 1; // after ':'
                    // arg until '(' (no whitespace)
                    let arg_start = p;
                    while p < inner.len() {
                        let c = inner.as_bytes()[p] as char;
                        if c == '(' || c.is_whitespace() { break; }
                        p += 1;
                    }
                    if p < inner.len() && inner.as_bytes()[p] as char == '(' {
                        let arg = &inner[arg_start..p];
                        // balanced parentheses for text
                        p += 1; // skip '('
                        let text_start = p;
                        let mut depth = 1;
                        while p < inner.len() {
                            let c = inner.as_bytes()[p] as char;
                            if c == '(' { depth += 1; }
                            else if c == ')' { depth -= 1; if depth == 0 { break; } }
                            p += 1;
                        }
                        if depth == 0 {
                            let text = &inner[text_start..p];
                            segs.push(Segment::Func { name: name.to_string(), args: vec![arg.to_string(), text.to_string()] });
                            continue;
                        }
                    }
                }
                // fallback: treat whole block as literal including delimiters
                lit.push_str("{{");
                lit.push_str(inner);
                lit.push_str("}}");
            } else {
                // regular char
                let ch = bytes[i] as char;
                lit.push(ch);
                i += ch.len_utf8();
            }
        }
        if !lit.is_empty() { segs.push(Segment::Lit(lit)); }
        Ok(Template(segs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Vars;
    impl VariableResolver for Vars { fn get(&self, key: &str) -> Option<String> { Some(format!("<{}>", key)) } }
    struct NoFunc; impl FuncResolver for NoFunc { fn call(&self, _:&str, _:&[String]) -> Result<String, SyntaxError> { Ok(String::new()) } }

    #[test]
    fn parse_literal_only() {
        let t = Template::parse("hello world").unwrap();
        assert!(matches!(&t.0[0], Segment::Lit(s) if s == "hello world"));
    }

    #[test]
    fn parse_with_vars() {
        let t = Template::parse("hi ${USER} from ${HOST}").unwrap();
        let s = t.render(&Vars, &NoFunc).unwrap();
        assert_eq!(s, "hi <USER> from <HOST>");
    }

    #[test]
    fn parse_dollar_escape() {
        let t = Template::parse("price: $$100").unwrap();
        let s = t.render(&Vars, &NoFunc).unwrap();
        assert_eq!(s, "price: $100");
    }

    #[test]
    fn parse_unclosed_errors() {
        let err = Template::parse("${OPEN").unwrap_err();
        matches!(err, SyntaxError::ResolveError(_));
    }
}

#[cfg(test)]
mod jynx_tests {
    use super::*;
    struct NoVar; impl VariableResolver for NoVar { fn get(&self, _:&str) -> Option<String> { None } }
    struct EchoFunc; impl FuncResolver for EchoFunc { fn call(&self, name:&str, args:&[String]) -> Result<String, SyntaxError> { Ok(format!("<{}:{}:{}>", name, args.get(0).cloned().unwrap_or_default(), args.get(1).cloned().unwrap_or_default())) } }

    #[test]
    fn parse_jynx_func_basic() {
        let t = Template::parse_jynx("hello %pre:warn(ERROR) world").unwrap();
        let s = t.render(&NoVar, &EchoFunc).unwrap();
        assert_eq!(s, "hello <pre:warn:ERROR> world");
    }

    #[test]
    fn parse_jynx_nested_parens() {
        let t = Template::parse_jynx("%pre:warn(Foo(bar))").unwrap();
        let s = t.render(&NoVar, &EchoFunc).unwrap();
        assert_eq!(s, "<pre:warn:Foo(bar)>");
    }
}

#[cfg(test)]
mod simple_tests {
    use super::*;
    struct NoVar; impl VariableResolver for NoVar { fn get(&self, _:&str) -> Option<String> { None } }
    struct Echo; impl FuncResolver for Echo { fn call(&self, n:&str, a:&[String]) -> Result<String, SyntaxError> { Ok(format!("<{}:{}:{}>", n, a.get(0).cloned().unwrap_or_default(), a.get(1).cloned().unwrap_or_default())) } }

    #[test]
    fn simple_var_and_literal() {
        let t = Template::parse_simple("Hello {{user.name}}!").unwrap();
        let s = t.render(&NoVar, &Echo).unwrap();
        assert_eq!(s, "Hello !"); // NoVar returns None, so empty for var
    }

    #[test]
    fn simple_func_inline() {
        let t = Template::parse_simple("{{color:red(Hi)}} world").unwrap();
        let s = t.render(&NoVar, &Echo).unwrap();
        assert_eq!(s, "<color:red:Hi> world");
    }
}
