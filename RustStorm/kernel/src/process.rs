
pub struct Process {
    process_id: usize
}

impl Process {
    pub fn create() -> Self {
        Self {
            process_id: 0
        }
    }
}