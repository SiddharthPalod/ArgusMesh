use crate::routing::priority_queue::PriorityQueue;
use crate::routing::envelope::{Envelope, Priority};
// use uuid::Uuid;

#[test]
fn stress_100_alerts_queue(){
    let mut q = PriorityQueue::new();

    for _ in 0..100 {
        q.push(Envelope::new(
            "n".into(),
            Priority::Normal,
            vec![1,2,3],
        ));
    }

    let mut count = 0;
    while q.pop().is_some() {
        count += 1;
    }
    assert_eq!(count, 100);
}
