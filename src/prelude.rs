//! syntax::prelude - grab-and-go UX for templates

pub use crate::error::SyntaxError;
pub use crate::tmpl::{Template, VariableResolver, FuncResolver};
pub use crate::easy::{Env, NoFunc, Store, render_bash, render_jynx, render_simple};

