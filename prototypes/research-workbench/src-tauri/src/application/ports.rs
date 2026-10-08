/// Values supplied by the composition root, never IPC filesystem capabilities.
pub trait Clock: Send + Sync {
    fn now_utc(&self) -> String;
}
pub trait IdGenerator: Send + Sync {
    fn next_uuid(&self) -> String;
}
pub struct SystemClock;
impl Clock for SystemClock {
    fn now_utc(&self) -> String {
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }
}
pub struct RandomIds;
impl IdGenerator for RandomIds {
    fn next_uuid(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
