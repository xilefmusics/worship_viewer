#[derive(Debug, Clone, Default)]
pub struct Index {
    index: usize,
    between_pages: bool,
    max_index: usize,
    scroll_type: ScrollType,
}

impl Index {
    pub fn get(&self) -> usize {
        return self.index;
    }

    pub fn scroll_str(&self) -> &'static str {
        self.scroll_type.to_str()
    }

    pub fn is_between_pages(&self) -> bool {
        self.between_pages
    }

    pub fn is_half_page_scroll(&self) -> bool {
        self.scroll_type == ScrollType::HalfPage
    }

    pub fn is_two_page_scroll(&self) -> bool {
        self.scroll_type == ScrollType::TwoPage
    }

    pub fn is_book_scroll(&self) -> bool {
        self.scroll_type == ScrollType::Book
    }

    pub fn is_two_half_page_scroll(&self) -> bool {
        self.scroll_type == ScrollType::TwoHalfPage
    }

    pub fn next_scroll_type(&self) -> Self {
        let mut next = Self {
            index: self.index,
            between_pages: false,
            max_index: self.max_index,
            scroll_type: self.scroll_type.next(),
        };
        if next.scroll_type == ScrollType::Book && next.index % 2 == 0 {
            next = next.prev();
        }
        next
    }

    pub fn set_max_index(&self, max_index: usize) -> Self {
        Self {
            index: 0,
            between_pages: false,
            max_index,
            scroll_type: self.scroll_type.clone(),
        }
    }

    fn increment(&self) -> usize {
        if self.index + 1 < self.max_index {
            self.index + 1
        } else {
            self.index
        }
    }

    fn double_increment(&self) -> usize {
        if self.index + 2 < self.max_index {
            self.index + 2
        } else {
            self.index
        }
    }

    fn decrement(&self) -> usize {
        if self.index > 0 {
            self.index - 1
        } else {
            0
        }
    }

    fn inner_jump(&self, new: usize) -> usize {
        if new >= self.max_index {
            self.max_index - 1
        } else {
            new
        }
    }

    fn double_decrement(&self) -> usize {
        if self.index > 1 {
            self.index - 2
        } else {
            0
        }
    }

    pub fn next(&self) -> Self {
        match self.scroll_type {
            ScrollType::OnePage => Self {
                index: self.increment(),
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::HalfPage => Self {
                index: if self.between_pages {
                    self.increment()
                } else {
                    self.index
                },
                between_pages: !self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::TwoPage => Self {
                index: self.increment(),
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::Book => Self {
                index: if self.index % 2 == 1 {
                    self.double_increment()
                } else {
                    self.increment()
                },
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::TwoHalfPage => Self {
                index: self.increment(),
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
        }
    }

    pub fn prev(&self) -> Self {
        match self.scroll_type {
            ScrollType::OnePage => Self {
                index: self.decrement(),
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::HalfPage => Self {
                index: if !self.between_pages {
                    self.decrement()
                } else {
                    self.index
                },
                between_pages: !self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::TwoPage => Self {
                index: self.decrement(),
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::Book => Self {
                index: if self.index % 2 == 1 {
                    self.double_decrement()
                } else {
                    self.decrement()
                },
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
            ScrollType::TwoHalfPage => Self {
                index: self.decrement(),
                between_pages: self.between_pages,
                max_index: self.max_index,
                scroll_type: self.scroll_type.clone(),
            },
        }
    }

    pub fn jump(&self, new: usize) -> Self {
        Self {
            index: self.inner_jump(
                if self.scroll_type == ScrollType::Book && new % 2 == 0 && new > 0 {
                    new - 1
                } else {
                    new
                },
            ),
            between_pages: self.between_pages, // TODO reset between_pages
            max_index: self.max_index,
            scroll_type: self.scroll_type.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ScrollType {
    #[default]
    OnePage,
    HalfPage,
    TwoPage,
    Book,
    TwoHalfPage,
}

impl ScrollType {
    pub fn next(&self) -> Self {
        match self {
            Self::OnePage => Self::HalfPage,
            Self::HalfPage => Self::TwoPage,
            Self::TwoPage => Self::Book,
            Self::Book => Self::TwoHalfPage,
            Self::TwoHalfPage => Self::OnePage,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::OnePage => "[1]",
            Self::HalfPage => "[1/2]",
            Self::TwoPage => "[2]",
            Self::Book => "[b]",
            Self::TwoHalfPage => "[2/2]",
        }
    }
}
