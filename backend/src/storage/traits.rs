pub trait Store: Send + Sync {
    fn persist(&self, key: Vec<u8>, val: Vec<u8>);
    fn load_all(&self) -> Vec<Vec<u8>>;
    fn remove(&self, key: Vec<u8>);
}
