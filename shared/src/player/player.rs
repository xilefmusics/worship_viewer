use super::{Orientation, PlayerItem, ScrollType, TocItem};
use crate::song::LinkOwned as SongLinkOwned;

use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::sync::OnceLock;
#[cfg(feature = "backend")]
use utoipa::ToSchema;

fn empty_item() -> &'static PlayerItem {
    static EMPTY_ITEM: OnceLock<PlayerItem> = OnceLock::new();
    EMPTY_ITEM.get_or_init(PlayerItem::default)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub struct Player {
    items: Vec<PlayerItem>,
    toc: Vec<TocItem>,
    scroll_type: ScrollType,
    scroll_type_cache_other_orientation: ScrollType,
    orientation: Orientation,
    between_items: bool,
    index: usize,
}

impl Player {
    pub fn new(items: Vec<PlayerItem>, toc: Vec<TocItem>) -> Self {
        Self {
            items,
            toc,
            scroll_type: ScrollType::default(),
            scroll_type_cache_other_orientation: ScrollType::Book,
            orientation: Orientation::Portrait,
            between_items: bool::default(),
            index: usize::default(),
        }
    }

    pub fn toc(&self) -> &[TocItem] {
        &self.toc
    }

    pub fn song_id(&self) -> Option<String> {
        self.toc
            .iter()
            .filter(|item| item.idx <= self.index())
            .last()
            .map(|item| item.id.clone())
            .flatten()
    }

    pub fn set_like_mut(&mut self, id: &str, liked: bool) {
        if let Some(idx) = self
            .toc
            .iter()
            .position(|item| item.id.as_ref() == Some(&id.to_string()))
        {
            self.toc[idx].liked = liked;
        }
    }

    pub fn set_like(&self, id: &str, liked: bool) -> Self {
        let mut new = self.clone();
        new.set_like_mut(id, liked);
        new
    }

    pub fn like_multi(&self, ids: &[String]) -> Self {
        let mut new = self.clone();
        for id in ids {
            new.set_like_mut(id, true);
        }
        new
    }

    pub fn set_scroll_type(&self, scroll_type: ScrollType) -> Self {
        let mut new = self.clone();

        new.scroll_type = scroll_type;
        new.between_items = false;

        if let ScrollType::Book = new.scroll_type {
            if new.index() % 2 == 0 {
                new.decrement();
            }
        }
        new
    }

    pub fn next_scroll_type(&self) -> Self {
        self.set_scroll_type(self.scroll_type.next())
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

    pub fn is_empty(&self) -> bool {
        self.items.len() == 0
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation.clone()
    }

    pub fn item(&self) -> (&PlayerItem, Option<&PlayerItem>) {
        if self.items.len() == 0 {
            return (empty_item(), None);
        }
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
        self.items.len().saturating_sub(1)
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

    pub fn update_orientation(&self, orientation: Orientation) -> Self {
        if self.orientation == orientation {
            return self.clone();
        }

        let mut new = self.clone();

        let new_scroll_type = new.scroll_type_cache_other_orientation;
        new.scroll_type_cache_other_orientation = new.scroll_type.clone();
        new = new.set_scroll_type(new_scroll_type);
        new.orientation = orientation;

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
                        id: item.id.clone(),
                        nr: item.nr.clone(),
                        liked: item.liked,
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
            scroll_type_cache_other_orientation: self.scroll_type_cache_other_orientation,
            orientation: self.orientation,
            between_items: self.between_items,
            index: self.index,
        }
    }
}

impl From<SongLinkOwned> for Player {
    fn from(link: SongLinkOwned) -> Self {
        Self {
            items: {
                let mut items = link
                    .song
                    .blobs
                    .iter()
                    .map(|blob| PlayerItem::Blob(blob.to_string()))
                    .collect::<Vec<PlayerItem>>();
                if link.song.data.sections.len() > 0 || items.len() == 0 {
                    items.push(PlayerItem::Chords(link.song.clone()))
                }
                items
            },
            toc: if link.song.not_a_song {
                vec![]
            } else {
                vec![TocItem {
                    idx: 0,
                    title: link.song.data.title,
                    id: Some(link.song.id.clone()),
                    nr: link.nr.clone().unwrap_or_default(),
                    liked: false,
                }]
            },
            scroll_type: ScrollType::default(),
            scroll_type_cache_other_orientation: ScrollType::Book,
            orientation: Orientation::Portrait,
            between_items: bool::default(),
            index: usize::default(),
        }
    }
}
