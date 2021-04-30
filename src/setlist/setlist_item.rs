use std::cmp;
use std::fmt;

#[derive(Debug, Clone, Eq)]
pub struct SetlistItem {
    pub title: String,
    pub key: String,
}

impl fmt::Display for SetlistItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl cmp::Ord for SetlistItem {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.title.cmp(&other.title)
    }
}

impl cmp::PartialOrd for SetlistItem {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl cmp::PartialEq for SetlistItem {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}

#[derive(Debug, Clone, Eq)]
pub struct SetlistItemFmtWithKeyWrapper {
    pub setlist_item: SetlistItem,
}

impl fmt::Display for SetlistItemFmtWithKeyWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.setlist_item.title, self.setlist_item.key)
    }
}

impl cmp::Ord for SetlistItemFmtWithKeyWrapper {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.setlist_item.cmp(&other.setlist_item)
    }
}

impl cmp::PartialOrd for SetlistItemFmtWithKeyWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl cmp::PartialEq for SetlistItemFmtWithKeyWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.setlist_item == other.setlist_item
    }
}
