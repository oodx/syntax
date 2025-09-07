use std::cell::RefCell; use std::rc::Rc; use std::collections::HashMap;
use syntax::tmpl::{Template, VariableResolver, FuncResolver};
use syntax::SyntaxError;

#[derive(Clone)]
struct Store { m: Rc<RefCell<HashMap<String,String>>> }
impl VariableResolver for Store { fn get(&self, k:&str)->Option<String>{ self.m.borrow().get(k).cloned() } }
impl FuncResolver for Store {
  fn call(&self, name:&str, a:&[String]) -> Result<String, SyntaxError> {
    match name {
      "set" => { if a.len()>=2 { self.m.borrow_mut().insert(a[0].clone(), a[1].clone()); } Ok(String::new()) },
      _ => Ok(String::new())
    }
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let st = Store { m: Rc::new(RefCell::new(HashMap::new())) };
  let tpl = Template::parse_jynx("%for:item(a,b,c)([${item}])(,)")?;
  let out = tpl.render(&st, &st)?;
  println!("{}", out);
  Ok(())
}

