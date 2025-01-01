use std::{
    net::IpAddr,
    str::FromStr,
    error::Error,
    sync::{
        mpsc, Arc,
        Mutex,
    },
    thread::{
        JoinHandle,
        self,
    },
    io::{ Cursor, Read },
    //ops::Drop,
    time::Duration,
    //collections::HashMap,
};
use tiny_http::{
    Server,
    Request,
    Response,
    Header,
    StatusCode,
    ReadWrite,
    Method::*,
};
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use base64::prelude::*;
use local_ip_address::local_ip;
use base16;
//use getch_rs::{ Getch, Key::* };
type Job = Box<dyn FnOnce() + Send + 'static>;
type ServerError = Box<dyn Error + Send + Sync + 'static>;
type Answer = Response<Cursor<Vec<u8>>>;
type Socket = Box<dyn ReadWrite + Send>;

use Opcode::*;

fn main() {
    let ip_port_res = args();
    let (ip, port) = match ip_port_res {
        Err(_) => panic!("USAGE:\n\tcargo run [IP Address] <Port>\n"),
        Ok((arg1, arg2)) => (arg1, arg2) 
    };
    println!("Press CONTROL-C to exit");
    let server_tp = ThreadPool::new(10, Some(10)).unwrap();
    let ws_tp = ThreadPool::new(0, None).unwrap();
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
    //let (tx_rq, rx_rq) = mpsc::channel::<Request>();
    let pool1 = Arc::new(pool);
    let pool2 = pool1.clone();
    let (tx_rp, rx_rp) = mpsc::channel::<(Request, Answer, bool)>();
    thread::scope(move |s| {
        s.spawn(move || {
            println!("Started server: {}:{}", &ip, &port);
            for request in server.incoming_requests() {
                println!("I: {} {}", request.method(), request.url());
                //let rx_th = rx_rq.clone();
                let tx_th = tx_rp.clone();
                let job = Box::new(move || process_request(request, tx_th));
                pool1.execute(job);
                /*let tx_job = &pool.sender;
                tx_job.send(job);*/
            }
        });
        s.spawn(move || {
            for (request, response, is_ws) in rx_rp.into_iter() {
                if !is_ws {
                    request.respond(response);
                    continue;
                }                    
                let mut stream = request.upgrade("websocket", response);
                let job = Box::new(move || handle_ws(Stream::new(stream)));
                pool2.execute(job);
            } 
        });
    });
    Ok(())
}
struct ThreadPool {
    workers: Vec<Worker>,
    limit: Option<usize>,
    sender: mpsc::Sender<Job>
}
struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>,
}
impl Worker {
    fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let handle = thread::spawn(move || loop {
            let res = rx.lock().unwrap().recv();
            match res {
                Ok(job) => {println!("Worker {} executing job", id); job(); println!("Worker {} done", id);}
                Err(_) => {println!("Worker {} shutting down", id); break;}
            }
        });
        Worker { 
            id: id,
            handle: Some(handle),
        }
    }
}
impl ThreadPool {
    fn new(amount: usize, limit: Option<usize>) -> Result<ThreadPool, String> {
        let mut workers: Vec<Worker> = Vec::with_capacity(amount);
        let (tx, rx) = mpsc::channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));
        for id in 0..amount {
            let worker = Worker::new(id, rx.clone());
            workers.push(worker);
        }     
        Ok(ThreadPool {
            workers: workers,
            limit: limit,
            sender: tx
        })
    }
    fn execute(&self, job: Job) -> Result<(), ()> {
        let res = &self.sender.send(job);
        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(())
        }
    }
    fn spawn(&self, job: Job) {
        let id = &self.workers.len();
        match &self.limit {
            Some(n) if id <= n => {
                let handle = thread::spawn(|| job());
                let worker = Worker {id: *id, handle: Some(handle)};
            } 
            None => {
                let handle = thread::spawn(|| job());
                let worker = Worker {id: *id, handle: Some(handle)};
            }
            _ => println!("Pool max size reached"),
        }
    }
}
/*impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(&self.sender);
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}*/
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
fn check_ws(request: &Request) -> bool {
    let res = request
        .headers()
        .iter()
        .find(|&h| h.field.equiv("Connection"))
        .map(|h| h.value.as_str());
    match res {
        Some(v) if v.to_ascii_lowercase().contains("upgrade") => true,
        _ => false
    }
}
fn encode(key: String) -> String {
    let mut str = key;
    str.push_str("258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let mut sha1 = Sha1::new();
    sha1.input_str(&str);
    let hash = sha1.result_str();
    let mut buf: &mut [u8] = &mut [0u8; 20];
    base16::decode_slice(hash.as_bytes(), &mut buf);
    let res = BASE64_STANDARD.encode(buf);
    res
}
fn get_upgrade_response(request: &Request) -> Answer {
    let key = request
        .headers()
        .iter()
        .find(|&h| h.field.equiv("Sec-WebSocket-Key"))
        .map(|h| h.value.as_str())
        .unwrap();
    let accept = encode(key.to_string());
    let headers: Vec<Header> = vec![
        Header::from_bytes(&*b"Sec-WebSocket-Accept", accept.as_bytes()).unwrap(),
        Header::from_bytes(&*b"Connection", &*b"Upgrade").unwrap(),
        Header::from_bytes(&*b"Upgrade", &*b"websocket").unwrap(),
    ];
    let mut response = Response::from_string(accept);
    for header in headers {
        response.add_header(header);
    }
    response = response.with_status_code(StatusCode(101));
    response
}
fn write_ws() -> Result<(), u8> {
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
        let control_byte: u8 = stream.read(1).expect("Could not get stream control content")[0];
        println!("{:?}", control_byte);
        let c_bits1 = into_bits(control_byte);
        let fin = c_bits1[0];
        let opcode = Opcode::from(from_bits(&c_bits1[3..8]));
        let content = stream.get_content(); 
        match opcode {
            Fragment => println!("fragment"),
            Text | Binary => println!("data"),
            Close => println!("close"),
            Ping => println!("ping"),
            Pong => println!("pong"),
            Unknown(n) => println!("Unknown: {n}")
        }
    }
}
enum Opcode {
    Fragment,
    Text,
    Binary,
    Close,
    Ping,
    Pong,
    Unknown(u8),
}
impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Fragment,
            1 => Self::Text,
            2 => Self::Binary,
            8 => Self::Close,
            9 => Self::Ping,
            10 => Self::Pong,
            n => Self::Unknown(n)
        }
    }
}
struct Stream { 
    inner: Socket
}
impl Stream {
    fn new(stream: Socket) -> Self {
        Stream {inner: stream}
    }
    fn inner(&mut self) -> &mut Socket {
        &mut self.inner
    }
    fn get_content(&mut self) -> Vec<u8> {
        let control_byte: u8 = self.read(1).expect("Could not get stream control content")[0];
        let c_bits2 = into_bits(control_byte);
        let is_masked = c_bits2[0]; 
        let len = from_bits(&c_bits2[1..8]);
        let real_len: u32;
        let len_len: u8 = match len {
            0..126 => 0,
            126 => 2,
            127 => 4,
            _ => unreachable!("len > 127")
        };
        if len_len == 0 {
            real_len = len as u32; 
        } else {
            let len_bytes: Vec<u8> = self.read(len_len as u32).expect("Could not get stream control content");
            let mut buf: u32 = 0;
            for i in 0..len_len {
                let shift = (len_len - i - 1) * 8;
                buf += (len_bytes[i as usize] as u32) << shift as u32;
            }
            real_len = buf;
        }
        let mask: Option<Vec<u8>> = match is_masked {
            false => None,
            true => {
                let res = self.read(4).expect("Could not get stream control content");
                Some(res)
            },
        };
        let mut content = self.read(real_len).expect("Could not get stream control content");
        if let Some(key) = mask { 
            for i in 0..content.len() {
                content[i] = content[i] ^ key[i % 4];
            }
        }
        println!("Content: {:?}", content);
        content
    }
    fn read(&mut self, bytes: u32) -> Result<Vec<u8>, String> {
        if bytes == 0 {
            return Ok(Vec::<u8>::new());
        }
        let mut buf: Vec<u8> = Vec::with_capacity(bytes as usize);
        let read = self.inner()
            .take(bytes as u64)
            .read_to_end(&mut buf);
        match read {
            Ok(0) => Err("Could not read anything".to_string()),
            Ok(n) => Ok(buf),
            Err(err) => Err(err.to_string())
        }
    }
}

fn into_bits(byte: u8) -> [bool; 8] {
    let mut bits: [bool; 8] = [false; 8];
    for i in 0..8 {
        let check: u8 = 0b10000000 >> i;
        let bit: u8 = byte & check;
        bits[i] = match bit {
            0 => false,
            _ => true
        };
    }
    bits
}
fn from_bits(bits: &[bool]) -> u8 {
    let mut byte: u8 = 0;
    for i in 0..8 {
        if let Some(true) = bits.get(i) {
            byte |= 0b10000000 >> 8 - bits.len() + i;
        }
    }
    byte
}
/*fn from_bytes<T: From<u8>>(bytes: Vec<u8>) /* -> Result<T, ()>*/ {
    let mut int: T = 0.into();
        
}*/
