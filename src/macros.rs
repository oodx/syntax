//! Terse macros for everyday usage

#[macro_export]
macro_rules! sx_store {
    ( $( $k:expr => $v:expr ),* $(,)? ) => {{
        let mut __s = $crate::easy::Store::new();
        $( __s = __s.with($k, $v); )*
        __s
    }};
}

#[macro_export]
macro_rules! sx_bash {
    ($tpl:expr) => {{
        $crate::easy::render_bash($tpl, & $crate::easy::Env, & $crate::easy::NoFunc)
    }};
    ($tpl:expr, with: $vars:expr) => {{
        $crate::easy::render_bash($tpl, & $vars, & $crate::easy::NoFunc)
    }};
    ($tpl:expr, with: $vars:expr, funcs: $funcs:expr) => {{
        $crate::easy::render_bash($tpl, & $vars, & $funcs)
    }};
}

#[macro_export]
macro_rules! sx_jynx {
    ($tpl:expr) => {{
        $crate::easy::render_jynx($tpl, & $crate::easy::Env, & $crate::easy::NoFunc)
    }};
    ($tpl:expr, with: $vars:expr) => {{
        $crate::easy::render_jynx($tpl, & $vars, & $crate::easy::NoFunc)
    }};
    ($tpl:expr, with: $vars:expr, funcs: $funcs:expr) => {{
        $crate::easy::render_jynx($tpl, & $vars, & $funcs)
    }};
}

#[macro_export]
macro_rules! sx_simple {
    ($tpl:expr) => {{
        $crate::easy::render_simple($tpl, & $crate::easy::Env, & $crate::easy::NoFunc)
    }};
    ($tpl:expr, with: $vars:expr) => {{
        $crate::easy::render_simple($tpl, & $vars, & $crate::easy::NoFunc)
    }};
    ($tpl:expr, with: $vars:expr, funcs: $funcs:expr) => {{
        $crate::easy::render_simple($tpl, & $vars, & $funcs)
    }};
}

