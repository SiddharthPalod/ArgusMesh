
#[derive(Debug)]
pub enum TransportError {
    NotConnected,
    Timeout,
    IoError,
    FragmentError,
    Internal(String),
}