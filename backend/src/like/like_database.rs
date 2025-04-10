use super::Like;
use fancy_surreal::RecordId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LikeDatabase {
    pub id: Option<String>,
    pub song: RecordId,
}

impl Into<Like> for LikeDatabase {
    fn into(self) -> Like {
        Like {
            id: self.id,
            song: self.song.key().to_string(),
        }
    }
}

impl Into<LikeDatabase> for Like {
    fn into(self) -> LikeDatabase {
        LikeDatabase {
            id: self.id,
            song: RecordId::from_table_key("songs", self.song),
        }
    }
}

impl fancy_surreal::Databasable for LikeDatabase {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}
