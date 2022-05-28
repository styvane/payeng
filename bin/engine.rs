use std::env;

use payeng::telemetry::Tracer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Tracer::new("payeng", "info").init_subscriber(std::io::stderr)?;
    Ok(())
}
