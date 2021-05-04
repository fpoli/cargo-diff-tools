use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub type FileChanges = HashMap<String, Vec<(usize, usize)>>;

/// Return for each file an ordered list of (start, len) intervals of modified lines.
pub fn parse_diff(diff: &str) -> Result<FileChanges> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"^\+\+\+ .?/(?P<filePath>.*)\s*$|^@@ -[0-9]+(,[0-9]+)? \+(?P<linesFrom>[0-9]+)(,(?P<linesLen>[0-9]+))? @@"
        ).expect("Failed to parse regex");
    }

    let mut file_changes: FileChanges = HashMap::new();
    let mut curr_file_path = None;
    for line in diff.lines() {
        if let Some(cap) = RE.captures(line) {
            if let Some(file_path_match) = cap.name("filePath") {
                let file_path = file_path_match.as_str().to_string();
                file_changes.insert(file_path.clone(), vec![]);
                curr_file_path = Some(file_path);
            }
            if let Some(lines_from_match) = cap.name("linesFrom") {
                let from = lines_from_match
                    .as_str()
                    .parse::<usize>()
                    .with_context(|| {
                        format!("Failed to parse start of lines range (line: {:?})", line)
                    })?;
                let len = if let Some(lines_len_match) = cap.name("linesLen") {
                    lines_len_match.as_str().parse::<usize>().with_context(|| {
                        format!("Failed to parse length of lines range (line: {:?})", line)
                    })?
                } else {
                    1
                };
                let curr_file_path_ref = curr_file_path
                    .as_ref()
                    .with_context(|| "Failed to retrieve current file path")?;
                file_changes
                    .get_mut(curr_file_path_ref)
                    .with_context(|| {
                        format!(
                            "Failed to retrieve ranges of file path {:?}",
                            curr_file_path_ref
                        )
                    })?
                    .push((from, len));
            }
        }
    }

    Ok(file_changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_parse_diff_1() {
        let diff = indoc! {"
            +++ b/Cargo.lock
            @@ -3,0 +4,9 @@
            @@ -5,0 +15,26 @@
            +++ b/Cargo.toml
            @@ -9,0 +10 @@
            +++ b/src/main.rs
            @@ -2,0 +3,3 @@
            @@ -8 +11 @@
            @@ -13,2 +16,15 @@
        "};
        let file_changes = parse_diff(diff).unwrap();
        eprintln!("{:?}", file_changes);
        assert_eq!(file_changes.len(), 3);
        assert_eq!(&file_changes["Cargo.lock"], &[(4, 9), (15, 26)]);
        assert_eq!(&file_changes["Cargo.toml"], &[(10, 1)]);
        assert_eq!(&file_changes["src/main.rs"], &[(3, 3), (11, 1), (16, 15)]);
    }

    #[test]
    fn test_parse_diff_2() {
        let diff = indoc! {"
            +++ b/prusti-viper/src/encoder/mir_encoder/mod.rs
            @@ -98,5 +98,5 @@
        "};
        let file_changes = parse_diff(diff).unwrap();
        eprintln!("{:?}", file_changes);
        assert_eq!(file_changes.len(), 1);
        assert_eq!(&file_changes["prusti-viper/src/encoder/mir_encoder/mod.rs"], &[(98, 5)]);
    }
}
