// Explicit opt-in real-network acceptance; never runs against a production profile.
#[tokio::main]
async fn main() {
    tflclient_lib::activity_install_acceptance().await;
}
