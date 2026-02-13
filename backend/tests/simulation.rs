use backend::routing::router::Router;
use backend::transport::test_sim::{SimTransport, SimConfig};
use backend::routing::envelope::{Envelope, Priority};
use backend::storage::Store;
use backend::crypto::keys::NodeKeys;

use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::test]
async fn test_simulated_network_flow() {
    let (tx1, mut rx1) = mpsc::channel(100);
    let (_tx2, rx2) = mpsc::channel(100); // dummy peer side

    let cfg = SimConfig {
        latency_ms: 50,
        drop_rate: 0.0,
    };

    let t1 = SimTransport::new(tx1, rx2, cfg);

    struct MemStore(Arc<Mutex<Vec<Vec<u8>>>>);

    impl Store for MemStore {
        fn persist(&self, _k: Vec<u8>, v: Vec<u8>) {
            self.0.lock().unwrap().push(v);
        }

        fn load_all(&self) -> Vec<Vec<u8>> {
            self.0.lock().unwrap().clone()
        }

        fn remove(&self, _k: Vec<u8>) {}
    }

    let s1 = MemStore(Arc::new(Mutex::new(vec![])));

    let keys = NodeKeys::generate();
    let sym_key = [7u8; 32];

    let mut router = Router::new(t1, s1, keys, sym_key);

    let env = Envelope::new("node2".into(), Priority::High, vec![1, 2, 3, 4]);
    router.enqueue_local(env).expect("enqueue_local should succeed");

    router.tick().await;

    match tokio::time::timeout(Duration::from_millis(300), rx1.recv()).await {
        Ok(Some(msg)) => {
            let received_env: Envelope = bincode::deserialize(&msg).unwrap();

            assert!(!received_env.payload.is_empty());

            assert_eq!(received_env.priority, Priority::High);

            assert_ne!(received_env.sender_pubkey, [0u8; 32]);
            assert_eq!(received_env.signature.len(), 64);
            assert_ne!(received_env.signature.as_slice(), [0u8; 64].as_slice());
            assert_ne!(received_env.nonce, [0u8; 12]);
        }

        Ok(None) => panic!("Channel closed unexpectedly"),
        Err(_) => panic!("Did not receive message in time"),
    }
}
