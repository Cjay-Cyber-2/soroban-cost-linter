pub struct UnnecessaryStringToBytes;

impl UnnecessaryStringToBytes {
    pub fn check(code: &str) -> bool {
        code.contains("to_bytes()") && code.contains("String")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unnecessary_string_to_bytes_positive() {
        let code = "let s = String::from(\"abc\"); let b = s.to_bytes();";
        assert!(UnnecessaryStringToBytes::check(code));
    }

    #[test]
    fn test_unnecessary_string_to_bytes_negative() {
        let code = "let b = vec![1u8, 2u8];";
        assert!(!UnnecessaryStringToBytes::check(code));
    }

    #[test]
    fn test_edge_case_empty() {
        assert!(!UnnecessaryStringToBytes::check(""));
    }
}
