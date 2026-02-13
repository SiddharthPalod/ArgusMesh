use tokio::time::{sleep, Duration};
use super::priority_queue::PriorityQueue;
use super::envelope::Envelope;

pub async fn run_scheduler<F>(queue: &mut PriorityQueue, mut send:F)
where
    F: FnMut(Envelope) -> bool,
{
    loop {
        if let Some(env) = queue.pop() {
            send(env);
        }
        else{
            sleep(Duration::from_millis(50)).await;
        }
    }
}