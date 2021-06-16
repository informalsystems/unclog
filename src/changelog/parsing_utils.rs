//! Utilities to assist in parsing changelogs.

use crate::error::Error;

pub(crate) fn trim_newlines(s: &str) -> &str {
    s.trim_end_matches(|c| c == '\n' || c == '\r')
}

pub(crate) fn extract_release_version(s: &str) -> crate::Result<&str> {
    // Just find the first digit in the string
    let version_start = s
        .chars()
        .position(|c| ('0'..='9').contains(&c))
        .ok_or_else(|| Error::CannotExtractVersion(s.to_owned()))?;
    Ok(&s[version_start..])
}

#[cfg(test)]
mod test {
    use super::extract_release_version;

    #[test]
    fn release_version_extraction() {
        let cases = vec![
            ("v0.1.0", "0.1.0"),
            ("0.1.0", "0.1.0"),
            ("v0.1.0-beta.1", "0.1.0-beta.1"),
        ];

        for (s, expected) in cases {
            let actual = extract_release_version(s).unwrap();
            assert_eq!(expected, actual);
        }

        assert!(extract_release_version("no-version").is_err());
    }
}
