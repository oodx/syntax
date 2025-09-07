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

