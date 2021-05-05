#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate ws;

extern crate pancurses;

pub mod method;
pub mod setlist;
pub mod song;
pub mod tui;
