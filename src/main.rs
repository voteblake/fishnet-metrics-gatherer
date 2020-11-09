use lambda::{handler_fn, Context};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
// use tracing::{info, Level};
// use tracing_subscriber::FmtSubscriber;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Serialize, Deserialize, Debug)]
struct FishnetStatus {
    analysis: FishnetAnalysis,
}

#[derive(Serialize, Deserialize, Debug)]
struct FishnetAnalysis {
    user: FishnetQueueMetric,
    system: FishnetQueueMetric,
}

#[derive(Serialize, Deserialize, Debug)]
struct FishnetQueueMetric {
    acquired: u64,
    queued: u64,
    oldest: u64,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // let subscriber = FmtSubscriber::builder()
    //     .with_max_level(Level::DEBUG)
    //     .finish();
    // tracing::subscriber::set_global_default(subscriber).expect("Could not set global subscriber");
    let func = handler_fn(func);
    lambda::run(func).await?;
    Ok(())
}

async fn func(event: Value, _: Context) -> Result<Value, Error> {
    let status = reqwest::get("https://lichess.org/fishnet/status")
        .await?
        .json::<FishnetStatus>()
        .await?;
    // info!("got status: {:?}", status);
    println!("got status: {:?}", status);
    Ok(event)
}
