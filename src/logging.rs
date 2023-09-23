use tracing::warn;
use tracing_subscriber::filter::FromEnvError;

pub fn init() -> Result<(), FromEnvError> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,actix_server::worker=WARN"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    color_eyre::install().unwrap_or_else(|e| {
        warn!("Colorized error reporting is not available: {e}");
    });

    Ok(())
}
