use payeng::prelude::runtime;
use payeng::telemetry::Tracer;

const CAPACITY: usize = 10_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Tracer::new("payeng", "info").init_subscriber(std::io::stderr)?;
    let reader = runtime::get_input()?;
    runtime::run(reader, std::io::stdout(), CAPACITY);
    Ok(())
}
