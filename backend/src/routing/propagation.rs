use super::envelope::Envelope;

pub fn should_forward(env: &Envelope) -> bool {
    if env.expired() {
        return false;
    }
    if env.hop_count >= env.max_hops {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::envelope::{Envelope, Priority};
    use std::time::{Duration, SystemTime};

    #[test]
    fn expired_messages_not_forwarded() {
        let mut env = Envelope::new("node".into(), Priority::Normal, vec![]);
        env.ttl_secs = 1;
        env.created_at = SystemTime::now() - Duration::from_secs(9999);

        assert!(!should_forward(&env));
    }

    #[test]
    fn hop_limit_blocks_forward() {
        let mut env = Envelope::new("node".into(), Priority::Normal, vec![]);
        env.hop_count = 5;
        env.max_hops = 5;

        assert!(!should_forward(&env));
    }

    #[test]
    fn valid_message_is_forwarded() {
        let mut env = Envelope::new("node".into(), Priority::Normal, vec![]);
        env.hop_count = 2;
        env.max_hops = 5;
        env.ttl_secs = 1000;

        assert!(should_forward(&env));
    }
}
