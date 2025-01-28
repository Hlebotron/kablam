#![feature(mpmc_channel)]
use std::{
    net::IpAddr,
    str::FromStr,
    error::Error,
    thread,
    time::Duration,
    sync::{
        mpmc,
        Arc,
        Mutex,
    },
    collections::HashMap,
};
use local_ip_address::local_ip;
//use getch_rs::{ Getch, Key::* };
type Job = Box<dyn FnOnce() + Send + 'static>;
type ServerError = Box<dyn Error + Send + Sync + 'static>;
use tiny_http::{Request, Server, Response, Method::*};

pub mod server;
#[doc(inline)]
use server::{
    *,
    Opcode::*,
};

pub mod setup;

pub mod game;
use game::*;

fn main() {
    let (ip, port) = args();
    println!(r#"Press CONTROL-C to exit
Legend:
I - Info
E - Error
"#);
    let server_tp = ThreadPool::new(10, Some(10)).unwrap();
    let ws_tp = ThreadPool::new(0, None).unwrap();
    let (tx_game, rx_server) = mpmc::channel::<GameAction>();
    let (tx_server, rx_game) = mpmc::channel::<PlayerAction>();
    thread::scope(|s| {
        s.spawn(move || {
            run_server(ip, port, server_tp, ws_tp, tx_server, rx_server);
        });
        s.spawn(move || {
            start_game(tx_game, rx_game);
        });
        /*s.spawn(move || {
            let getch = Getch::new();
            loop {
                let key = getch.getch();
                if let Ok(Ctrl('c')) = key {
                    println!("Gracefully stopping");
                    let pool = server_tp_clone.lock().unwrap();
                    drop(pool);
                    break;
                }
            }
        });*/
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
