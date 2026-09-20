//! Builds and matches the header text a file is expected to start with.
//! The only dynamic piece is `{year}`: rendered as the current year when
//! *inserting* a header, matched against any 4-digit run when *checking*
//! one (so a header stamped in an earlier year still passes today).

pub struct Template {
    /// The raw template text, e.g. `"// Copyright (c) {year} Acme Corp"`.
    pub raw: String,
}

impl Template {
    pub fn new(raw: impl Into<String>) -> Self {
        Template { raw: raw.into() }
    }

    /// The literal text to insert, with `{year}` replaced by `year`.
    pub fn render(&self, year: i32) -> String {
        self.raw.replace("{year}", &year.to_string())
    }

    /// True if `content` (the file's own text, header not yet stripped)
    /// starts with this template — modulo `{year}` matching any run of
    /// 4 ASCII digits — after skipping a leading shebang line (`#!...`)
    /// if present, since a header must come after that, not before it.
    pub fn matches(&self, content: &str) -> bool {
        let body = skip_shebang(content);
        let parts: Vec<&str> = self.raw.split("{year}").collect();
        match_template_parts(&parts, body)
    }
}

/// Returns `content` with a single leading shebang line (and the
/// newline after it) removed, if present; otherwise `content` unchanged.
pub fn skip_shebang(content: &str) -> &str {
    if let Some(rest) = content.strip_prefix("#!") {
        if let Some(nl) = rest.find('\n') {
            return &rest[nl + 1..];
        }
    }
    content
}

fn match_template_parts(parts: &[&str], text: &str) -> bool {
    let mut rest = text;
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            match rest.strip_prefix(part) {
                Some(r) => rest = r,
                None => return false,
            }
        } else {
            // Everything up to the next literal part is the {year}
            // placeholder's value — it must be a run of exactly 4 ASCII
            // digits, no more, no fewer, so "20264" doesn't count as a
            // "2026"-prefixed match.
            let digits_end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            if digits_end != 4 {
                return false;
            }
            rest = &rest[4..];
            match rest.strip_prefix(part) {
                Some(r) => rest = r,
                None => return false,
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_year_placeholder() {
        let t = Template::new("// Copyright (c) {year} Acme");
        assert_eq!(t.render(2026), "// Copyright (c) 2026 Acme");
    }

    #[test]
    fn template_with_no_placeholder_renders_unchanged() {
        let t = Template::new("// SPDX-License-Identifier: MIT");
        assert_eq!(t.render(2026), "// SPDX-License-Identifier: MIT");
    }

    #[test]
    fn matches_exact_rendered_header_at_top_of_file() {
        let t = Template::new("// Copyright (c) {year} Acme");
        let content = "// Copyright (c) 2026 Acme\nfn main() {}\n";
        assert!(t.matches(content));
    }

    #[test]
    fn matches_a_different_year_than_the_current_one() {
        let t = Template::new("// Copyright (c) {year} Acme");
        let content = "// Copyright (c) 2019 Acme\nfn main() {}\n";
        assert!(t.matches(content));
    }

    #[test]
    fn rejects_a_non_4_digit_year() {
        let t = Template::new("// Copyright (c) {year} Acme");
        assert!(!t.matches("// Copyright (c) 26 Acme\n"));
        assert!(!t.matches("// Copyright (c) 202612345 Acme\n"));
    }

    #[test]
    fn rejects_missing_header_entirely() {
        let t = Template::new("// Copyright (c) {year} Acme");
        assert!(!t.matches("fn main() {}\n"));
    }

    #[test]
    fn rejects_wrong_holder_name() {
        let t = Template::new("// Copyright (c) {year} Acme");
        assert!(!t.matches("// Copyright (c) 2026 SomeoneElse\nfn main() {}\n"));
    }

    #[test]
    fn matches_after_skipping_a_shebang_line() {
        let t = Template::new("# Copyright (c) {year} Acme");
        let content = "#!/bin/sh\n# Copyright (c) 2026 Acme\necho hi\n";
        assert!(t.matches(content));
    }

    #[test]
    fn skip_shebang_leaves_content_without_one_unchanged() {
        assert_eq!(skip_shebang("fn main() {}\n"), "fn main() {}\n");
    }

    #[test]
    fn template_with_two_placeholders_matches_both_independently() {
        let t = Template::new("{year}-{year}");
        assert!(t.matches("2020-2026 trailer"));
        assert!(!t.matches("2020-abcd trailer"));
    }
}
