//! zeolite-parser: Rust port of the JS location rewriter (rewriteLocationRefs).
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Mirrors the JS rules: bail out if `location` is ever bound, rewrite
/// `location` references and `x.location` reads to `__zlSafe(...).__zlLoc$`.
struct Rw {
    edits: Vec<(usize, usize, String)>,
    skip: Vec<(usize, usize)>,
    shadow: bool,
}

const ZL_WIN: [&str; 6] = ["window", "self", "top", "parent", "globalThis", "document"];

impl<'a> Visit<'a> for Rw {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(id) = &it.id {
            if id.name == "location" { self.shadow = true; }
        }
        walk::walk_variable_declarator(self, it);
    }
    fn visit_function(&mut self, it: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if let Some(id) = &it.id { if id.name == "location" { self.shadow = true; } }
        for p in &it.params.items {
            if let BindingPattern::BindingIdentifier(pid) = &p.pattern {
                if pid.name == "location" { self.shadow = true; }
            }
        }
        walk::walk_function(self, it, flags);
    }
    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        for p in &it.params.items {
            if let BindingPattern::BindingIdentifier(pid) = &p.pattern {
                if pid.name == "location" { self.shadow = true; }
            }
        }
        walk::walk_arrow_function_expression(self, it);
    }
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if it.name == "location" {
            self.edits.push((it.span.start as usize, it.span.end as usize, "__zlSafe(window).__zlLoc$".to_string()));
        }
    }
    /* obj.location = v on a plain object is a property write, not a navigation */
    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if let AssignmentTarget::StaticMemberExpression(m) = &it.left {
            if m.property.name == "location" && !is_win_obj(&m.object) {
                self.skip.push((m.span.start as usize, m.span.end as usize));
            }
        }
        walk::walk_assignment_expression(self, it);
    }
    /* x.location (read) becomes __zlSafe(x.location).__zlLoc$ */
    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        if it.property.name == "location" && !is_win_obj(&it.object) && !matches!(it.object, Expression::Super(_)) {
            let (s, e) = (it.span.start as usize, it.span.end as usize);
            self.edits.push((s, e, String::new())); /* text filled in at apply time */
        }
        walk::walk_static_member_expression(self, it);
    }
}

/// Returns the rewritten source, or the input unchanged when nothing applies.
pub fn is_win_obj(e: &Expression) -> bool {
    matches!(e, Expression::Identifier(id) if ZL_WIN.contains(&id.name.as_str()))
}

/// Returns `src` with `location` references rewritten, or unchanged if it cannot be rewritten safely.
pub fn rewrite_location(src: &str) -> String {
    if !src.contains("location") { return src.to_string(); }
    let alloc = Allocator::default();
    let parsed = Parser::new(&alloc, src, SourceType::cjs()).parse();
    if parsed.program.body.is_empty() && !src.trim().is_empty() { return src.to_string(); }
    let mut rw = Rw { edits: Vec::new(), skip: Vec::new(), shadow: false };
    oxc_ast_visit::Visit::visit_program(&mut rw, &parsed.program);
    if rw.shadow { return src.to_string(); }
    let mut edits: Vec<(usize, usize, String)> = rw.edits.into_iter()
        .filter(|(s, e, _)| !rw.skip.contains(&(*s, *e)))
        .map(|(s, e, rep)| if rep.is_empty() {
            (s, e, format!("__zlSafe({}).__zlLoc$", &src[s..e]))
        } else { (s, e, rep) })
        .collect();
    if edits.is_empty() { return src.to_string(); }
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    let mut out = src.to_string();
    let mut last = usize::MAX;
    for (s, e, rep) in edits {
        if e > last { continue; }
        last = s;
        out.replace_range(s..e, &rep);
    }
    out
}

#[no_mangle]
pub extern "C" fn needs_location_rewrite(ptr: *const u8, len: usize) -> u32 {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let src = std::str::from_utf8(bytes).unwrap_or("");
    if src.contains("location") { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::rewrite_location;
    #[test]
    fn rewrites_bare_location() {
        assert_eq!(rewrite_location("x(location);"), "x(__zlSafe(window).__zlLoc$);");
    }
    #[test]
    fn rewrites_member_location() {
        assert_eq!(rewrite_location("a = w.location;"), "a = __zlSafe(w.location).__zlLoc$;");
    }
    #[test]
    fn keeps_member_location_writes() {
        let s = "obj.location = 1;";
        assert_eq!(rewrite_location(s), s);
    }
    #[test]
    fn leaves_shadowed_location() {
        let s = "function f(location){ return location; }";
        assert_eq!(rewrite_location(s), s);
    }
}
