use async_graphql::Object;

pub struct Hello;

#[Object]
impl Hello {
    async fn hello(&self) -> String {
        "hello".to_string()
    }
}

impl Default for Hello {
    fn default() -> Self {
        Hello
    }
}
