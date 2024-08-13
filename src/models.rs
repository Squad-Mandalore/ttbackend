use sqlx::postgres::types;

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq, Debug)]
#[derive(sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum WorktimeType {
    Break,
    Ride,
    Work,
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct Worktime {
    pub worktime_id: i32,
    pub employee_id: i32,
    pub task_id: i32,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    #[graphql(skip)]
    pub timeduration: Option<types::PgInterval>,
    pub work_type: Option<WorktimeType>,
}

#[async_graphql::ComplexObject]
impl Worktime {
    async fn timeduration(&self) -> String {
        let mut months = 0;
        let mut days = 0;
        let mut seconds = 0;
        if let Some(interval) = &self.timeduration {
            months = interval.months;
            days = interval.days;
            seconds = (interval.microseconds as f64 / 1_000_000_f64).round() as i32
        }
        format!("P{}M{}DT0H0M{}S", months, days, seconds)
    }
}
