use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn init_tracing() {
  use std::io::IsTerminal;
  dotenv().ok();
  let pkg_name = env!("CARGO_PKG_NAME");
  let format = tracing_subscriber::fmt::format()
    .with_level(true)
    .with_thread_names(true)
    .with_thread_ids(false)
    .with_target(false)
    .with_file(false)
    .with_line_number(false)
    .compact();
  let stderr_subscriber = Registry::default()
    .with(
      EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into())
        .add_directive(format!("{}=debug", pkg_name).parse().unwrap())
        .add_directive("multifiledownloader=debug".parse().unwrap()),
    )
    .with(
      tracing_subscriber::fmt::layer()
        .with_ansi(std::io::stderr().is_terminal())
        .with_writer(std::io::stderr)
        .event_format(format.clone()),
    );

  tracing::subscriber::set_global_default(stderr_subscriber).unwrap();
}
