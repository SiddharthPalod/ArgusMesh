use crate::routing::dedup::DedupCache;
use uuid::Uuid;

#[test]
fn churn_simulation_dedup_stability() {
    let mut d = DedupCache::new(1000);
    for _ in 0..500 {
        let id = Uuid::new_v4();

        assert!(!d.seen(id)); 
        assert!(d.seen(id)); 
    }
}
