use engine::{*, display_handler::GameWindow};

#[tokio::main]
async fn main() {
    env_logger::init();
    let window = GameWindow::new().await;
    run(window).await;
}
