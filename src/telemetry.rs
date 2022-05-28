//! Tracer type.
//!
//! The telemetry module defines the [`Tracer`] type.

use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

use crate::error::Error;

pub struct Tracer<'a> {
    name: &'a str,
    env_filter: &'a str,
}

impl<'a> Tracer<'a> {
    /// Creates new [`Tracer`] instance.
    pub fn new(name: &'a str, env_filter: &'a str) -> Self {
        Self { name, env_filter }
    }

    /// Initializes the underline `Subscriber`.
    pub fn init_subscriber<Sink>(&self, sink: Sink) -> Result<(), Error>
    where
        Sink: for<'b> MakeWriter<'b> + Send + Sync + 'static,
    {
        let formatting_layer = BunyanFormattingLayer::new(self.name.into(), sink);
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(self.env_filter));

        let subscriber = Registry::default()
            .with(env_filter)
            .with(JsonStorageLayer)
            .with(formatting_layer);
        set_global_default(subscriber)?;
        Ok(())
    }
}
