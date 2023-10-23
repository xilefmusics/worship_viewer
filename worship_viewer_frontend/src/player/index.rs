use worship_viewer_shared::types::TocItem;

#[derive(Debug, Clone, Default)]
pub struct Index {
    page_index: usize,
    max_page_index: usize,
    between_pages: bool,
    scroll_type: ScrollType,
    select_type: SelectType,
    max_number_index: usize,
    page2number: Vec<usize>,
    number2page: Vec<usize>,
    number2string: Vec<String>,
}

impl Index {
    fn increment(&self) -> usize {
        if self.page_index + 1 < self.max_page_index {
            self.page_index + 1
        } else {
            self.page_index
        }
    }
    fn double_increment(&self) -> usize {
        if self.page_index + 2 < self.max_page_index {
            self.page_index + 2
        } else {
            self.page_index
        }
    }
    fn decrement(&self) -> usize {
        if self.page_index > 0 {
            self.page_index - 1
        } else {
            0
        }
    }
    fn inner_jump(&self, new: usize) -> usize {
        if new >= self.max_page_index {
            self.max_page_index - 1
        } else {
            new
        }
    }
    fn double_decrement(&self) -> usize {
        if self.page_index > 1 {
            self.page_index - 2
        } else {
            0
        }
    }
    fn page2number(&self, idx: usize) -> usize {
        self.page2number[idx]
    }
    fn number2page(&self, nr: usize) -> usize {
        self.number2page[nr]
    }
    fn number2string(&self, nr: usize) -> String {
        self.number2string[nr].clone()
    }

    pub fn get_page_index(&self) -> usize {
        self.page_index
    }
    pub fn get_page_string(&self) -> String {
        format!("{}", self.page_index + 1)
    }
    pub fn get_max_page_index(&self) -> usize {
        self.max_page_index
    }
    pub fn get_max_page_string(&self) -> String {
        format!("{}", self.max_page_index + 1)
    }
    pub fn get_number_index(&self) -> usize {
        self.page2number(self.get_page_index())
    }
    pub fn get_number_string(&self) -> String {
        self.number2string(self.get_number_index())
    }
    pub fn get_max_number_index(&self) -> usize {
        self.max_number_index
    }
    pub fn get_max_number_string(&self) -> String {
        self.number2string(self.get_max_number_index())
    }
    pub fn get_index(&self) -> usize {
        match self.select_type {
            SelectType::Page => self.get_page_index(),
            SelectType::Number => self.get_number_index(),
        }
    }
    pub fn get_string(&self) -> String {
        match self.select_type {
            SelectType::Page => self.get_page_string(),
            SelectType::Number => self.get_number_string(),
        }
    }
    pub fn get_max_index(&self) -> usize {
        match self.select_type {
            SelectType::Page => self.get_max_page_index(),
            SelectType::Number => self.get_max_number_index(),
        }
    }
    pub fn get_max_string(&self) -> String {
        match self.select_type {
            SelectType::Page => self.get_max_page_string(),
            SelectType::Number => self.get_max_number_string(),
        }
    }
    pub fn get_scroll_str(&self) -> &'static str {
        self.scroll_type.to_str()
    }
    pub fn get_select_str(&self) -> &'static str {
        self.select_type.to_str()
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

    pub fn next_select_type(&self) -> Self {
        let mut new = self.clone();
        new.select_type = new.select_type.next();
        new
    }
    pub fn next_scroll_type(&self) -> Self {
        let mut new = self.clone();
        new.select_type = new.select_type.next();
        new.scroll_type = new.scroll_type.next();
        if new.scroll_type == ScrollType::Book && new.page_index % 2 == 0 {
            new = new.prev_page();
        }
        new
    }

    pub fn set_max_page_index(&self, max_page_index: usize) -> Self {
        let mut new = self.clone();
        new.max_page_index = max_page_index;
        new
    }
    pub fn create_number_index_mappings(self, toc: &Vec<TocItem>) -> Self {
        let mut new = self.clone();
        new.number2string = toc.iter().map(|item| item.nr.clone()).collect();
        new.number2page = toc.iter().map(|item| item.idx).collect();
        new.page2number = vec![usize::MAX; new.max_page_index];
        for (nr, item) in toc.iter().enumerate() {
            new.page2number[item.idx] = nr;
        }
        let mut last = 0;
        for i in 0..new.page2number.len() {
            if new.page2number[i] == usize::MAX {
                new.page2number[i] = last;
            } else {
                last = new.page2number[i];
            }
        }
        new.max_number_index = new.number2page.len() - 1;
        new
    }

    pub fn next_page(&self) -> Self {
        let mut new = self.clone();
        match new.scroll_type {
            ScrollType::OnePage => new.page_index = new.increment(),
            ScrollType::HalfPage => {
                new.page_index = if new.between_pages {
                    new.increment()
                } else {
                    new.page_index
                }
            }
            ScrollType::TwoPage => new.page_index = new.increment(),
            ScrollType::Book => {
                new.page_index = if new.page_index % 2 == 1 {
                    new.double_increment()
                } else {
                    new.increment()
                }
            }
            ScrollType::TwoHalfPage => new.page_index = new.increment(),
        }
        new
    }
    pub fn prev_page(&self) -> Self {
        let mut new = self.clone();
        match new.scroll_type {
            ScrollType::OnePage => new.page_index = new.decrement(),
            ScrollType::HalfPage => {
                new.page_index = if !new.between_pages {
                    new.decrement()
                } else {
                    new.page_index
                }
            }
            ScrollType::TwoPage => new.page_index = new.decrement(),
            ScrollType::Book => {
                new.page_index = if new.page_index % 2 == 1 {
                    new.double_decrement()
                } else {
                    new.decrement()
                }
            }
            ScrollType::TwoHalfPage => new.page_index = new.decrement(),
        }
        new
    }
    pub fn jump_page(&self, new_page_index: usize) -> Self {
        let mut new = self.clone();
        new.page_index = self.inner_jump(
            if self.scroll_type == ScrollType::Book && new_page_index % 2 == 0 && new_page_index > 0
            {
                new_page_index - 1
            } else {
                new_page_index
            },
        );
        new
    }
    pub fn jump_number(&self, new_number_index: usize) -> Self {
        self.jump_page(self.number2page(new_number_index))
    }
    pub fn jump(&self, new_index: usize) -> Self {
        match self.select_type {
            SelectType::Page => self.jump_page(new_index),
            SelectType::Number => self.jump_number(new_index),
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

#[derive(Debug, Clone, Default, PartialEq)]
pub enum SelectType {
    #[default]
    Page,
    Number,
}

impl SelectType {
    pub fn next(&self) -> Self {
        match self {
            Self::Page => Self::Number,
            Self::Number => Self::Page,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Page => "[page]",
            Self::Number => "[number]",
        }
    }
}
