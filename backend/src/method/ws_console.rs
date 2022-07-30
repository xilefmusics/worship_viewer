use ws::{connect, CloseCode};

use std::env;
use std::io::{self, BufRead};
use std::thread;

use super::Error;

pub fn ws_console(_args: env::Args) -> Result<(), Error> {
    connect("ws://127.0.0.1:8001", |out| {
        let out_send = out.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let line = line.unwrap();
                if line == "stop" {
                    out_send.close(CloseCode::Normal).unwrap();
                }
                out_send.send(line).unwrap();
            }
        });
        move |msg| {
            println!("{}", msg);
            Ok(())
        }
    })?;
    Ok(())
}
