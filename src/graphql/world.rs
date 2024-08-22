use async_graphql::Object;

pub struct World;

#[Object]
impl World {
    async fn world(&self) -> String {
        "world".to_string()
    }
}

impl Default for World {
    fn default() -> Self {
        World
    }
}
