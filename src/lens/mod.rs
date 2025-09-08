//! Syntax Lens: pre-wired resolvers and terse helpers for terminal UX
//!
//! Optional integration with Paintbox (feature = "paintbox").

use crate::tmpl::{Template, VariableResolver, FuncResolver};
use crate::SyntaxError;

#[derive(Clone, Default)]
pub struct Env;
impl VariableResolver for Env {
    fn get(&self, key: &str) -> Option<String> { std::env::var(key).ok() }
}

#[cfg(feature = "paintbox")]
mod pb {
    use super::*;
    use paintbox::prelude::*;

    #[derive(Clone)]
    pub struct UiTheme {
        pub theme: Theme,
        pub color_mode: ColorMode,
    }

    pub struct PaintboxFuncs { pub ui: UiTheme }
    impl FuncResolver for PaintboxFuncs {
        fn call(&self, name: &str, args: &[String]) -> Result<String, SyntaxError> {
            match name {
                // %color:<class>(text)
                "color" => {
                    let class = args.get(0).cloned().unwrap_or_default();
                    let text = args.get(1).cloned().unwrap_or_default();
                    let doc = pb_doc_span!(&self.ui.theme, &class, &text);
                    Ok(Renderer::new(self.ui.color_mode).render(&doc))
                }
                // %box:<style>(title)(body)
                "box" => {
                    let style_name = args.get(0).cloned().unwrap_or_else(|| "NORMAL".into());
                    let title = args.get(1).cloned().unwrap_or_default();
                    let body = args.get(2).cloned().unwrap_or_default();
                    let doc = Doc::from_plain(&body);
                    let bx = match style_name.to_ascii_uppercase().as_str() {
                        "ROUNDED" => wrap_in_box(&doc, &ROUNDED, &BoxOptions{ padding:1, title: Some(title) }),
                        "DOUBLE"  => wrap_in_box(&doc, &DOUBLE,  &BoxOptions{ padding:1, title: Some(title) }),
                        "ASCII"   => wrap_in_box(&doc, &ASCII,   &BoxOptions{ padding:1, title: Some(title) }),
                        _          => wrap_in_box(&doc, &NORMAL,  &BoxOptions{ padding:1, title: Some(title) }),
                    };
                    Ok(Renderer::new(self.ui.color_mode).render(&bx))
                }
                _ => Ok(String::new()),
            }
        }
    }

    /// Render a Jynx-style template (`%color:%`, `%box:%`) with Paintbox functions.
    pub fn render_jynx_with_theme(tpl: &str, ui: UiTheme) -> Result<String, SyntaxError> {
        let t = Template::parse_jynx(tpl)?;
        t.render(&Env, &PaintboxFuncs{ ui })
    }
}

#[cfg(feature = "paintbox")]
pub use pb::{UiTheme, render_jynx_with_theme};

#[macro_export]
macro_rules! sx_jynx {
    ($ctx:expr, $tpl:expr) => {{
        #[allow(unused_imports)]
        use crate::lens::render_jynx_with_theme;
        render_jynx_with_theme($tpl, $ctx)
    }};
}

