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
    async fn timeduration(&self) -> Option<String> {
        match &self.timeduration {
            None => None,
            Some(interval) => {
                let months = interval.months;
                let days = interval.days;
                let seconds = interval.microseconds as f64 / 1_000_000_f64;
                Some(format!("P{}M{}DT0H0M{}S", months, days, seconds))
            },
        }
    }
}
