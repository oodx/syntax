use super::super::{Template, Segment, Arg};
use crate::error::SyntaxError;

pub fn parse_jynx(input: &str) -> Result<Template, SyntaxError> {
    let mut segs: Vec<Segment> = Vec::new();
    let mut lit = String::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '$' {
            if i + 1 < bytes.len() && bytes[i + 1] as char == '$' { lit.push('$'); i += 2; continue; }
            if i + 1 < bytes.len() && bytes[i + 1] as char == '{' {
                if !lit.is_empty() { segs.push(Segment::Lit(std::mem::take(&mut lit))); }
                i += 2; let start = i; let mut found = false;
                while i < bytes.len() { if bytes[i] as char == '}' { found = true; break; } i += 1; }
                if !found { return Err(SyntaxError::ResolveError("Unclosed ${ in template".into())); }
                let var = &input[start..i];
                if !var.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                    return Err(SyntaxError::ResolveError(format!("Invalid var name: {}", var)));
                }
                segs.push(Segment::Var(var.to_string()));
                i += 1; continue;
            }
            lit.push('$'); i += 1; continue;
        }
        if ch == '%' {
            let name_start = i + 1; let mut j = name_start;
            while j < bytes.len() { let c = bytes[j] as char; if c.is_ascii_alphanumeric() || c == '_' || c == '-' { j += 1; } else { break; } }
            if j == name_start { lit.push('%'); i += 1; continue; }
            if j >= bytes.len() || bytes[j] as char != ':' { lit.push('%'); i += 1; continue; }
            let name = &input[name_start..j]; j += 1;
            let arg_start = j; while j < bytes.len() { let c = bytes[j] as char; if c == '(' || c.is_whitespace() { break; } j += 1; }
            if j >= bytes.len() || bytes[j] as char != '(' { lit.push('%'); i += 1; continue; }
            let arg = &input[arg_start..j]; j += 1; let text_start = j; let mut depth = 1;
            while j < bytes.len() { let c = bytes[j] as char; if c == '(' { depth += 1; } else if c == ')' { depth -= 1; if depth == 0 { break; } } j += 1; }
            if depth != 0 { return Err(SyntaxError::ResolveError("Unclosed ( in %func call".into())); }
            let text1 = &input[text_start..j]; j += 1;
            // Optional more parentheses groups
            let mut bodies: Vec<Template> = Vec::new();
            let first_tpl = parse_jynx(text1)?; bodies.push(first_tpl);
            // read additional ( ... )
            while j < bytes.len() && bytes[j] as char == '(' {
                j += 1; let start2 = j; let mut d2 = 1;
                while j < bytes.len() { let c = bytes[j] as char; if c == '(' { d2 += 1; } else if c == ')' { d2 -= 1; if d2 == 0 { break; } } j += 1; }
                if d2 != 0 { return Err(SyntaxError::ResolveError("Unclosed ( in %func call".into())); }
                let txt = &input[start2..j]; j += 1; bodies.push(parse_jynx(txt)?);
            }
            if !lit.is_empty() { segs.push(Segment::Lit(std::mem::take(&mut lit))); }
            if name == "for" {
                let var = arg.to_string();
                let list = bodies.get(0).cloned().unwrap_or_else(|| Template(vec![]));
                let body_t = bodies.get(1).cloned().unwrap_or_else(|| Template(vec![]));
                let sep = bodies.get(2).cloned();
                segs.push(Segment::For { var, list, body: body_t, sep });
            } else {
                let mut fn_args: Vec<Arg> = Vec::new();
                fn_args.push(Arg::Text(arg.to_string()));
                for t in bodies { fn_args.push(Arg::Tpl(t)); }
                segs.push(Segment::Func { name: name.to_string(), args: fn_args });
            }
            i = j; continue;
        }
        lit.push(ch); i += ch.len_utf8();
    }
    if !lit.is_empty() { segs.push(Segment::Lit(lit)); }
    Ok(Template(segs))
}
