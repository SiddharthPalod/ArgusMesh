use sled::Db;
use uuid::Uuid;
use crate::core::alert::Alert;

pub struct AlertStore{
    db: Db,
}

impl AlertStore{

    pub fn open(path: &str) -> sled::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn save_alert(&self, alert: &Alert) -> sled::Result<()> {
        let key = alert.alert_id.as_bytes();
        let value = bincode::serialize(alert).unwrap();
        self.db.insert(key, value)?;
        Ok(())
    }

    pub fn get_alert(&self, alert_id: &Uuid) -> sled::Result<Option<Alert>> {
        let key = alert_id.as_bytes();
        if let Some(bytes) = self.db.get(key)? {
            let alert: Alert = bincode::deserialize(&bytes).unwrap();
            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }

    pub fn all_alerts(&self) -> sled::Result<Vec<Alert>> {
        let mut alerts = Vec::new();
        for item in self.db.iter() {
            let (_, value) = item?;
            let alert: Alert = bincode::deserialize(&value).unwrap();
            alerts.push(alert);
        }
        Ok(alerts)
    }
}