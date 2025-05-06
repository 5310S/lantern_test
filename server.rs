
use crate::blockchain::Blockchain;
use crate::rate_limit::rate_limited;
use crate::routes::build_routes;
use crate::storage::{load_chain, save_chain};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task;
use warp::Filter;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let chain = Arc::new(Mutex::new(load_chain().unwrap_or_else(Blockchain::new)));
    let chain_status = chain.clone();
    let chain_for_filter = chain.clone();

    let routes = build_routes(chain_status, rate_limited().map(|_| ())).with(warp::log::custom(|info| {
        println!("📥 {} {} {}", info.method(), info.path(), info.status());
    }));

    let chain_clone = chain.clone();
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            let c = chain_clone.lock().unwrap();
            save_chain(&*c);
        }
    });

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
    Ok(())
}
