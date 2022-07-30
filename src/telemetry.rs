use tracing::Subscriber;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;

pub fn get_subsciber<Sink>(name: String, info: String, sink: Sink) -> impl Subscriber + Sync + Send where Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(info));
    let formatting_layer = BunyanFormattingLayer::new(
        name,
        // Output the formatted spans to stdout.
        sink
    );
    // The `with` method is provided by `SubscriberExt`, an extension
    // trait for `Subscriber` exposed by `tracing_subscriber`
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send ) {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");
    // `set_global_default` can be used by applications to specify
    // what subscriber should be used to process spans.
    set_global_default(subscriber).expect("Failed to set subscriber");
}
