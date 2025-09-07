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
