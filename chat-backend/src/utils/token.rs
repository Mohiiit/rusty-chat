#[derive(Clone, Debug)]
pub struct Ctx {
    username: String,
}

// Constructor.
impl Ctx {
    pub fn new(username: String) -> Self {
        Self { username }
    }
}

// Property Accessors.
impl Ctx {
    pub fn username(&self) -> &String {
        &self.username
    }
}
