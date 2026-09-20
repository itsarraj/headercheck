use std::fs;
use std::path::{Path, PathBuf};

use crate::template::Template;

const SKIP_DIR_NAMES: &[&str] = &[".git", "target", "node_modules", "vendor", "dist", "build"];

/// Every file under `root` whose extension (without the leading `.`) is
/// in `extensions`, skipping VCS/build/dependency directories.
pub fn find_files(root: &Path, extensions: &[String]) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk(root, extensions, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(dir: &Path, extensions: &[String], out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            if SKIP_DIR_NAMES.contains(&name.to_string_lossy().as_ref()) {
                continue;
            }
            walk(&path, extensions, out)?;
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.iter().any(|e| e == ext) {
                out.push(path);
            }
        }
    }
    Ok(())
}

/// Inserts `template`'s rendered header (for `year`) into `content`,
/// after a leading shebang line if present, followed by a blank line
/// before the original content resumes.
pub fn insert_header(content: &str, template: &Template, year: i32) -> String {
    let rendered = template.render(year);
    if let Some(rest) = content.strip_prefix("#!") {
        if let Some(nl) = rest.find('\n') {
            let shebang_line = &content[..content.len() - rest.len() + nl + 1];
            let body = &rest[nl + 1..];
            return format!("{shebang_line}{rendered}\n\n{body}");
        }
    }
    format!("{rendered}\n\n{content}")
}

pub fn has_header(content: &str, template: &Template) -> bool {
    template.matches(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("headercheck-test-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn find_files_matches_only_requested_extensions() {
        let dir = temp_dir("ext-filter");
        fs::write(dir.join("a.rs"), "").unwrap();
        fs::write(dir.join("b.py"), "").unwrap();
        fs::write(dir.join("c.txt"), "").unwrap();
        let found = find_files(&dir, &["rs".to_string()]).unwrap();
        assert_eq!(found, vec![dir.join("a.rs")]);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_files_skips_vcs_and_build_directories() {
        let dir = temp_dir("skip-dirs");
        fs::create_dir_all(dir.join("target")).unwrap();
        fs::write(dir.join("target/generated.rs"), "").unwrap();
        fs::write(dir.join("real.rs"), "").unwrap();
        let found = find_files(&dir, &["rs".to_string()]).unwrap();
        assert_eq!(found, vec![dir.join("real.rs")]);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_files_recurses_into_nested_directories() {
        let dir = temp_dir("nested");
        fs::create_dir_all(dir.join("src/deep")).unwrap();
        fs::write(dir.join("src/deep/lib.rs"), "").unwrap();
        let found = find_files(&dir, &["rs".to_string()]).unwrap();
        assert_eq!(found, vec![dir.join("src/deep/lib.rs")]);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn insert_header_prepends_with_a_blank_line_separator() {
        let t = Template::new("// Copyright (c) {year} Acme");
        let out = insert_header("fn main() {}\n", &t, 2026);
        assert_eq!(out, "// Copyright (c) 2026 Acme\n\nfn main() {}\n");
    }

    #[test]
    fn insert_header_goes_after_a_shebang_line() {
        let t = Template::new("# Copyright (c) {year} Acme");
        let out = insert_header("#!/bin/sh\necho hi\n", &t, 2026);
        assert_eq!(out, "#!/bin/sh\n# Copyright (c) 2026 Acme\n\necho hi\n");
    }

    #[test]
    fn insert_header_is_idempotent_via_has_header_afterward() {
        let t = Template::new("// Copyright (c) {year} Acme");
        let out = insert_header("fn main() {}\n", &t, 2026);
        assert!(has_header(&out, &t));
    }

    #[test]
    fn has_header_uses_the_shared_shebang_skip_logic() {
        let t = Template::new("# Copyright (c) {year} Acme");
        assert_eq!(crate::template::skip_shebang("#!/bin/sh\nrest"), "rest");
        assert!(!has_header("no header here\n", &t));
    }
}
