use crate::routing::router::Router;
use crate::transport::Transport;
use crate::storage::Store;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;

pub struct Node<T: Transport + 'static, S: Store + 'static> {
    router: Arc<Mutex<Router<T, S>>>,
    transport: Arc<T>,
}

impl<T: Transport, S: Store> Node<T, S> {
    pub fn new(router: Arc<Mutex<Router<T, S>>>, transport: Arc<T>) -> Self {
        Self { router, transport }
    }

    pub async fn run(&self) {
        let mut interval = time::interval(Duration::from_millis(10));
        loop {
            tokio::select! {
                // Inbound processing
                recv_res = self.transport.recv() => {
                     match recv_res {
                         Ok(data) => {
                             let mut router = self.router.lock().unwrap();
                             router.receive(&data);
                         }
                         Err(_) => {
                              // If recv fails (e.g. channel closed or error), standard backoff
                              time::sleep(Duration::from_millis(100)).await;
                         }
                     }
                }

                // Outbound processing (periodic)
                _ = interval.tick() => {
                    for _ in 0..5 {
                        let bytes_opt = {
                            let mut router_lock = self.router.lock().unwrap();
                            router_lock.next_outbound()
                        };
    
                        if let Some(bytes) = bytes_opt {
                            let _ = self.transport.send(bytes).await;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }
}