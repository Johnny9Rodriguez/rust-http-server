#![allow(dead_code, unused_variables)]
use std::collections::HashMap;

#[derive(Debug)]
struct Headers(HashMap<String, String>);

impl Headers {
    fn new() -> Self {
        Headers(HashMap::new())
    }

    fn parse(&mut self, data: &[u8]) -> (usize, bool, Option<String>) {
        let s = match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(err) => {
                return (
                    0,
                    false,
                    Some("Unable to decode data as UTF-8 string: {err}".to_string()),
                );
            }
        };

        if let Some(n) = s.find("\r\n") {
            if n == 0 {
                return (2, true, None);
            }

            let line = &s[..n].trim();
            let mut parts = line.splitn(2, ':');
            let key = parts.next();
            let value = parts.next().map(str::trim);

            match (key, value) {
                (Some(k), Some(v)) if !k.is_empty() && !k.chars().any(char::is_whitespace) => {
                    self.0.insert(k.to_string(), v.to_string());
                    return (n + 2, false, None);
                }
                _ => {
                    return (
                        0,
                        false,
                        Some("Invalid header format: expected `Key: Value`".to_string()),
                    );
                }
            }
        }

        (0, false, None)
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}

#[cfg(test)]
mod tests {
    use crate::headers::Headers;

    #[test]
    fn test_valid_single_header() {
        let mut headers = Headers::new();
        let data = b"Host: localhost:42069\r\n\r\n";
        let (n, done, err) = headers.parse(data);

        assert!(err.is_none());
        assert_eq!(headers.get("Host"), Some(&"localhost:42069".to_string()));
        assert_eq!(n, 23);
        assert!(!done);
    }

    #[test]
    fn test_invalid_spacing_header() {
        let mut headers = Headers::new();
        let data = b"       Host : localhost:42069       \r\n\r\n";
        let (n, done, err) = headers.parse(data);

        assert!(err.is_some(), "expected error for invalid spacing");
        assert_eq!(n, 0);
        assert!(!done);
    }

    #[test]
    fn test_single_header_with_extra_whitespace() {
        let mut headers = Headers::new();
        let data = b"   Host: localhost:42069    \r\n\r\n";
        let (n, done, err) = headers.parse(data);

        assert!(err.is_none());
        assert_eq!(headers.get("Host"), Some(&"localhost:42069".to_string()));
        assert_eq!(n, 30);
        assert!(!done);
    }

    #[test]
    fn test_valid_two_headers() {
        let mut headers = Headers::new();
        let data = b"   Host: localhost:42069    \r\nContent-Type: application/json\r\n\r\n";
        let (n, done, err) = headers.parse(data);

        assert!(err.is_none());
        assert_eq!(headers.get("Host"), Some(&"localhost:42069".to_string()));
        assert_eq!(n, 30);
        assert!(!done);

        let (n, done, err) = headers.parse(&data[30..]);

        assert!(err.is_none());
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(n, 32);
        assert!(!done);
    }

    #[test]
    fn test_valid_done() {
        let mut headers = Headers::new();
        let data = b"\r\n";
        let (n, done, err) = headers.parse(data);

        assert!(err.is_none());
        assert_eq!(n, 2);
        assert!(done);
    }

    #[test]
    fn test_valid_emoji() {
        let mut headers = Headers::new();
        let data = "Emoji: 😄\r\n".as_bytes();
        let (n, done, err) = headers.parse(data);

        assert!(err.is_none());
        assert_eq!(headers.get("Emoji"), Some(&"😄".to_string()));
        assert_eq!(n, 13);
        assert!(!done);
    }
}
