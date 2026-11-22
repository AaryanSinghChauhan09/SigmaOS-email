//! sigmaos-email CLI — parse a message file and print summary.

use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("usage: sigmaos-email <message-file>")?;
    let raw = fs::read_to_string(path)?;
    let msg = sigmaos_email::parse(&raw)?;
    println!("From:    {}", msg.from().unwrap_or("(none)"));
    println!("Subject: {}", msg.subject().unwrap_or("(none)"));
    println!("Multipart: {}", msg.is_multipart());
    println!("Body bytes: {}", msg.body.len());
    Ok(())
}
