//! zeolite-parser: Rust port of the JS location rewriter (rewriteLocationRefs).
//! Rules mirror the JS function one for one; the JS in the shell stays the reference
//! until the differential test (tests/golden.txt) is green on every case.
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::{GetSpan, SourceType};

const ZL_WIN: [&str; 6] = ["window", "self", "top", "parent", "globalThis", "document"];
const ZL_LP: [&str; 13] = [
    "href", "origin", "protocol", "host", "hostname", "port", "pathname", "search", "hash",
    "assign", "replace", "reload", "toString",
];

#[derive(Default)]
struct Rw {
    /// (start, end, member): member = wrap whole expression, else replace bare identifier
    edits: Vec<(usize, usize, bool)>,
    lhs_skip: Vec<(usize, usize)>,
    shadow: bool,
}

fn span_of(e: &Expression) -> (usize, usize) {
    let s = e.span();
    (s.start as usize, s.end as usize)
}

fn is_loc_expr(e: &Expression) -> bool {
    matches!(e, Expression::Identifier(id) if id.name == "location")
}

fn key_is_location(k: &PropertyKey) -> bool {
    matches!(k, PropertyKey::StaticIdentifier(id) if id.name == "location")
}

/// Port of zlP: true when a binding pattern binds the name "location".
fn zlp(p: &BindingPattern) -> bool {
    match p {
        BindingPattern::BindingIdentifier(id) => id.name == "location",
        BindingPattern::AssignmentPattern(a) => zlp(&a.left),
        BindingPattern::ObjectPattern(o) => {
            o.properties.iter().any(|pr| {
                (key_is_location(&pr.key)
                    && (pr.shorthand || zlp(&pr.value)))
                    || (!matches!(pr.value, BindingPattern::BindingIdentifier(_)) && zlp(&pr.value))
            }) || o.rest.as_ref().map_or(false, |r| zlp(&r.argument))
        }
        BindingPattern::ArrayPattern(a) => {
            a.elements.iter().flatten().any(zlp) || a.rest.as_ref().map_or(false, |r| zlp(&r.argument))
        }
    }
}

fn params_bind_location(ps: &FormalParameters) -> bool {
    ps.items.iter().any(|p| zlp(&p.pattern))
        || ps.rest.as_ref().map_or(false, |r| zlp(&r.rest.argument))
}

impl<'a> Visit<'a> for Rw {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        match &it.id {
            BindingPattern::BindingIdentifier(id) if id.name == "location" => self.shadow = true,
            BindingPattern::ObjectPattern(o) => {
                for pr in &o.properties {
                    let loc_value = matches!(&pr.value, BindingPattern::BindingIdentifier(i) if i.name == "location");
                    if key_is_location(&pr.key) && (pr.shorthand || loc_value) {
                        self.shadow = true;
                    }
                }
            }
            _ => {}
        }
        if let Some(init) = &it.init {
            if is_loc_expr(init) {
                let (s, e) = span_of(init);
                self.edits.push((s, e, false));
            }
        }
        walk::walk_variable_declarator(self, it);
    }

    fn visit_function(&mut self, it: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if it.r#type == FunctionType::FunctionDeclaration {
            if it.id.as_ref().map_or(false, |id| id.name == "location") {
                self.shadow = true;
            }
        }
        if params_bind_location(&it.params) {
            self.shadow = true;
        }
        walk::walk_function(self, it, flags);
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        if params_bind_location(&it.params) {
            self.shadow = true;
        }
        walk::walk_arrow_function_expression(self, it);
    }

    fn visit_class(&mut self, it: &Class<'a>) {
        if it.r#type == ClassType::ClassDeclaration && it.id.as_ref().map_or(false, |id| id.name == "location") {
            self.shadow = true;
        }
        walk::walk_class(self, it);
    }

    fn visit_catch_clause(&mut self, it: &CatchClause<'a>) {
        if let Some(p) = &it.param {
            if matches!(&p.pattern, BindingPattern::BindingIdentifier(i) if i.name == "location") {
                self.shadow = true;
            }
        }
        walk::walk_catch_clause(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if is_loc_expr(&it.right) {
            let (s, e) = span_of(&it.right);
            self.edits.push((s, e, false));
        }
        /* obj.location = v is a property write on a plain object, not a navigation */
        if let AssignmentTarget::StaticMemberExpression(m) = &it.left {
            let win_obj = matches!(&m.object, Expression::Identifier(id) if ZL_WIN.contains(&id.name.as_str()));
            if m.property.name == "location" && !win_obj {
                self.lhs_skip.push((m.span.start as usize, m.span.end as usize));
            }
        }
        walk::walk_assignment_expression(self, it);
    }

    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        if let Some(arg) = &it.argument {
            if is_loc_expr(arg) {
                let (s, e) = span_of(arg);
                self.edits.push((s, e, false));
            }
        }
        walk::walk_return_statement(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        for a in &it.arguments {
            if let Some(e) = a.as_expression() {
                if is_loc_expr(e) {
                    let (s, en) = span_of(e);
                    self.edits.push((s, en, false));
                }
            }
        }
        walk::walk_call_expression(self, it);
    }

    fn visit_new_expression(&mut self, it: &NewExpression<'a>) {
        for a in &it.arguments {
            if let Some(e) = a.as_expression() {
                if is_loc_expr(e) {
                    let (s, en) = span_of(e);
                    self.edits.push((s, en, false));
                }
            }
        }
        walk::walk_new_expression(self, it);
    }

    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        /* children first, as acorn-walk's full() does: the order decides ties in the stable sort */
        walk::walk_static_member_expression(self, it);
        let prop = it.property.name.as_str();
        let object_ok = !matches!(&it.object, Expression::Super(_) | Expression::ImportExpression(_));
        if prop == "location" && object_ok {
            self.edits.push((it.span.start as usize, it.span.end as usize, true));
        } else if ZL_LP.contains(&prop) && is_loc_expr(&it.object) {
            let (s, e) = span_of(&it.object);
            self.edits.push((s, e, false));
        }
    }
}

/// Returns `src` with `location` references rewritten, unchanged when it cannot be rewritten safely.
pub fn rewrite_location(src: &str) -> String {
    if src.is_empty() || !src.contains("location") || src.len() > 4_000_000 {
        return src.to_string();
    }
    let alloc = Allocator::default();
    let opts = ParseOptions { preserve_parens: false, ..ParseOptions::default() };
    let parsed = Parser::new(&alloc, src, SourceType::cjs()).with_options(opts).parse();
    if parsed.program.body.is_empty() && !src.trim().is_empty() {
        return src.to_string();
    }
    let mut rw = Rw::default();
    rw.visit_program(&parsed.program);
    if rw.shadow {
        return src.to_string();
    }
    let mut edits: Vec<(usize, usize, bool)> = rw
        .edits
        .into_iter()
        .filter(|(s, e, _)| !rw.lhs_skip.contains(&(*s, *e)))
        .collect();
    if edits.is_empty() {
        return src.to_string();
    }
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    let mut out = src.to_string();
    let mut last = usize::MAX;
    for (s, e, member) in edits {
        if e > last {
            continue;
        }
        last = s;
        let text = out[s..e].to_string();
        let rep = if member {
            format!("__zlSafe({}).__zlLoc$", text)
        } else {
            "__zlSafe(window).__zlLoc$".to_string()
        };
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
    fn rewrites_bare_location_in_call() {
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
