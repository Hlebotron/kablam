use std::{
    net::IpAddr,
    str::FromStr,
    error::Error,
    thread,
    time::Duration,
    sync::{
        mpsc,
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
use server::Server::{
    *,
    Opcode::*,
};

pub mod setup;

pub mod game;
use game::game::*;

fn main() {
    let ip_port_res = args();
    let (ip, port) = match ip_port_res {
        Err(_) => panic!("USAGE:\n\tcargo run [IP Address] <Port>\n"),
        Ok((arg1, arg2)) => (arg1, arg2) 
    };
    println!("Press CONTROL-C to exit");
    let server_tp = ThreadPool::new(10, Some(10)).unwrap();
    let ws_tp = ThreadPool::new(0, None).unwrap();
    let (tx_game, rx_server) = mpsc::channel::<GameAction>();
    let (tx_server, rx_game) = mpsc::channel::<PlayerAction>();
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
fn args() -> Result<(IpAddr, u16), Box<dyn Error>> {
    //TODO: Make all the panics in this function (return only (IpAddr, u16))
    let args: Vec<String> = std::env::args().collect();
    let ip: IpAddr;
    let port: u16;
    match args.len() {
        2 => {
            port = args[1].parse()?; 
            ip = local_ip()?;
        }
        3 => {
            port = args[2].parse()?; 
            ip = IpAddr::from_str(&args[1])?;
        }
        _ => panic!("USAGE:\n\tcargo run [IP Address] <Port>\n")
    }
    Ok((ip, port))
}
