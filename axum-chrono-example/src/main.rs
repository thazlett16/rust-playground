use axum::Router;
use tracing::debug;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod error {
    use tracing::error;

    #[derive(Debug)]
    pub enum Error {
        IOError,
    }

    impl core::fmt::Display for Error {
        fn fmt(
            &self,
            fmt: &mut core::fmt::Formatter,
        ) -> core::result::Result<(), core::fmt::Error> {
            write!(fmt, "{self:?}")
        }
    }

    impl std::error::Error for Error {}

    impl From<std::io::Error> for Error {
        fn from(value: std::io::Error) -> Self {
            error!("IOError: \n\n{}", value);

            Self::IOError
        }
    }

    pub type Result<T> = core::result::Result<T, Error>;
}

mod ping_rfc3339 {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum::{Json, Router};
    use serde::{Deserialize, Serialize};
    use std::ops::Add;
    use chrono::{DateTime, Utc, Duration};
    use tracing::info;

    #[derive(Clone, Debug, Serialize)]
    // #[serde(rename_all = "camelCase")]
    struct PingResponse {
        message: String,
        always_null: Option<String>,
        current_date_time: DateTime<Utc>,
    }

    impl PingResponse {
        fn new(message: String) -> Self {
            Self {
                message,
                always_null: None,
                current_date_time: Utc::now(),
            }
        }
    }

    impl Default for PingResponse {
        fn default() -> Self {
            Self::new(String::from("API is responsive"))
        }
    }

    async fn get_ping_response() -> impl IntoResponse {
        info!("get_ping_response");

        (StatusCode::OK, Json(PingResponse::default())).into_response()
    }

    #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
    // #[serde(rename_all = "camelCase")]
    struct PingRequest {
        message_optional: Option<String>,
        current_date_time: Option<DateTime<Utc>>,
    }

    async fn post_ping_response(Json(request): Json<PingRequest>) -> impl IntoResponse {
        info!("post_ping_response - request: {:?}", request);

        let mut response = PingResponse::default();
        if let Some(message_optional) = request.message_optional {
            response.message = message_optional.to_ascii_uppercase();
        }
        if let Some(current_date_time) = request.current_date_time {
            response.current_date_time = current_date_time.add(Duration::days(5));
        }

        (StatusCode::OK, Json(response)).into_response()
    }

    pub fn new_ping_router_rfc3339() -> Router {
        Router::new().route(
            "/ping/chrono/rfc3339",
            get(get_ping_response).post(post_ping_response),
        )
    }
}

#[tokio::main]
async fn main() -> error::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new("debug"))
        .init();

    debug!("Logging Configured");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    let address = listener.local_addr()?;

    debug!("listening on {:?}", address);

    let router = Router::new()
        .merge(ping_rfc3339::new_ping_router_rfc3339());

    debug!("router created");

    axum::serve(listener, router).await?;

    Ok(())
}
