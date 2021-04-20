#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

extern crate pancurses;

pub mod line;
pub mod method;
pub mod song;
