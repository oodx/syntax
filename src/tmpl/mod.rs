//! Template AST for structured string construction.

use crate::error::SyntaxError;
pub mod parser;

#[derive(Debug, Clone, Default)]
pub struct Template(pub Vec<Segment>);

#[derive(Debug, Clone)]
pub enum Arg {
    Text(String),
    Tpl(Template),
}

#[derive(Debug, Clone)]
pub enum Segment {
    Lit(String),
    Var(String),
    Get(String),
    Set { key: String, value: Template },
    Func { name: String, args: Vec<Arg> },
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
    pub fn parse(input: &str) -> Result<Self, SyntaxError> { parser::bash::parse(input) }
    pub fn render<V: VariableResolver, F: FuncResolver>(
        &self,
        vars: &V,
        funcs: &F,
    ) -> Result<String, SyntaxError> {
        let mut out = String::new();
        for seg in &self.0 {
            match seg {
                Segment::Lit(s) => out.push_str(s),
                Segment::Var(k) | Segment::Get(k) => out.push_str(&vars.get(k).unwrap_or_default()),
                Segment::Set { key, value } => {
                    let val = value.render(vars, funcs)?;
                    let _ = funcs.call("set", &vec![key.clone(), val]);
                }
                Segment::Func { name, args } => {
                    let mut evald: Vec<String> = Vec::new();
                    for a in args {
                        match a {
                            Arg::Text(t) => evald.push(t.clone()),
                            Arg::Tpl(tpl) => evald.push(tpl.render(vars, funcs)?),
                        }
                    }
                    out.push_str(&funcs.call(name, &evald)?);
                }
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
    pub fn parse_jynx(input: &str) -> Result<Self, SyntaxError> { parser::jynx::parse_jynx(input) }

    /// Simple pseudo-template language ("SimpleTL")
    /// Supported tokens inside `{{ ... }}`:
    /// - Variables: `{{var}}` where var = [A-Za-z0-9_.-]+
    /// - Inline functions: `{{func:arg(text)}}` similar to Jynx, with balanced parentheses for text
    /// - Comments: `{{! anything here }}` (ignored)
    /// Everything else is treated as literal text, including unmatched braces.
    pub fn parse_simple(input: &str) -> Result<Self, SyntaxError> { parser::simple::parse_simple(input) }
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
