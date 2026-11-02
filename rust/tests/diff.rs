//! Differential test: every case in golden.txt was produced by running the shipped JS
//! rewriteLocationRefs (acorn) on the same input. The Rust port must match byte for byte.
use zeolite_parser::rewrite_location;

#[test]
fn matches_js_reference_on_golden_cases() {
    let text = include_str!("golden.txt");
    let mut failures: Vec<String> = Vec::new();
    let mut total = 0usize;
    let mut lines = text.split('\n').peekable();
    while let Some(l) = lines.next() {
        if l == "=====END" { break; }
        if l != "=====IN" { continue; }
        let mut input: Vec<&str> = Vec::new();
        while let Some(&n) = lines.peek() {
            if n == "-----OUT" { break; }
            input.push(lines.next().unwrap());
        }
        lines.next(); // -----OUT
        let mut expected: Vec<&str> = Vec::new();
        while let Some(&n) = lines.peek() {
            if n == "=====IN" || n == "=====END" { break; }
            expected.push(lines.next().unwrap());
        }
        let src = input.join("\n");
        let want = expected.join("\n");
        let want = want.strip_suffix('\n').unwrap_or(&want).to_string();
        total += 1;
        let got = rewrite_location(&src);
        if got != want {
            failures.push(format!("IN:   {:?}\nWANT: {:?}\nGOT:  {:?}", src, want, got));
        }
    }
    assert!(total > 0, "no golden cases parsed");
    assert!(failures.is_empty(), "{} of {} cases differ:\n{}", failures.len(), total, failures.join("\n\n"));
}
