//! Self-hostable Bang server

#![feature(mpmc_channel)]
#![feature(never_type)]
use std::{
    net::IpAddr,
    str::FromStr,
    error::Error,
    thread,
    sync::mpmc,
};
use local_ip_address::local_ip;
use getch_rs::{ Getch, Key::* };
type Job = Box<dyn FnOnce() + Send + 'static>;
type ServerError = Box<dyn Error + Send + Sync + 'static>;

pub mod server;
use server::*;

pub mod http;
use http::*;

pub mod setup;

pub mod game;
use game::*;

fn main() {
    Server::start("192.168.1.133:6969").expect("jog");
    let (ip, port) = args();
    println!("Press CONTROL-C to exit\nLegend:\n  \x1b[32m|\x1b[0m - Info\n  \x1b[101m \x1b[0m - Error\n  \x1b[103m \x1b[0m - Warning");
    thread::sleep(std::time::Duration::from_secs(1));
    let server_tp = ThreadPool::new(10, Some(10)).unwrap();
    let ws_tp = ThreadPool::new(0, None).unwrap();
    let (tx_game, rx_server) = mpmc::channel::<GameAction>();
    let (tx_server, rx_game) = mpmc::channel::<PlayerAction>();
    thread::scope(|s| {
        s.spawn(move || {
            //In the file "server.rs"
            run_server(ip, port, server_tp, ws_tp, tx_server, rx_server);
        });
        s.spawn(move || {
            //In the file "game.rs"
            start_game(tx_game, rx_game);
        });
        s.spawn(move || {
            let getch = Getch::new();
            loop {
                let key = getch.getch();
                match key {
                    Ok(Ctrl('c')) => {
                        w!("Gracefully stopping"); 
                        break;
                    }
                    Ok(_) => (),
                    Err(err) => e!("E: Getch - {err}")
                }
            }
        });
    });
}
fn args() -> (IpAddr, u16) {
    let args: Vec<String> = std::env::args().collect();
    let ip: Option<IpAddr>;
    let port: Option<u16>;
    let res = match args.len() {
        2 => {
            port = args[1].parse().ok(); 
            ip = local_ip().ok();
            (ip, port)
        }
        3 => {
            port = args[2].parse().ok(); 
            ip = IpAddr::from_str(&args[1]).ok();
            (ip, port)
        }
        _ => panic!("USAGE:\n\tcargo run [IP Address] <Port>\n")
    };
    if let None = &res.0 {
        panic!("Invalid port, please enter a u16 (0 to 65535)"); 
    }
    if let None = &res.1 {
        panic!("Invalid IP address, please following this format: {{u8}}.{{u8}}.{{u8}}.{{u8}}"); 
    }
    (res.0.unwrap(), res.1.unwrap())
}
///Print an error message
///
///It puts a red space before the content.
#[macro_export]
macro_rules! e {
    ($content:literal) => {
        eprintln!("\x1b[101m \x1b[0m {}", $content)
    };
    {$content:literal, $($args:tt)+} => {
        eprintln!("{}", format!("\x1b[101m \x1b[0m {}", format_args!($content, $($args)+)))
    };
}
///Print an info message
///
///It puts a green pipe before the content.
#[macro_export]
macro_rules! i {
    ($content:literal) => {
        println!("\x1b[32m|\x1b[0m {}", $content)
    };
    { $content:literal, $($args:tt)+} => {
        println!("{}", format!("\x1b[32m|\x1b[0m {}", format!($content, $($args)+)))
    };
}
///Print a warning message
///
///It puts a yellow space before the content.
#[macro_export]
macro_rules! w {
    ($content:literal) => {
        eprintln!("\x1b[103m \x1b[0m {}", $content)
    };
    { $content:literal, $($args:tt)+} => {
        eprintln!("{}", format!("\x1b[103m \x1b[0m {}", format!($content, $($args)+)))
    };
}
