use similar::{ChangeTag, TextDiff};
use std::fmt::Write;

/// Generates a Unified Diff format string comparing old_text and new_text.
pub fn generate_diff(old_text: &str, new_text: &str) -> String {
    let diff = TextDiff::from_lines(old_text, new_text);
    let mut output = String::new();

    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            let _ = write!(&mut output, "{}{}", sign, change);
        }
    }
    
    output
}
