use payeng::prelude::runtime;
use payeng::telemetry::Tracer;

const CAPACITY: usize = 10_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(level_filter) = std::env::var("RUST_LOG") {
        if level_filter.parse::<tracing::Level>().is_ok() {
            Tracer::new("payeng", &level_filter).init_subscriber(std::io::stderr)?;
        }
    }

    let reader = runtime::get_input()?;
    runtime::run(reader, std::io::stdout(), CAPACITY);
    Ok(())
}
