use std::env;

use worship_viewer::method;

fn main() {
    let mut args = env::args();
    args.next();
    match args.next() {
        Some(method) => match method.as_str() {
            "show" => method::show(args).unwrap_or_else(|err| {
                eprintln!("Error: {}", err);
            }),
            method => eprintln!("Error: No method {}", method),
        }
        None => eprintln!("Error: No method given"),
    }
}
