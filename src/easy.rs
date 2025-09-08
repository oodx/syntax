//! Easy mode helpers and default resolvers

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::tmpl::{Template, VariableResolver, FuncResolver};
use crate::SyntaxError;

#[derive(Clone, Default)]
pub struct Env;
impl VariableResolver for Env {
    fn get(&self, key: &str) -> Option<String> { std::env::var(key).ok() }
}

#[derive(Clone, Default)]
pub struct NoFunc;
impl FuncResolver for NoFunc {
    fn call(&self, _name: &str, _args: &[String]) -> Result<String, SyntaxError> { Ok(String::new()) }
}

#[derive(Clone, Default)]
pub struct Store { pub m: Rc<RefCell<HashMap<String, String>>> }
impl Store {
    pub fn new() -> Self { Self { m: Rc::new(RefCell::new(HashMap::new())) } }
    pub fn with(mut self, k: &str, v: &str) -> Self { self.m.borrow_mut().insert(k.into(), v.into()); self }
    pub fn get_map(&self) -> HashMap<String, String> { self.m.borrow().clone() }
}
impl VariableResolver for Store {
    fn get(&self, key: &str) -> Option<String> { self.m.borrow().get(key).cloned() }
}
impl FuncResolver for Store {
    fn call(&self, name: &str, args: &[String]) -> Result<String, SyntaxError> {
        match name {
            "set" if args.len() >= 2 => { self.m.borrow_mut().insert(args[0].clone(), args[1].clone()); Ok(String::new()) }
            _ => Ok(String::new()),
        }
    }
}

pub fn render_bash<T: AsRef<str>, V: VariableResolver, F: FuncResolver>(tpl: T, vars: &V, funcs: &F) -> Result<String, SyntaxError> {
    Template::parse(tpl.as_ref())?.render(vars, funcs)
}

pub fn render_jynx<T: AsRef<str>, V: VariableResolver, F: FuncResolver>(tpl: T, vars: &V, funcs: &F) -> Result<String, SyntaxError> {
    Template::parse_jynx(tpl.as_ref())?.render(vars, funcs)
}

pub fn render_simple<T: AsRef<str>, V: VariableResolver, F: FuncResolver>(tpl: T, vars: &V, funcs: &F) -> Result<String, SyntaxError> {
    Template::parse_simple(tpl.as_ref())?.render(vars, funcs)
}

