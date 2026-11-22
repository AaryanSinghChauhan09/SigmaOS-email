//! SigmaOS component binary: email

use sigmaos_email::Component;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Component::new().operate()?;
    println!("sigmaos-email: ok");
    Ok(())
}
