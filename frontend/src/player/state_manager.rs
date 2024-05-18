use worship_viewer_shared::types::{PlayerData, TocItem};

#[derive(Debug, Clone, Default)]
pub struct State {
    max_page_index: usize,
    max_number_index: usize,
    page2number: Vec<usize>,
    number2page: Vec<usize>,
    number2string: Vec<String>,
    toc: Vec<TocItem>,
    page2blob: Vec<String>,
}

impl State {
    pub fn new(player_data: PlayerData) -> Self {
        // TODO: move this logic to the backend
        let page2blob = player_data.data;
        let toc = player_data.toc;
        let max_page_index = page2blob.len();
        let number2string = toc.iter().map(|item| item.nr.clone()).collect();
        let number2page = toc.iter().map(|item| item.idx).collect::<Vec<usize>>();
        let mut page2number = vec![usize::MAX; max_page_index];
        for (nr, item) in toc.iter().enumerate() {
            page2number[item.idx] = nr;
        }
        let mut last = 0;
        for i in 0..page2number.len() {
            if page2number[i] == usize::MAX {
                page2number[i] = last;
            } else {
                last = page2number[i];
            }
        }
        let max_number_index = number2page.len() - 1;
        Self {
            max_page_index,
            max_number_index,
            page2number,
            number2page,
            number2string,
            toc,
            page2blob,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CustomState {
    page_index: usize,
    between_pages: bool,
    scroll_type: ScrollType,
    select_type: SelectType,
}

#[derive(Debug, Clone, Default)]
pub struct StateManager {
    state: Box<State>,
    custom_state: Box<CustomState>,
}

impl StateManager {
    pub fn new(state: State, custom_state: CustomState) -> Self {
        Self {
            state: Box::new(state),
            custom_state: Box::new(custom_state),
        }
    }

    fn increment(&self) -> usize {
        if self.custom_state.page_index + 1 < self.state.max_page_index {
            self.custom_state.page_index + 1
        } else {
            self.custom_state.page_index
        }
    }
    fn double_increment(&self) -> usize {
        if self.custom_state.page_index + 2 < self.state.max_page_index {
            self.custom_state.page_index + 2
        } else {
            self.custom_state.page_index
        }
    }
    fn decrement(&self) -> usize {
        if self.custom_state.page_index > 0 {
            self.custom_state.page_index - 1
        } else {
            0
        }
    }
    fn inner_jump(&self, new: usize) -> usize {
        if new >= self.state.max_page_index {
            self.state.max_page_index - 1
        } else {
            new
        }
    }
    fn double_decrement(&self) -> usize {
        if self.custom_state.page_index > 1 {
            self.custom_state.page_index - 2
        } else {
            0
        }
    }
    fn page2number(&self, idx: usize) -> usize {
        self.state.page2number[idx]
    }
    fn number2page(&self, nr: usize) -> usize {
        self.state.number2page[nr]
    }
    fn number2string(&self, nr: usize) -> String {
        self.state.number2string[nr].clone()
    }

    pub fn get_page_index(&self) -> usize {
        self.custom_state.page_index
    }
    pub fn get_page_string(&self) -> String {
        format!("{}", self.get_page_index() + 1)
    }
    pub fn get_max_page_index(&self) -> usize {
        self.state.max_page_index
    }
    pub fn get_max_page_string(&self) -> String {
        format!("{}", self.get_max_page_index() + 1)
    }
    pub fn get_number_index(&self) -> usize {
        self.page2number(self.get_page_index())
    }
    pub fn get_number_string(&self) -> String {
        self.number2string(self.get_number_index())
    }
    pub fn get_max_number_index(&self) -> usize {
        self.state.max_number_index
    }
    pub fn get_max_number_string(&self) -> String {
        self.number2string(self.get_max_number_index())
    }
    pub fn get_index(&self) -> usize {
        match self.custom_state.select_type {
            SelectType::Page => self.get_page_index(),
            SelectType::Number => self.get_number_index(),
        }
    }
    pub fn get_string(&self) -> String {
        match self.custom_state.select_type {
            SelectType::Page => self.get_page_string(),
            SelectType::Number => self.get_number_string(),
        }
    }
    pub fn get_max_index(&self) -> usize {
        match self.custom_state.select_type {
            SelectType::Page => self.get_max_page_index(),
            SelectType::Number => self.get_max_number_index(),
        }
    }
    pub fn get_max_string(&self) -> String {
        match self.custom_state.select_type {
            SelectType::Page => self.get_max_page_string(),
            SelectType::Number => self.get_max_number_string(),
        }
    }
    pub fn get_scroll_str(&self) -> &'static str {
        self.custom_state.scroll_type.to_str()
    }
    pub fn get_select_str(&self) -> &'static str {
        self.custom_state.select_type.to_str()
    }
    pub fn is_between_pages(&self) -> bool {
        self.custom_state.between_pages
    }
    pub fn is_half_page_scroll(&self) -> bool {
        self.custom_state.scroll_type == ScrollType::HalfPage
    }
    pub fn is_two_page_scroll(&self) -> bool {
        self.custom_state.scroll_type == ScrollType::TwoPage
    }
    pub fn is_book_scroll(&self) -> bool {
        self.custom_state.scroll_type == ScrollType::Book
    }
    pub fn is_two_half_page_scroll(&self) -> bool {
        self.custom_state.scroll_type == ScrollType::TwoHalfPage
    }
    pub fn get_data_index_one(&self) -> usize {
        if self.is_two_half_page_scroll() {
            if self.get_page_index() % 2 == 0
                && self.get_max_page_index() > self.get_page_index()
                && self.get_page_index() > 0
            {
                self.get_page_index() + 1
            } else {
                self.get_page_index()
            }
        } else {
            self.get_page_index()
        }
    }
    pub fn get_data_index_two(&self) -> Option<usize> {
        if (self.is_half_page_scroll() && self.is_between_pages()
            || self.is_two_page_scroll()
            || self.is_book_scroll() && self.get_page_index() != 0)
            && self.get_max_page_index() > self.get_page_index()
        {
            Some(self.get_page_index() + 1)
        } else if self.is_two_half_page_scroll() {
            if self.get_page_index() == 0 {
                None
            } else if self.get_page_index() % 2 == 1
                && self.get_max_page_index() > self.get_page_index()
            {
                Some(self.get_page_index() + 1)
            } else {
                Some(self.get_page_index())
            }
        } else {
            None
        }
    }
    pub fn get_blob(&self) -> String {
        self.state.page2blob[self.get_data_index_one()].clone()
    }
    pub fn get_next_blob(&self) -> Option<String> {
        self.get_data_index_two()
            .map(|idx| self.state.page2blob[idx].clone())
    }
    pub fn get_toc(&self) -> Vec<TocItem> {
        self.state.toc.clone()
    }
    pub fn get_toc_len(&self) -> usize {
        self.state.toc.len()
    }

    pub fn next_select_type(&self) -> Self {
        let mut new = self.clone();
        new.custom_state.select_type = new.custom_state.select_type.next();
        new
    }
    pub fn next_scroll_type(&self) -> Self {
        let mut new = self.clone();
        new.custom_state.scroll_type = new.custom_state.scroll_type.next();
        if new.custom_state.scroll_type == ScrollType::Book && new.custom_state.page_index % 2 == 0
        {
            new = new.prev_page();
        }
        new
    }

    pub fn next_page(&self) -> Self {
        let mut new = self.clone();
        match new.custom_state.scroll_type {
            ScrollType::OnePage => new.custom_state.page_index = new.increment(),
            ScrollType::HalfPage => {
                new.custom_state.page_index = if new.custom_state.between_pages {
                    new.increment()
                } else {
                    new.custom_state.page_index
                };
                if new.custom_state.page_index < new.state.max_page_index - 1
                    || new.custom_state.between_pages
                {
                    new.custom_state.between_pages = !new.custom_state.between_pages;
                }
            }
            ScrollType::TwoPage => new.custom_state.page_index = new.increment(),
            ScrollType::Book => {
                new.custom_state.page_index = if new.custom_state.page_index % 2 == 1 {
                    new.double_increment()
                } else {
                    new.increment()
                }
            }
            ScrollType::TwoHalfPage => new.custom_state.page_index = new.increment(),
        }
        new
    }
    pub fn prev_page(&self) -> Self {
        let mut new = self.clone();
        let last_page_index = new.custom_state.page_index;
        match new.custom_state.scroll_type {
            ScrollType::OnePage => new.custom_state.page_index = new.decrement(),
            ScrollType::HalfPage => {
                new.custom_state.page_index = if !new.custom_state.between_pages {
                    new.decrement()
                } else {
                    new.custom_state.page_index
                };
                if last_page_index > 0 || new.custom_state.between_pages {
                    new.custom_state.between_pages = !new.custom_state.between_pages;
                }
            }
            ScrollType::TwoPage => new.custom_state.page_index = new.decrement(),
            ScrollType::Book => {
                new.custom_state.page_index = if new.custom_state.page_index % 2 == 1 {
                    new.double_decrement()
                } else {
                    new.decrement()
                }
            }
            ScrollType::TwoHalfPage => new.custom_state.page_index = new.decrement(),
        }
        new
    }
    pub fn jump_page(&self, new_page_index: usize) -> Self {
        let mut new = self.clone();
        new.custom_state.page_index = self.inner_jump(
            if self.custom_state.scroll_type == ScrollType::Book
                && new_page_index % 2 == 0
                && new_page_index > 0
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
        match self.custom_state.select_type {
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
            Self::Page => "[pg]",
            Self::Number => "[nr]",
        }
    }
}
