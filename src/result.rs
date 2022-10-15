
pub struct ComputeResult {
    pub data: Vec<u8>,
    pub initialization_time: std::time::Duration,
    pub computation_time: std::time::Duration,
    pub data_fetch_time: std::time::Duration
}
