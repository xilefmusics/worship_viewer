#[derive(Default, Clone)]
pub struct Filter<'a> {
    id: Option<&'a str>,
    collection: Option<&'a str>,
}

impl<'a> Filter<'a> {
    pub fn new(id: Option<&'a str>, collection: Option<&'a str>) -> Self {
        Self { id, collection }
    }

    pub fn get_collection(&self) -> Option<&'a str> {
        self.collection
    }

    pub fn get_id(&self) -> Option<&'a str> {
        self.id
    }
}
