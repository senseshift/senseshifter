use tokio_util::sync::CancellationToken;
use ss_bh::server::BhServerBuilder;
use ss_bh::server::config::BhServerConfig;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = BhServerConfig::default();

    let cancellation_token = CancellationToken::new();
    let builder = BhServerBuilder::new(config)
        .with_cancellation_token(cancellation_token.clone())
        .with_sniff_into(std::path::PathBuf::from("data"));

    let server = builder.build();

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                cancellation_token.cancel();
                break;
            }
        }
    }

}