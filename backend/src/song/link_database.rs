use super::{Link, SimpleChord};
use fancy_surreal::RecordId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkDatabase {
    id: RecordId,
    nr: Option<String>,
    key: Option<SimpleChord>,
}

impl Into<Link> for LinkDatabase {
    fn into(self) -> Link {
        Link {
            id: self.id.key().to_string(),
            nr: self.nr,
            key: self.key,
        }
    }
}

impl Into<LinkDatabase> for Link {
    fn into(self) -> LinkDatabase {
        LinkDatabase {
            id: RecordId::from_table_key("songs", self.id),
            nr: self.nr,
            key: self.key,
        }
    }
}
