use std::sync::LazyLock;
use std::time::Duration;

// TODO(TFL): swap the contact URL once the real TFLives domain exists.
pub const USER_AGENT: &str = "TFLClient/0.1.0 (+https://tflives.example)";

pub static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(8)
        .build()
        .expect("Failed to create HTTP client")
});
