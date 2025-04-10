use super::Collection;
use crate::song::LinkDatabase as SongLinkDatabase;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectionDatabase {
    pub id: Option<String>,
    pub title: String,
    pub cover: String,
    pub songs: Vec<SongLinkDatabase>,
}

impl Into<Collection> for CollectionDatabase {
    fn into(self) -> Collection {
        Collection {
            id: self.id,
            title: self.title,
            cover: self.cover,
            songs: self.songs.into_iter().map(|link| link.into()).collect(),
        }
    }
}

impl Into<CollectionDatabase> for Collection {
    fn into(self) -> CollectionDatabase {
        CollectionDatabase {
            id: self.id,
            title: self.title,
            cover: self.cover,
            songs: self.songs.into_iter().map(|link| link.into()).collect(),
        }
    }
}

impl fancy_surreal::Databasable for CollectionDatabase {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}
