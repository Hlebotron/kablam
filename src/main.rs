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
    let mut player = Player::new(Character::ElGringo, Role::Sheriff, None);
    let mut deck = HashMap::from([
        (Card::Bang, 3u8),
        (Card::Miss, 5u8)
    ]);
    player.pull_card(&mut deck);
    run_server(ip, port, server_tp, ws_tp);
    /*thread::scope(|s| {
        s.spawn(move || {*/
        /*});
        s.spawn(move || {
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
        });
    });*/
}
fn args() -> Result<(IpAddr, u16), Box<dyn Error>> {
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
fn run_server(ip: IpAddr, port: u16, pool: ThreadPool, ws_pool: ThreadPool) -> Result<(), ServerError> {
    let address = format!("{}:{}", &ip, &port);
    let server = Server::http(address)?;
    let pool1 = Arc::new(pool);
    let pool2 = pool1.clone();
    let (tx_rp, rx_rp) = mpsc::channel::<(Request, Answer, bool)>();
    thread::scope(move |s| {
        s.spawn(move || {
            println!("Started server: {}:{}", &ip, &port);
            for request in server.incoming_requests() {
                println!("I: {} {}", request.method(), request.url());
                let tx_th = tx_rp.clone();
                let job = Box::new(move || process_request(request, tx_th));
                pool1.execute(job);
            }
        });
        s.spawn(move || {
            for (request, response, is_ws) in rx_rp.into_iter() {
                if !is_ws {
                    request.respond(response);
                    continue;
                }                    
                let mut stream = Stream::new(
                    request.upgrade("websocket", response),
                    Duration::from_secs(5)
                );
                let job = Box::new(move || handle_ws(stream));
                pool2.execute(job);
            } 
        });
    });
    Ok(())
}
fn handle_ws(mut stream: Stream) {
    /*
    thread::scope(|s| {
        let (tx, rx) = mpsc::channel::<String>();
        let stream1 = Arc::new(Mutex::new(stream));
        let stream2 = stream1.clone();
        s.spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                println!("pog");
                let msg = rx.try_recv();
                if let Ok(message) = msg {
                    println!("{}", message);
                }
                let mut stream_locked = stream1.lock().expect("Could not lock onto stream");
                let res = stream_locked.write(&[0b10000001, 3, 97, 98, 99]);
                stream_locked.flush();
                if let Err(_) = res {
                    println!("Disconnected (write)");
                    break;
                }
            }
        });
        s.spawn(move || {
            loop {
            }
        });
    });*/
    loop {
        let frame = stream.get_frame(); 
        let content = frame.bytes();
        match frame.opcode() {
            Fragment => println!("fragment"),
            Text => {
                let text = frame.text().expect("Those bastards lied about UTF-8");
                println!("{}", text);
            }
            Binary => println!("data"),
            Close => {
                println!("Closing");
                stream.close(frame);
                break;
            },
            Ping => {
                println!("Received Ping; responding");
                stream.pong(&content, 1024);
            },
            Pong => println!("Received Pong: {:?}", content),
            Unknown => println!("Unknown")
        }
        let mut buf: Vec<u8> = Vec::new();
        thread::sleep(Duration::from_secs(1));
    }
}
fn process_request(mut request: Request, tx: mpsc::Sender<(Request, Answer, bool)>) {
    let is_ws = check_ws(&request);
    let response = match (request.method(), request.url()) {
        (Get, "/") => Response::from_string("pog"),
        (Get, "/ws") => match is_ws {
            true => get_upgrade_response(&request),
            false => Response::from_string("Invalid"),
        }
        _ => Response::from_string("Invalid"),
    };
    tx.send((request, response, is_ws));
}
