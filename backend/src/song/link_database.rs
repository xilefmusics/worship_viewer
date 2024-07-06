use super::{Link, SimpleChord};
use fancy_surreal::{Id, RecordId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LinkDatabase {
    id: RecordId,
    nr: Option<String>,
    key: Option<SimpleChord>,
}

impl Into<Link> for LinkDatabase {
    fn into(self) -> Link {
        Link {
            id: self.id.id.to_string(),
            nr: self.nr,
            key: self.key,
        }
    }
}

impl Into<LinkDatabase> for Link {
    fn into(self) -> LinkDatabase {
        LinkDatabase {
            id: RecordId {
                tb: "songs".to_string(),
                id: Id::String(self.id),
            },
            nr: self.nr,
            key: self.key,
        }
    }
}
