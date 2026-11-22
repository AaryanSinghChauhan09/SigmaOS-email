//! SigmaOS email component — RFC 5322 message parsing utilities.
//!
//! Safe, allocation-conscious parsing of internet message headers and
//! MIME-lite body separation. No network access; pure parsing.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

/// Errors produced while parsing a message.
#[derive(Debug)]
pub enum EmailError {
    /// Input was not valid UTF-8/ASCII message text.
    InvalidEncoding,
    /// A required header was missing.
    MissingHeader(&'static str),
    /// Header block was malformed.
    MalformedHeader(String),
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEncoding => write!(f, "message is not valid text"),
            Self::MissingHeader(h) => write!(f, "missing required header: {h}"),
            Self::MalformedHeader(m) => write!(f, "malformed header: {m}"),
        }
    }
}

impl std::error::Error for EmailError {}

/// Parsed internet message.
#[derive(Debug, Clone)]
pub struct Message {
    /// Header fields in original order-insensitive map (lowercased keys).
    pub headers: BTreeMap<String, String>,
    /// Raw body after the blank line separating headers.
    pub body: String,
}

impl Message {
    /// Header lookup, case-insensitive.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_ascii_lowercase()).map(String::as_str)
    }

    /// Sender address from `From:` (raw, unvalidated).
    pub fn from(&self) -> Option<&str> { self.header("from") }

    /// Subject line.
    pub fn subject(&self) -> Option<&str> { self.header("subject") }

    /// Whether the message appears to be multipart MIME.
    pub fn is_multipart(&self) -> bool {
        self.header("content-type")
            .map(|v| v.to_ascii_lowercase().contains("multipart/"))
            .unwrap_or(false)
    }
}

/// Parse a raw RFC 5322 message into headers + body.
///
/// Handles folded (continuation) header lines beginning with space/tab.
/// Does not decode encoded-words or MIME bodies — that is a separate layer.
pub fn parse(raw: &str) -> Result<Message, EmailError> {
    let (head, body) = split_head_body(raw);
    let headers = parse_headers(head)?;
    Ok(Message { headers, body: body.to_string() })
}

fn split_head_body(raw: &str) -> (&str, &str) {
    // Header/body separator is a blank line (CRLF CRLF or LF LF).
    if let Some(idx) = raw.find("\r\n\r\n") {
        (&raw[..idx], &raw[idx + 4..])
    } else if let Some(idx) = raw.find("\n\n") {
        (&raw[..idx], &raw[idx + 2..])
    } else {
        (raw, "")
    }
}

fn parse_headers(head: &str) -> Result<BTreeMap<String, String>, EmailError> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut current_key: Option<String> = None;
    for line in head.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // folded continuation
            if let Some(key) = current_key.clone() {
                if let Some(entry) = map.get_mut(&key) {
                    entry.push(' ');
                    entry.push_str(line.trim());
                }
                continue;
            }
            return Err(EmailError::MalformedHeader("orphan continuation".into()));
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| EmailError::MalformedHeader(line.to_string()))?;
        let key = key.trim().to_ascii_lowercase();
        if key.is_empty() {
            return Err(EmailError::MalformedHeader(line.to_string()));
        }
        map.insert(key.clone(), value.trim().to_string());
        current_key = Some(key);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_message() {
        let raw = "From: alice@example.com\r\nSubject: Hello\r\n\r\nBody text\r\n";
        let msg = parse(raw).unwrap();
        assert_eq!(msg.from(), Some("alice@example.com"));
        assert_eq!(msg.subject(), Some("Hello"));
        assert_eq!(msg.body.trim(), "Body text");
    }

    #[test]
    fn unfolds_headers() {
        let raw = "Subject: This is a\r\n\tlong subject\r\n\r\nx";
        let msg = parse(raw).unwrap();
        assert_eq!(msg.subject(), Some("This is a long subject"));
    }

    #[test]
    fn detects_multipart() {
        let raw = "Content-Type: multipart/mixed; boundary=abc\r\n\r\n--abc";
        assert!(parse(raw).unwrap().is_multipart());
    }

    #[test]
    fn rejects_bad_header() {
        assert!(parse("no colon here\n\nbody").is_err());
    }
}
