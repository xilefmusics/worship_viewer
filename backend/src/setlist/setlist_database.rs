use super::Setlist;
use crate::song::LinkDatabase as SongLinkDatabase;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetlistDatabase {
    pub id: Option<String>,
    pub title: String,
    pub songs: Vec<SongLinkDatabase>,
}

impl Into<Setlist> for SetlistDatabase {
    fn into(self) -> Setlist {
        Setlist {
            id: self.id,
            title: self.title,
            songs: self.songs.into_iter().map(|link| link.into()).collect(),
        }
    }
}

impl Into<SetlistDatabase> for Setlist {
    fn into(self) -> SetlistDatabase {
        SetlistDatabase {
            id: self.id,
            title: self.title,
            songs: self.songs.into_iter().map(|link| link.into()).collect(),
        }
    }
}

impl fancy_surreal::Databasable for SetlistDatabase {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}
