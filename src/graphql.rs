use async_graphql::{extensions::Logger, EmptySubscription, MergedObject, Schema};
use sqlx::{Pool, Postgres};

mod timer;
mod world;

#[derive(MergedObject, Default)]
pub struct Query(timer::Timer, world::World);

#[derive(MergedObject, Default)]
pub struct Mutation(timer::TimerMutation);

pub type SchemaType = Schema<Query, Mutation, EmptySubscription>;

pub fn create_schema(database_pool: Pool<Postgres>) -> SchemaType {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(Logger)
        .data(database_pool)
        .finish()
}
