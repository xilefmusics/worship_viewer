use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct TocItem {
    pub idx: usize,
    pub title: String,
    pub nr: String,
    pub song: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PlayerData {
    pub data: Vec<String>,
    pub toc: Vec<TocItem>,
}

// TODO: have a look at this
impl Add for PlayerData {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.data.len() == 0 {
            return other;
        }
        let last_self_data = self.data[self.data.len() - 1].clone();
        Self {
            toc: self
                .toc
                .into_iter()
                .chain(other.toc.iter().map(|item| {
                    let item = TocItem {
                        idx: if self.data.len() > 0
                            && other.data.len() > 0
                            && self.data[self.data.len() - 1] == other.data[0]
                        {
                            item.idx + self.data.len() - 1
                        } else {
                            item.idx + self.data.len()
                        },
                        title: item.title.clone(),
                        nr: item.nr.clone(),
                        song: item.song.clone(),
                    };
                    item
                }))
                .collect::<Vec<TocItem>>(),
            data: self
                .data
                .into_iter()
                .chain(
                    other
                        .data
                        .into_iter()
                        .skip_while(|data| *data == last_self_data),
                )
                .collect::<Vec<String>>(),
        }
    }
}
