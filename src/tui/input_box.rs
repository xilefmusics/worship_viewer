use pancurses::{Input, Window};

use super::Error;

pub struct InputBox {
    window: Window,
}

impl InputBox {
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

    pub fn render(&self) {
        self.window.clear();
        self.window.draw_box(0, 0);
        self.window.refresh();
    }

    pub fn input(&self) -> Option<String> {
        let mut content = String::new();
        self.render();
        self.window.mv(1, 2);
        pancurses::curs_set(1);

        loop {
            match self.window.getch() {
                Some(Input::Character('\n')) => {
                    pancurses::curs_set(0);
                    return Some(content);
                }
                Some(Input::Character(c)) => {
                    if (c as u32) == 27 {
                        pancurses::curs_set(0);
                        return None;
                    } else if (c as u32) == 127 {
                        if content.len() == 0 {
                            continue;
                        }
                        content.pop();
                        let (y, x) = self.window.get_cur_yx();
                        self.window.mv(y, x - 1);
                        self.window.addch(' ');
                        self.window.mv(y, x - 1);
                        self.window.refresh();
                        continue;
                    }
                    content.push(c);
                    self.window.addch(c);
                    self.window.refresh();
                }
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pancurses::{endwin, initscr, Input};

    // #[test]
    fn _basic_ibox() {
        let window = initscr();
        pancurses::noecho();
        pancurses::cbreak();
        pancurses::curs_set(0);

        let ibox = InputBox::new(3, window.get_max_x(), 0, 0, &window).unwrap();
        assert_eq!(ibox.input(), Some("Hallo".to_string()));
        assert_eq!(ibox.input(), None);

        loop {
            match window.getch() {
                Some(Input::Character('q')) => break,
                _ => (),
            }
        }

        endwin();
    }
}
