use super::super::{Template, Segment};
use crate::error::SyntaxError;

pub fn parse(input: &str) -> Result<Template, SyntaxError> {
    let mut segs: Vec<Segment> = Vec::new();
    let mut lit = String::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] as char {
            '$' => {
                if i + 1 < bytes.len() && bytes[i + 1] as char == '$' {
                    lit.push('$');
                    i += 2;
                    continue;
                }
                // $name form
                if i + 1 < bytes.len() {
                    let mut j = i + 1;
                    if (bytes[j] as char).is_ascii_alphabetic() || bytes[j] as char == '_' {
                        j += 1;
                        while j < bytes.len() {
                            let c = bytes[j] as char;
                            if c.is_ascii_alphanumeric() || c == '_' || c == '-' { j += 1; } else { break; }
                        }
                        let name = &input[i+1..j];
                        if !lit.is_empty() { segs.push(Segment::Lit(std::mem::take(&mut lit))); }
                        segs.push(Segment::Get(name.to_string()));
                        i = j;
                        continue;
                    }
                }
                if i + 1 < bytes.len() && bytes[i + 1] as char == '{' {
                    if !lit.is_empty() { segs.push(Segment::Lit(std::mem::take(&mut lit))); }
                    i += 2; // skip `${`
                    let start = i;
                    let mut found = false;
                    while i < bytes.len() {
                        if bytes[i] as char == '}' { found = true; break; }
                        i += 1;
                    }
                    if !found { return Err(SyntaxError::ResolveError("Unclosed ${ in template".into())); }
                    let var = &input[start..i];
                    if !var.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                        return Err(SyntaxError::ResolveError(format!("Invalid var name: {}", var)));
                    }
                    segs.push(Segment::Var(var.to_string()));
                    i += 1; // skip `}`
                    continue;
                }
                lit.push('$');
                i += 1;
            }
            ch => { lit.push(ch); i += ch.len_utf8(); }
        }
    }
    if !lit.is_empty() { segs.push(Segment::Lit(lit)); }
    Ok(Template(segs))
}
