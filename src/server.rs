pub mod Server {
    use std::{
        net::IpAddr,
        str::FromStr,
        error::Error,
        sync::{
            mpsc,
            Arc,
            Mutex,
        },
        thread::{
            JoinHandle,
            self,
        },
        io::{ Cursor, Read, Write },
        //ops::Drop,
        time::Duration,
        //collections::HashMap,
        fmt::{ Display, Formatter },
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
    pub type Job = Box<dyn FnOnce() + Send + 'static>;
    type ServerError = Box<dyn Error + Send + Sync + 'static>;
    pub type Answer = Response<Cursor<Vec<u8>>>;
    type Socket = Box<dyn ReadWrite + Send>;

    use Opcode::*;
    pub struct ThreadPool {
        workers: Vec<Worker>,
        limit: Option<usize>,
        sender: mpsc::Sender<Job>
    }
    struct Worker {
        id: usize,
        handle: Option<JoinHandle<()>>,
    }
    impl Worker {
        pub fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
            let handle = thread::spawn(move || loop {
                let res = rx.lock().unwrap().recv();
                match res {
                    Ok(job) => {
                        println!("Worker {} executing job", id); 
                        job(); 
                        println!("Worker {} done", id);
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", id); 
                        break;
                    }
                }
            });
            Worker { 
                id: id,
                handle: Some(handle),
            }
        }
    }
    impl ThreadPool {
        pub fn new(amount: usize, limit: Option<usize>) -> Result<ThreadPool, String> {
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
        pub fn execute(&self, job: Job) -> Result<(), ()> {
            let res = &self.sender.send(job);
            match res {
                Ok(_) => Ok(()),
                Err(_) => Err(())
            }
        }
        pub fn spawn(&self, job: Job) {
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
    pub fn check_ws(request: &Request) -> bool {
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
    pub fn encode(key: String) -> String {
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
    pub fn get_upgrade_response(request: &Request) -> Answer {
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
    #[derive(Copy, Clone)]
    #[non_exhaustive]
    pub enum Opcode {
        //TODO: Fragment
        Fragment = 0,
        Text = 1,
        Binary = 2,
        Close = 8,
        Ping = 9,
        Pong = 10,
        Unknown = 16,
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
                _ => Self::Unknown
            }
        }
    }
    impl Display for Opcode {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
            let display = match self {
                Self::Fragment => "Fragment",
                Self::Text => "Text",
                Self::Binary => "Binary", 
                Self::Close => "Close", 
                Self::Ping => "Ping", 
                Self::Pong => "Pong", 
                Self::Unknown => "Unknown", 
            };
            write!(f, "{display}")
        }
    }
    pub struct Frame {
        fin: bool,
        rsv1: bool,
        rsv2: bool,
        rsv3: bool,
        opcode: Opcode,
        mask: Option<Vec<u8>>,
        content: Vec<u8>
    }
    impl Frame {
        pub fn new(fin: bool, rsv1: bool, rsv2: bool, rsv3: bool, opcode: Opcode, mask: Option<Vec<u8>>, content: Vec<u8>) -> Frame {
            Frame {
                fin: fin,
                rsv1: rsv1,
                rsv2: rsv2,
                rsv3: rsv3,
                opcode: opcode,
                mask: mask,
                content: content
            }
        }
        pub fn fin(&self) -> bool {
            self.fin
        }
        pub fn rsv1(&self) -> bool {
            self.rsv1
        }
        pub fn rsv2(&self) -> bool {
            self.rsv2
        }
        pub fn rsv3(&self) -> bool {
            self.rsv3
        }
        pub fn opcode(&self) -> Opcode {
            self.opcode
        }
        pub fn mask(&self) -> &Option<Vec<u8>> {
            &self.mask
        }
        pub fn bytes(&self) -> Vec<u8> {
            self.content.to_vec()
        }
        pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
            String::from_utf8(self.bytes())
        }
        pub fn content_len(&self) -> u64 {
            self
                .bytes()
                .len()
                .try_into()
                .expect("Math ain't mathing: III")
        }
    }
    impl Display for Frame {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
            write!(f, "Frame(fin: {}, rsv1: {}, rsv2: {}, rsv3: {}, opcode: {}, mask: {:?}, content: {:?})", self.fin, self.rsv1, self.rsv2, self.rsv3, self.opcode, self.mask, self.content)
        }
    }
    pub struct Stream { 
        inner: Socket,
        timeout: Duration
    }
    impl Stream {
        pub fn new(stream: Socket, timeout: Duration) -> Self {
            Stream {
                inner: stream, 
                timeout: timeout
            }
        }
        pub fn get_frame(&mut self) -> Frame {
            let control_bytes: Vec<u8> = self.read_n(2).expect("Could not get stream control content");
            let control_bits1 = into_bits(control_bytes[0]);
            let fin = control_bits1[0];
            let rsv1 = control_bits1[1];
            let rsv2 = control_bits1[2];
            let rsv3 = control_bits1[3];
            let opcode = Opcode::from(from_bits(&control_bits1[4..8]));
            let control_bits2 = into_bits(control_bytes[1]);
            let is_masked = control_bits2[0]; 
            let len = from_bits(&control_bits2[1..8]);
            let real_len: u64;
            let len_len: u8 = match len {
                0..126 => 0,
                126 => 2,
                127 => 8,
                _ => unreachable!("len > 127")
            };
            if len_len == 0 {
                real_len = len as u64; 
            } else {
                let len_bytes: Vec<u8> = self.read_n(len_len as u64).expect("Could not get stream control content");
                let mut buf: u64 = 0;
                for i in 0..len_len {
                    let shift = (len_len - i - 1) * 8;
                    buf += (len_bytes[i as usize] as u64) << shift as u64;
                }
                real_len = buf;
            }
            let mask: Option<Vec<u8>> = match is_masked {
                false => None,
                true => {
                    let res = self.read_n(4).expect("Could not get stream control content");
                    Some(res)
                },
            };
            let mut content = self.read_n(real_len).expect("Could not get stream control content");
            if let Some(key) = &mask { 
                for i in 0..content.len() {
                    content[i] = content[i] ^ key[i % 4];
                }
            }
            Frame::new(fin, rsv1, rsv2, rsv3, opcode, mask, content)
        }
        pub fn write_frame(&mut self, frame: Frame) {
            let mut byte1: u8 = 0;
            byte1 |= from_bits(&[frame.fin(), frame.rsv1(), frame.rsv2(), frame.rsv3()]) << 4;
            byte1 |= frame.opcode() as u8;
            let mut byte2: u8 = 0;
            let (len_len, len) = set_len(frame.content_len());
            byte2 |= len_len; 
            let mut content = frame.bytes();
            self.write_all(&[byte1, byte2]);
            self.write_all(&len);
            if let Some(key) = frame.mask() { 
                byte2 |= 1 << 7;
                for i in 0..content.len() {
                    content[i] = content[i] ^ key[i % 4];
                }
                self.write_all(&key);
            }
            self.write_all(&content);
            self.flush();
        }
        pub fn close(mut self, mut frame: Frame) {
            frame.mask = None;
            self.write_frame(frame);
        }
        pub fn pong(&mut self, content: &[u8], limit: u64) {
            if content.len() <= limit as usize {
                let mut byte2 = 0x00;
                let (len_len, len) = set_len(content.len().try_into().expect("Math ain't mathing again"));
                byte2 |= len_len;
                self.write_all(&[0x8A, byte2]);
                self.write_all(len.as_slice());
                self.write_all(content);
                self.flush();
            }
        }
        pub fn ping(&mut self, content: &[u8]) {
            let (len_len, len) = set_len(content.len().try_into().expect("Math ain't mathing again"));
            let byte1: u8 = 0b10001001; //fin: true, opcode: 9
            let mut byte2: u8 = 0b00000000;
            byte2 |= len_len;
            self.write_all(&[byte1, byte2]);
            self.write_all(len.as_slice());
            self.write_all(content);
            self.flush();
            println!("Sent Ping: {:?}", content);
            let frame = self.get_frame();
            if let Pong = frame.opcode() {
                println!("Received Pong: {:?}", frame.bytes());
            } else {
                println!("Received other frame: {}", frame);
            }
        }
        pub fn inner(&mut self) -> &mut Socket {
            &mut self.inner
        }
        pub fn into_socket(self) -> Socket {
            self.inner
        }
        pub fn read_n(&mut self, bytes: u64) -> Result<Vec<u8>, String> {
            if bytes == 0 {
                return Ok(Vec::<u8>::new());
            }
            let mut buf: Vec<u8> = Vec::with_capacity(bytes as usize);
            let read = self
                .take(bytes as u64)
                .read_to_end(&mut buf);
            match read {
                Ok(0) => Err("Could not read anything".to_string()),
                Ok(n) => Ok(buf),
                Err(err) => Err(err.to_string())
            }
        }
    }
    impl Write for Stream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner().write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.inner().flush()
        }
    }
    impl Read for Stream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner().read(buf)
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
    fn set_len(size: u64) -> (u8, Vec<u8>) {
        let (len_len, len): (u8, Vec<u8>) = match size {
            0..126 => (size.try_into().expect("Math ain't mathing"), Vec::new()),
            126..=65535 => (126, size.to_be_bytes().to_vec()),
            _ => (127, size.to_be_bytes().to_vec())
        };
        (len_len, len)
    }
}
