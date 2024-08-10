#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "WORKTIME_TYPE", rename_all = "lowercase")]
pub enum WorktimeType {
    Break,
    Ride,
    Work,
}

#[derive(async_graphql::SimpleObject)]
pub struct Worktime {
    pub worktime_id: i32,
    pub employee_id: i32,
    pub task_id: i32,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub timeduration: chrono::Duration,
    pub work_type: WorktimeType,
}
