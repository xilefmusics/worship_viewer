use super::{PlayerItem, ScrollType, TocItem};
use crate::song::{Song, SongData};

use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Player {
    items: Vec<PlayerItem>,
    toc: Vec<TocItem>,
    scroll_type: ScrollType,
    between_items: bool,
    index: usize,
}

impl Player {
    pub fn new(items: Vec<PlayerItem>, toc: Vec<TocItem>) -> Self {
        Self {
            items,
            toc,
            scroll_type: ScrollType::default(),
            between_items: bool::default(),
            index: usize::default(),
        }
    }

    pub fn toc(&self) -> &[TocItem] {
        &self.toc
    }

    pub fn next_scroll_type(&self) -> Self {
        let mut new = self.clone();
        new.scroll_type = self.scroll_type.next();
        new.between_items = false;
        new
    }
    pub fn scroll_type(&self) -> &ScrollType {
        &self.scroll_type
    }
    pub fn scroll_type_str(&self) -> &str {
        self.scroll_type.to_str()
    }
    pub fn is_half_page_scroll(&self) -> bool {
        self.scroll_type == ScrollType::HalfPage
    }

    pub fn item(&self) -> (&PlayerItem, Option<&PlayerItem>) {
        let current = match self.scroll_type {
            ScrollType::OnePage | ScrollType::HalfPage | ScrollType::TwoPage | ScrollType::Book => {
                &self.items[self.index]
            }
            ScrollType::TwoHalfPage => {
                if self.index % 2 == 0 && self.index != 0 && self.index < self.max_index() {
                    &self.items[self.index + 1]
                } else {
                    &self.items[self.index]
                }
            }
        };
        let next = match self.scroll_type {
            ScrollType::OnePage | ScrollType::HalfPage => {
                if self.between_items && self.index < self.max_index() {
                    Some(&self.items[self.index + 1])
                } else {
                    None
                }
            }
            ScrollType::TwoPage => {
                if self.index < self.max_index() {
                    Some(&self.items[self.index + 1])
                } else {
                    None
                }
            }
            ScrollType::Book => {
                if self.index < self.max_index() && self.index != 0 {
                    Some(&self.items[self.index + 1])
                } else {
                    None
                }
            }
            ScrollType::TwoHalfPage => {
                if self.index % 2 == 1 && self.index < self.max_index() {
                    Some(&self.items[self.index + 1])
                } else if self.index != 0 && self.index != self.max_index() {
                    Some(&self.items[self.index])
                } else {
                    None
                }
            }
        };
        (current, next)
    }

    pub fn index(&self) -> usize {
        self.index
    }
    pub fn max_index(&self) -> usize {
        self.items.len() - 1
    }

    fn increment(&mut self) {
        if self.index < self.max_index() {
            self.index += 1;
        }
    }
    fn decrement(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }
    fn toggle_between_items(&mut self) {
        self.between_items = !self.between_items;
    }

    pub fn next(&self) -> Self {
        let mut new = self.clone();
        match new.scroll_type {
            ScrollType::OnePage => new.increment(),
            ScrollType::HalfPage => {
                if new.between_items {
                    new.increment();
                }
                new.toggle_between_items();
            }
            ScrollType::TwoPage => {
                new.increment();
                new.increment();
            }
            ScrollType::Book => {
                new.increment();
                if self.index > 0 {
                    new.increment();
                }
            }
            ScrollType::TwoHalfPage => new.increment(),
        }
        new
    }
    pub fn prev(&self) -> Self {
        let mut new = self.clone();
        match new.scroll_type {
            ScrollType::OnePage => new.decrement(),
            ScrollType::HalfPage => {
                if self.index > 0 {
                    new.toggle_between_items();
                    if new.between_items {
                        new.decrement();
                    }
                } else {
                    new.between_items = false;
                }
            }
            ScrollType::TwoPage | ScrollType::Book => {
                new.decrement();
                new.decrement();
            }
            ScrollType::TwoHalfPage => new.decrement(),
        }
        new
    }
    pub fn jump(&self, mut index: usize) -> Self {
        let mut new = self.clone();
        if index > new.max_index() {
            index = new.max_index();
        }
        new.index = index;
        if new.scroll_type == ScrollType::Book && new.index % 2 == 0 {
            new.decrement();
        }
        new
    }
}

impl Add for Player {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.items.len() == 0 {
            return other;
        }
        let last_self_item = self.items[self.items.len() - 1].clone();
        Self {
            toc: self
                .toc
                .into_iter()
                .chain(other.toc.iter().map(|item| {
                    let item = TocItem {
                        idx: if self.items.len() > 0
                            && other.items.len() > 0
                            && self.items[self.items.len() - 1] == other.items[0]
                        {
                            item.idx + self.items.len() - 1
                        } else {
                            item.idx + self.items.len()
                        },
                        title: item.title.clone(),
                        nr: item.nr.clone(),
                    };
                    item
                }))
                .collect::<Vec<TocItem>>(),
            items: self
                .items
                .into_iter()
                .chain(
                    other
                        .items
                        .into_iter()
                        .skip_while(|item| *item == last_self_item),
                )
                .collect(),
            scroll_type: self.scroll_type,
            between_items: self.between_items,
            index: self.index,
        }
    }
}

impl From<Song> for Player {
    fn from(song: Song) -> Self {
        Self {
            items: match &song.data {
                SongData::Blob(data) => data
                    .blobs
                    .iter()
                    .map(|blob| PlayerItem::Blob(blob.to_string()))
                    .collect(),
                SongData::Chord(data) => vec![PlayerItem::Chords(data.clone())],
            },
            toc: if song.not_a_song() {
                vec![]
            } else {
                vec![TocItem {
                    idx: 0,
                    title: song.title().to_string(),
                    nr: match &song.data {
                        SongData::Blob(data) => data.nr.to_string(),
                        SongData::Chord(_) => "".to_string(),
                    },
                }]
            },
            scroll_type: ScrollType::default(),
            between_items: bool::default(),
            index: usize::default(),
        }
    }
}
