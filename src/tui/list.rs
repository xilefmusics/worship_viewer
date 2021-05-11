use pancurses::{Input, Window};
use pancurses::{A_BOLD, A_NORMAL, A_REVERSE};

use std::cell::Cell;
use std::fmt::Display;

use super::Error;
use super::InputBox;

pub struct List<T: Display + Clone + Ord> {
    items: Vec<T>,
    idx: Cell<usize>,
    selected: Cell<bool>,
    window: Window,
}

impl<T: Display + Clone + Ord> List<T> {
    pub fn new(
        nlines: i32,
        ncols: i32,
        begy: i32,
        begx: i32,
        parent: &Window,
        items: Vec<T>,
    ) -> Result<Self, Error> {
        let window = parent.subwin(nlines, ncols, begy, begx)?;
        let idx = Cell::new(0);
        let selected = Cell::new(false);
        Ok(Self {
            items,
            window,
            idx,
            selected,
        })
    }

    pub fn render(&self) {
        self.window.clear();
        if self.selected.get() {
            self.window.color_set(2);
        }
        self.window.draw_box(0, 0);
        if self.selected.get() {
            self.window.color_set(0);
        }

        let idx_first = std::cmp::max(
            std::cmp::min(
                self.idx.get() as i32 - self.window.get_max_y() / 2 - 1,
                self.items.len() as i32 - self.window.get_max_y() + 2,
            ),
            0,
        ) as usize;
        let idx_highlight = self.idx.get() - idx_first as usize;
        self.items
            .iter()
            .skip(idx_first)
            .take((self.window.get_max_y() - 2) as usize)
            .map(|item| {
                format!(" {}", item)
                    .chars()
                    .chain(std::iter::repeat(' '))
                    .take((self.window.get_max_x() - 2) as usize)
                    .collect::<String>()
            })
            .enumerate()
            .for_each(|(idx, item)| {
                if idx == idx_highlight {
                    self.window.attrset(A_REVERSE | A_BOLD);
                }
                self.window.mvprintw((idx + 1) as i32, 1, item);
                if idx == idx_highlight {
                    self.window.attrset(A_NORMAL);
                }
            });

        self.window.refresh();
    }

    pub fn isearch(&self, backwards: bool) -> Result<(), Error> {
        let ibox = InputBox::new(
            3,
            self.window.get_max_x() - 2,
            self.window.get_max_y() - 4,
            1,
            &self.window,
        )?;
        if let Some(input) = ibox.input() {
            if backwards {
                self.bsearch(&input);
            } else {
                self.search(&input);
            }
        } else {
            self.render();
        }
        Ok(())
    }

    fn search(&self, s: &str) {
        self.select(
            self.items
                .iter()
                .enumerate()
                .skip(self.idx.get() + 1)
                .find(|(_, item)| format!("{}", item).contains(s))
                .map(|(idx, _)| idx)
                .unwrap_or(self.idx.get()) as usize,
        )
    }

    fn bsearch(&self, s: &str) {
        self.select(
            self.items
                .iter()
                .enumerate()
                .rev()
                .skip(self.items.len() - self.idx.get())
                .find(|(_, item)| format!("{}", item).contains(s))
                .map(|(idx, _)| idx)
                .unwrap_or(self.idx.get()) as usize,
        )
    }

    pub fn next(&self) {
        if self.items.len() == 0 {
            return;
        }
        self.select(self.idx.get() + 1)
    }

    pub fn prev(&self) {
        if self.items.len() == 0 {
            return;
        }
        self.select((self.idx.get() as i32 - 1).rem_euclid(self.items.len() as i32) as usize);
    }

    pub fn select(&self, idx: usize) {
        if self.items.len() == 0 {
            return;
        }
        self.idx.set(idx.rem_euclid(self.items.len()));
        self.render();
    }

    pub fn selected_item(&self) -> Option<T> {
        if self.items.len() == 0 {
            return None;
        }
        Some(self.items[self.idx.get()].clone())
    }

    pub fn items(&self) -> Vec<T> {
        self.items.clone()
    }

    pub fn sort(&mut self) {
        self.items.sort();
        self.idx.set(0);
        self.render();
    }

    pub fn insert(&mut self, item: T) {
        self.items.push(item);
        self.render();
    }

    pub fn remove(&mut self) {
        if self.items.len() == 0 {
            return;
        }
        self.items.remove(self.idx.get());
        self.render();
        if self.idx.get() == self.items.len() {
            self.prev();
        }
    }

    pub fn move_up(&mut self) {
        let idx = self.idx.get();
        if idx == 0 {
            return;
        }
        self.items.swap(idx, idx - 1);
        self.select(idx - 1);
    }

    pub fn change_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.select(0);
        self.render();
    }

    pub fn change_selected_item(&mut self, item: T) {
        self.items[self.idx.get()] = item;
        self.render();
    }

    pub fn move_down(&mut self) {
        let idx = self.idx.get();
        if idx == self.items.len() - 1 {
            return;
        }
        self.items.swap(idx, idx + 1);
        self.select(idx + 1);
    }

    pub fn handle_input(&self, input: Option<Input>) -> Result<(), Error> {
        match input {
            Some(Input::Character('j')) => self.next(),
            Some(Input::Character('k')) => self.prev(),
            Some(Input::Character('/')) => self.isearch(false)?,
            Some(Input::Character('?')) => self.isearch(true)?,
            _ => (),
        }
        Ok(())
    }

    pub fn set_selected(&self, selected: bool) {
        self.selected.set(selected);
    }
}

impl List<String> {
    pub fn new_item(&mut self) -> Result<Option<String>, Error> {
        let ibox = InputBox::new(
            3,
            self.window.get_max_x() - 2,
            self.window.get_max_y() - 4,
            1,
            &self.window,
        )?;
        if let Some(input) = ibox.input() {
            self.insert(input.clone());
            return Ok(Some(input));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pancurses::{endwin, initscr, Input};

    //#[test]
    fn _basic_list() {
        let window = initscr();
        pancurses::noecho();
        pancurses::curs_set(0);

        let items = (0..100)
            .map(|n| format!("Item {}", n))
            .collect::<Vec<String>>();
        let list = List::new(window.get_max_x(), window.get_max_y(), 0, 0, &window, items).unwrap();
        list.render();

        loop {
            match window.getch() {
                Some(Input::Character('q')) => break,
                Some(Input::Character('j')) => list.next(),
                Some(Input::Character('k')) => list.prev(),
                Some(Input::Character('/')) => list.isearch(false).unwrap(),
                Some(Input::Character('?')) => list.isearch(true).unwrap(),
                _ => (),
            }
        }

        endwin();
    }
}
