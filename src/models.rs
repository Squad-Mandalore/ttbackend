use async_graphql::{Enum, SimpleObject};

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum WorktimeType {
    Break,
    Ride,
    Work,
}

#[derive(SimpleObject)]
pub struct Worktime {
    // pub worktime_id: i32,
    pub employee_id: i32,
    // pub task_id: i32,
    // pub start_time: chrono::DateTime<chrono::Utc>,
    // pub stop_time: chrono::DateTime<chrono::Utc>,
    // pub timeduration: chrono::Duration,
    // pub worktime_type: WorktimeType,
}
