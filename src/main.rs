use engine::{display_handler::GameWindow, *};

#[tokio::main]
async fn main() {
    env_logger::init();
    let window = GameWindow::new().await;

    //debugger::setup(window);
    run(window).await;
}
