use std::env;

use worship_viewer::method;

fn main() {
    let mut args = env::args();
    args.next();
    match args.next() {
        Some(method) => match method.as_str() {
            "show" => method::show(args).unwrap_or_else(|err| eprintln!("Error: {}", err)),
            "import" => method::import(args).unwrap_or_else(|err| eprintln!("Error: {}", err)),
            "tui" => method::tui(args).unwrap_or_else(|err| eprintln!("Error: {}", err)),
            "server" => method::server(args).unwrap_or_else(|err| eprintln!("Error: {}", err)),
            "ws_console" => {
                method::ws_console(args).unwrap_or_else(|err| eprintln!("Error: {}", err))
            }
            method => eprintln!("Error: No such method ({})", method),
        },
        None => eprintln!("Error: No method given"),
    }
}
