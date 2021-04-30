use pancurses::{Input, Window};

use super::Error;

pub struct ConfirmationBox {
    window: Window,
}

impl ConfirmationBox {
    pub fn new(
        nlines: i32,
        ncols: i32,
        begy: i32,
        begx: i32,
        parent: &Window,
    ) -> Result<Self, Error> {
        let window = parent.subwin(nlines, ncols, begy, begx)?;
        Ok(Self { window })
    }

    pub fn confirm_on(&self, text: &str) -> bool {
        self.window.clear();
        self.window.draw_box(0, 0);
        self.window.refresh();
        self.window.mvprintw(
            1,
            2,
            text.chars()
                .take((self.window.get_max_x() - 6) as usize)
                .chain(" (y/n)".chars())
                .collect::<String>(),
        );
        loop {
            match self.window.getch() {
                Some(Input::Character('y')) | Some(Input::Character('Y')) => return true,
                _ => return false,
            }
        }
    }
}
