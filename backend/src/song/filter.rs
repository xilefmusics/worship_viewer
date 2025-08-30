#[derive(Default, Clone, Debug)]
pub struct Filter<'a> {
    id: Option<&'a str>,
    collection: Option<&'a str>,
    setlist: Option<&'a str>,
}

impl<'a> Filter<'a> {
    pub fn new(id: Option<&'a str>, collection: Option<&'a str>, setlist: Option<&'a str>) -> Self {
        Self { id, collection, setlist }
    }

    pub fn get_setlist(&self) -> Option<&'a str> {
        self.setlist
    }

    pub fn get_collection(&self) -> Option<&'a str> {
        self.collection
    }

    pub fn get_id(&self) -> Option<&'a str> {
        self.id
    }
}
