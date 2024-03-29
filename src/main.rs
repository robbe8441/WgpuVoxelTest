#![windows_subsystem = "windows"]
use engine::{display_handler::GameWindow, *};

#[tokio::main]
async fn main() {
    let window = GameWindow::new().await;
    run(window).await;
}
