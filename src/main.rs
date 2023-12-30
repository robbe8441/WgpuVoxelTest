use engine::*;

fn main() {
    env_logger::init();

    let rt = tokio::runtime::Runtime::new().unwrap();

    let game = create_window();
    rt.block_on(game);
}
