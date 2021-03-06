#![allow(dead_code)]

#[macro_use]
extern crate log;

use std::result::Result as StdResult;

use dotenv::dotenv;

use crate::cerebot::{Cerebot, RunResult};
use crate::error::Error;

mod cerebot;
mod config;
mod dispatch;
mod error;
mod event;
mod handlers;
mod state;
mod template_renderer;
mod util;

type Result<T> = StdResult<T, Error>;

fn main() {
    dotenv().ok();
    env_logger::init();
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .core_threads(8)
        .build()
        .unwrap();

    runtime.block_on(async move {
        let mut bot = Cerebot::create().unwrap();
        while let RunResult::Restart = bot.run().await.unwrap() {}
    });
}
