use futures::pin_mut;
use tokio_util::sync::CancellationToken;

pub mod config;
mod task;

pub struct BhServerBuilder {
    config: config::BhServerConfig,
    cancellation_token: Option<CancellationToken>,
}

impl BhServerBuilder {
    pub fn new(
        config: config::BhServerConfig,
    ) -> Self {
        Self {
            config,
            cancellation_token: None,
        }
    }

    pub fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation_token = Some(token);
        self
    }

    pub fn build(self) -> BhServer {
        let cancellation_token = self.cancellation_token.unwrap_or_else(|| CancellationToken::new());

        let task = task::BhServerTask::new(
            self.config.listen().clone(),
            cancellation_token,
        );

        let _join_token = tokio::spawn(
            async move {
                pin_mut!(task);

                if let Err(err) = task.run().await {
                    tracing::error!("bH Server Task failed: {:?}", err);
                }

                tracing::info!("bH Server Task exited.");
            }
        );

        BhServer {}
    }
}

pub struct BhServer {

}