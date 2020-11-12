use lambda::{handler_fn, Context};
use once_cell::sync::OnceCell;
use reqwest;
use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, Dimension, MetricDatum, PutMetricDataInput};
use rusoto_core::Region;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use Default;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum FishnetStatus {
    Analysis {
        user: FishnetQueueMetric,
        system: FishnetQueueMetric,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct FishnetQueueMetric {
    acquired: u64,
    queued: u64,
    oldest: u64,
}

fn cloudwatch_client() -> &'static CloudWatchClient {
    static INSTANCE: OnceCell<CloudWatchClient> = OnceCell::new();
    INSTANCE.get_or_init(|| CloudWatchClient::new(Region::default()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global subscriber");
    let func = handler_fn(func);
    lambda::run(func).await?;
    Ok(())
}

async fn func(event: Value, _: Context) -> Result<Value, Error> {
    let metrics = match reqwest::get("https://lichess.org/fishnet/status")
        .await?
        .json::<FishnetStatus>()
        .await?
    {
        FishnetStatus::Analysis { user, system } => PutMetricDataInput {
            namespace: "fishnet".to_string(),
            metric_data: vec![
                MetricDatum {
                    metric_name: "queued".to_string(),
                    dimensions: Some(vec![Dimension {
                        name: "queue".to_string(),
                        value: "user".to_string(),
                    }]),
                    value: Some(user.queued as f64),
                    ..Default::default()
                },
                MetricDatum {
                    metric_name: "queued".to_string(),
                    dimensions: Some(vec![Dimension {
                        name: "queue".to_string(),
                        value: "system".to_string(),
                    }]),
                    value: Some(system.queued as f64),
                    ..Default::default()
                },
                MetricDatum {
                    metric_name: "oldest".to_string(),
                    dimensions: Some(vec![Dimension {
                        name: "queue".to_string(),
                        value: "user".to_string(),
                    }]),
                    value: Some(user.oldest as f64),
                    ..Default::default()
                },
                MetricDatum {
                    metric_name: "oldest".to_string(),
                    dimensions: Some(vec![Dimension {
                        name: "queue".to_string(),
                        value: "system".to_string(),
                    }]),
                    value: Some(system.oldest as f64),
                    ..Default::default()
                },
                MetricDatum {
                    metric_name: "acquired".to_string(),
                    dimensions: Some(vec![Dimension {
                        name: "queue".to_string(),
                        value: "user".to_string(),
                    }]),
                    value: Some(user.acquired as f64),
                    ..Default::default()
                },
                MetricDatum {
                    metric_name: "acquired".to_string(),
                    dimensions: Some(vec![Dimension {
                        name: "queue".to_string(),
                        value: "system".to_string(),
                    }]),
                    value: Some(system.acquired as f64),
                    ..Default::default()
                },
            ],
        },
    };
    info!("got status: {:?}", metrics);

    cloudwatch_client().put_metric_data(metrics).await?;

    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn des() {
        assert!(serde_json::from_str::<FishnetStatus>(r#"
        {"analysis":{"user":{"acquired":31,"queued":0,"oldest":0},"system":{"acquired":73,"queued":0,"oldest":0}}}
        "#).is_ok())
    }
}
