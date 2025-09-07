use super::super::{Template, Segment, Arg};
use crate::error::SyntaxError;

pub fn parse_simple(input: &str) -> Result<Template, SyntaxError> {
    let mut segs: Vec<Segment> = Vec::new();
    let mut lit = String::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] as char == '{' && i + 1 < bytes.len() && bytes[i + 1] as char == '{' {
            if !lit.is_empty() { segs.push(Segment::Lit(std::mem::take(&mut lit))); }
            i += 2; let start = i; let mut j = i; let mut found = false;
            while j + 1 < bytes.len() { if bytes[j] as char == '}' && bytes[j + 1] as char == '}' { found = true; break; } j += 1; }
            if !found { lit.push_str("{{"); i = start; continue; }
            let inner = input[start..j].trim(); i = j + 2;
            if inner.starts_with('!') { continue; }
            if !inner.is_empty() && inner.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.') {
                segs.push(Segment::Var(inner.to_string()));
                continue;
            }
            // func: name:arg(text)
            let mut k = 0; while k < inner.len() { let c = inner.as_bytes()[k] as char; if c.is_ascii_alphanumeric() || c == '_' || c == '-' { k += 1; } else { break; } }
            if k > 0 && k < inner.len() && inner.as_bytes()[k] as char == ':' {
                let name = &inner[..k]; let mut p = k + 1;
                let arg_start = p; while p < inner.len() { let c = inner.as_bytes()[p] as char; if c == '(' || c.is_whitespace() { break; } p += 1; }
                if p < inner.len() && inner.as_bytes()[p] as char == '(' {
                    let arg = &inner[arg_start..p]; p += 1; let text_start = p; let mut depth = 1;
                    while p < inner.len() { let c = inner.as_bytes()[p] as char; if c == '(' { depth += 1; } else if c == ')' { depth -= 1; if depth == 0 { break; } } p += 1; }
                    if depth == 0 {
                        let text1 = &inner[text_start..p]; p += 1;
                        let mut bodies: Vec<Template> = vec![super::bash::parse(text1)?];
                        // extra bodies
                        while p < inner.len() && inner.as_bytes()[p] as char == '(' {
                            p += 1; let s2 = p; let mut d2 = 1;
                            while p < inner.len() { let c = inner.as_bytes()[p] as char; if c == '(' { d2 += 1; } else if c == ')' { d2 -= 1; if d2 == 0 { break; } } p += 1; }
                            if d2 != 0 { break; }
                            let t2 = &inner[s2..p]; p += 1; bodies.push(super::bash::parse(t2)?);
                        }
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
                        continue;
                    }
                }
            }
            lit.push_str("{{"); lit.push_str(inner); lit.push_str("}}");
        } else {
            let ch = bytes[i] as char; lit.push(ch); i += ch.len_utf8();
        }
    }
    if !lit.is_empty() { segs.push(Segment::Lit(lit)); }
    Ok(Template(segs))
}
