use std::sync::Arc;
use std::env;

use eyre::Result;
use log::LevelFilter;
use phtm::app::App;
use phtm::io::handler::IoAsyncHandler;
use pushr::push::instructions::{InstructionCache, InstructionSet};
use phtm::io::IoEvent;
use phtm::start_ui;

#[tokio::main]
async fn main() -> Result<()> {

    let args: Vec<String> = env::args().collect(); 
    if args.len() < 2 {
        return Ok(());
    }

    let bin = args[0].clone();
    let input = args[1].clone();

    println!("bin = {}", bin);
    println!("input = {}", input);

    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    // We need to share the App between thread
    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone(), bin, input)));
    let app_ui = Arc::clone(&app);

    // Configure log
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    // Handle IO in a specifc thread
    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    start_ui(&app_ui).await?;

    Ok(())
}
