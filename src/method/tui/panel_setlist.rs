use ::pancurses::{Input, Window};

use std::cell::Cell;
use std::rc::Rc;

use crate::setlist::SetlistPool;
use crate::song::SongPool;
use crate::tui::List;

use super::super::Error;

pub struct PanelSetlist {
    window: Window,
    list_setlist: List<String>,
    list_setlist_songs: List<String>,
    list_all_songs: List<String>,
    song_pool: Rc<SongPool>,
    setlist_pool: Rc<SetlistPool>,
    current_list: Cell<isize>,
}

impl PanelSetlist {
    pub fn new(
        nlines: i32,
        ncols: i32,
        begy: i32,
        begx: i32,
        parent: &Window,
        song_pool: Rc<SongPool>,
        setlist_pool: Rc<SetlistPool>,
    ) -> Result<Self, Error> {
        let window = parent.subwin(nlines, ncols, begy, begx)?;
        let current_list = Cell::new(0);
        let width = ncols / 3;

        let list_setlist = List::new(nlines, width, 0, 0, &window, setlist_pool.titles())?;
        list_setlist.set_selected(true);
        let list_setlist_songs = List::new(
            nlines,
            width,
            0,
            width,
            &window,
            setlist_pool
                .get_first()
                .map(|setlist| setlist.titles())
                .unwrap_or(vec![]),
        )?;
        let list_all_songs =
            List::new(nlines, width, 0, ncols - width, &window, song_pool.titles())?;

        Ok(Self {
            window,
            list_setlist,
            list_setlist_songs,
            list_all_songs,
            song_pool,
            setlist_pool,
            current_list,
        })
    }

    pub fn render(&self) {
        self.list_setlist.render();
        self.list_setlist_songs.render();
        self.list_all_songs.render();
        self.window.refresh();
    }

    fn set_selected(&self) {
        self.list_setlist.set_selected(self.current_list.get() == 0);
        self.list_setlist_songs
            .set_selected(self.current_list.get() == 1);
        self.list_all_songs
            .set_selected(self.current_list.get() == 2);
        self.render();
    }

    pub fn handle_input(&mut self, input: Option<Input>) -> Result<(), Error> {
        match input {
            Some(Input::Character('h')) => {
                self.current_list
                    .set((self.current_list.get() - 1).rem_euclid(3));
                self.set_selected();
            }
            Some(Input::Character('l')) => {
                self.current_list
                    .set((self.current_list.get() + 1).rem_euclid(3));
                self.set_selected();
            }
            Some(Input::Character(' ')) => {
                if self.current_list.get() == 2 {
                    if let Some(title) = self.list_all_songs.selected_item() {
                        self.list_setlist_songs.insert(title);
                    }
                } else if self.current_list.get() == 1 {
                    self.list_setlist_songs.remove();
                } else if self.current_list.get() == 0 {
                    if let Some(title) = self.list_setlist.selected_item() {
                        if let Some(setlist) = self.setlist_pool.get(title) {
                            self.list_setlist_songs.change_items(setlist.titles());
                        }
                    }
                }
            }
            Some(Input::Character('K')) => {
                if self.current_list.get() == 1 {
                    self.list_setlist_songs.move_up();
                }
            }
            Some(Input::Character('J')) => {
                if self.current_list.get() == 1 {
                    self.list_setlist_songs.move_down();
                }
            }
            Some(Input::Character('n')) => {
                if self.current_list.get() == 0 {
                    self.list_setlist_songs.new_item()?;
                }
            }
            Some(Input::Character('\t')) => {
                self.current_list
                    .set((self.current_list.get() + 1).rem_euclid(3));
                self.set_selected();
            }
            input => {
                if self.current_list.get() == 0 {
                    self.list_setlist.handle_input(input)
                } else if self.current_list.get() == 1 {
                    self.list_setlist_songs.handle_input(input)
                } else if self.current_list.get() == 2 {
                    self.list_all_songs.handle_input(input)
                } else {
                    Ok(())
                }?
            }
        }
        Ok(())
    }
}
