#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate pancurses;
extern crate reqwest;
extern crate rocket_contrib;
extern crate serde;
extern crate ws;

pub mod method;
pub mod setlist;
pub mod song;
pub mod tui;
