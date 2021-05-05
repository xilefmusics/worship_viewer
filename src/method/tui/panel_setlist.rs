use ::pancurses::{Input, Window};

use std::rc::Rc;

use crate::setlist::{Setlist, SetlistItem, SetlistItemFmtWithKeyWrapper, SetlistPool};
use crate::song::SongPool;
use crate::tui::{ConfirmationBox, List};

use super::super::Error;

enum Mode {
    Setlist,
    SetlistSong,
    AllSongs,
}

pub struct PanelSetlist {
    window: Window,
    list_setlist: List<String>,
    list_setlist_songs: List<SetlistItemFmtWithKeyWrapper>,
    list_all_songs: List<String>,
    setlist_pool: Rc<SetlistPool>,
    song_pool: Rc<SongPool>,
    mode: Mode,
    current_setlist_title: Option<String>,
    confirmation_box: ConfirmationBox,
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
        let width = ncols / 3;
        let mode = Mode::Setlist;

        let list_setlist = List::new(nlines, width, 0, 0, &window, setlist_pool.true_titles())?;
        list_setlist.set_selected(true);

        let list_setlist_songs = List::new(
            nlines,
            width,
            0,
            width,
            &window,
            setlist_pool
                .get_first()
                .map(|setlist| {
                    setlist
                        .items()
                        .into_iter()
                        .map(|setlist_item| SetlistItemFmtWithKeyWrapper { setlist_item })
                        .collect::<Vec<SetlistItemFmtWithKeyWrapper>>()
                })
                .unwrap_or(vec![]),
        )?;

        let list_all_songs = List::new(
            nlines,
            width,
            0,
            ncols - width,
            &window,
            song_pool.titles()?,
        )?;

        let current_setlist_title = setlist_pool.get_first().map(|setlist| setlist.title);

        let confirmation_box =
            ConfirmationBox::new(3, window.get_max_x(), window.get_max_y() - 3, 0, &window)?;

        Ok(Self {
            window,
            list_setlist,
            list_setlist_songs,
            list_all_songs,
            setlist_pool,
            song_pool,
            mode,
            current_setlist_title,
            confirmation_box,
        })
    }

    pub fn render(&self) {
        self.list_setlist.render();
        self.list_setlist_songs.render();
        self.list_all_songs.render();
        self.window.refresh();
    }

    fn next_prev_mode(&mut self, prev: bool) {
        if prev {
            match self.mode {
                Mode::Setlist => {
                    self.list_setlist.set_selected(false);
                    self.list_all_songs.set_selected(true);
                    self.mode = Mode::AllSongs;
                }
                Mode::SetlistSong => {
                    self.list_setlist_songs.set_selected(false);
                    self.list_setlist.set_selected(true);
                    self.mode = Mode::Setlist;
                }
                Mode::AllSongs => {
                    self.list_all_songs.set_selected(false);
                    self.list_setlist_songs.set_selected(true);
                    self.mode = Mode::SetlistSong;
                }
            }
        } else {
            match self.mode {
                Mode::Setlist => {
                    self.list_setlist.set_selected(false);
                    self.list_setlist_songs.set_selected(true);
                    self.mode = Mode::SetlistSong;
                }
                Mode::SetlistSong => {
                    self.list_setlist_songs.set_selected(false);
                    self.list_all_songs.set_selected(true);
                    self.mode = Mode::AllSongs;
                }
                Mode::AllSongs => {
                    self.list_all_songs.set_selected(false);
                    self.list_setlist.set_selected(true);
                    self.mode = Mode::Setlist;
                }
            }
        }
        self.render();
    }

    fn add_title_to_setlist(&mut self, title: String) {
        if let Ok(Some(song)) = self.song_pool.get(&SetlistItem {
            title: title.clone(),
            key: "Self".to_string(),
        }) {
            let key = song.key.clone();
            let setlist_item = SetlistItem { title, key };
            let wrapper = SetlistItemFmtWithKeyWrapper { setlist_item };
            self.list_setlist_songs.insert(wrapper);
        }
    }

    fn set_key(&mut self, key: &str) {
        if let Some(mut item) = self.list_setlist_songs.selected_item() {
            item.setlist_item.key = if key == "b" || key == "#" {
                if item.setlist_item.key.as_str() == "Self" {
                    return;
                }
                item.setlist_item
                    .key
                    .chars()
                    .take(1)
                    .chain(key.chars())
                    .collect::<String>()
            } else {
                key.to_string()
            };
            self.list_setlist_songs.change_selected_item(item)
        }
    }

    fn write_current_setlist(&self) -> Result<(), Error> {
        if let Some(title) = self.current_setlist_title.clone() {
            if self
                .confirmation_box
                .confirm_on(&format!("Do you want to write the setlist \"{}\"", title))
            {
                let items = self
                    .list_setlist_songs
                    .items()
                    .into_iter()
                    .map(|item| item.setlist_item)
                    .collect::<Vec<SetlistItem>>();
                let setlist = Setlist::new(title, items);
                self.setlist_pool.update_setlist(setlist)?;
            }
        }
        self.render();
        Ok(())
    }

    fn select_setlist(&mut self, title: String) -> Result<(), Error> {
        self.current_setlist_title = Some(title.clone());
        Ok(if let Some(setlist) = self.setlist_pool.get(title)? {
            self.list_setlist_songs.change_items(
                setlist
                    .items()
                    .into_iter()
                    .map(|setlist_item| SetlistItemFmtWithKeyWrapper { setlist_item })
                    .collect::<Vec<SetlistItemFmtWithKeyWrapper>>(),
            );
        } else {
            self.list_setlist_songs.change_items(vec![])
        })
    }

    pub fn handle_input_mode_setlist(&mut self, input: Option<Input>) -> Result<(), Error> {
        match input {
            Some(Input::Character('n')) => {
                if let Some(title) = self.list_setlist.new_item()? {
                    self.list_setlist.sort();
                    self.select_setlist(title)?
                }
            }
            Some(Input::Character(' ')) => {
                if let Some(title) = self.list_setlist.selected_item() {
                    self.select_setlist(title)?;
                }
            }
            input => self.list_setlist.handle_input(input)?,
        }
        Ok(())
    }

    pub fn handle_input_mode_setlist_song(&mut self, input: Option<Input>) -> Result<(), Error> {
        match input {
            Some(Input::Character(' ')) => self.list_setlist_songs.remove(),
            Some(Input::Character('K')) => self.list_setlist_songs.move_up(),
            Some(Input::Character('J')) => self.list_setlist_songs.move_down(),
            Some(Input::Character('A')) => self.set_key("A"),
            Some(Input::Character('B')) => self.set_key("B"),
            Some(Input::Character('C')) => self.set_key("C"),
            Some(Input::Character('D')) => self.set_key("D"),
            Some(Input::Character('E')) => self.set_key("E"),
            Some(Input::Character('F')) => self.set_key("F"),
            Some(Input::Character('G')) => self.set_key("G"),
            Some(Input::Character('b')) => self.set_key("b"),
            Some(Input::Character('#')) => self.set_key("#"),
            Some(Input::Character('r')) => self.set_key("Self"),
            input => self.list_setlist_songs.handle_input(input)?,
        }
        Ok(())
    }

    pub fn handle_input_mode_all_songs(&mut self, input: Option<Input>) -> Result<(), Error> {
        match input {
            Some(Input::Character(' ')) => {
                if let Some(title) = self.list_all_songs.selected_item() {
                    self.add_title_to_setlist(title);
                }
            }
            input => self.list_all_songs.handle_input(input)?,
        }
        Ok(())
    }

    pub fn handle_input(&mut self, input: Option<Input>) -> Result<(), Error> {
        match input {
            Some(Input::Character('w')) => self.write_current_setlist()?,
            Some(Input::Character('h')) => self.next_prev_mode(true),
            Some(Input::Character('l')) | Some(Input::Character('\t')) => {
                self.next_prev_mode(false)
            }
            input => match self.mode {
                Mode::Setlist => self.handle_input_mode_setlist(input),
                Mode::SetlistSong => self.handle_input_mode_setlist_song(input),
                Mode::AllSongs => self.handle_input_mode_all_songs(input),
            }?,
        }
        Ok(())
    }
}
