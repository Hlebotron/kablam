use std::{
    net::{ TcpListener, TcpStream, IpAddr, ToSocketAddrs },
    io::{ Read, Write, BufReader, self, BufRead },
    time::Duration,
    thread,
    default::Default,
    fmt::{ Display, Formatter },
    str::FromStr,
    convert::{ Infallible, From },
    rc::Rc,
};
use crate::{i, e, w}; 
use HeaderKey::*;
use HeaderContent::*;
/*trait TryRead {
    fn try_read(&mut self, buf: &mut [u8]) -> Result<Option<usize>>;
    fn read_timeout(&mut self, buf: &mut [u8], duration: Duration) -> Result<Option<usize>>;
}
trait TryWrite {
    fn try_write(&mut self, buf: &mut [u8]) -> Result<Option<usize>>;
    fn write_timeout(&mut self, buf: &mut [u8], duration: Duration) -> Result<Option<usize>>;
}*/
const ADDRESS: &'static str = "192.168.1.133:6969";
trait AsBytes {
    fn as_bytes(&self) -> Vec<u8>;
    //fn as_bytes_iter<T: Iterator>() -> T {}
}
impl Display for Header<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.0, self.1)
    }
}
impl Display for HeaderKey {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let print = match self {
            Host => "Host",
            Connection => "Connection",
            AIM => "A-IM",
            Accept => "Accept",
            AcceptCharset => "Accept-Charset",
            AcceptEncoding => "Accept-Encoding",
            AcceptLanguage => "Accept-Language",
            AccessControlRequestMethod => "Access-Control-Request-Method",
            AccessControlRequestHeaders => "Access-Control-Request-Headers",
            Authorization => "Authorization",
            CacheControl => "Cache-Control",
            ContentEncoding => "Content-Encoding",
            ContentLength => "Content-Length",
            ContentMD5 => "Content-MD5",
            ContentType => "Content-Type",
            Cookie => "Cookie",
            Date => "Date",
            Expect => "Expect",
            Forwarded => "Forwarded",
            From => "From",
            HTTP2Settings => "HTTP2-Settings",
            IfMatch => "If-Match",
            IfModifiedSince => "If-Modified-Since",
            IfNoneMatch => "If-None-Match",
            IfUnmodifiedSince => "If-Unmodified-Since",
            MaxForwards => "Max-Forwards",
            Origin => "Origin",
            Pragma => "Pragma",
            Prefer => "Prefer",
            ProxyAuthorization => "Proxy-Authorization",
            Range => "Range",
            Referer => "Referer",
            TE => "TE",
            Trailer => "Trailer",
            TransferEncoding => "Transfer-Encoding",
            UserAgent => "User-Agent",
            Upgrade => "Upgrade",
            Via => "Via",
            Warning => "Warning",
            Other(header) => header,
        };
        write!(f, "{}", print)
    }
}
impl TryFrom<String> for Header<'_> {
    type Error = ();
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let split: Vec<&str> = value.split(": ").collect();
        let key = HeaderKey::from_str(split[0])?;
        Ok(Header(key, Text(split[1].to_string())))
    }
}
impl FromStr for HeaderKey {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s {
            "Host" => Host,
            "Connection" => Connection,
            "A-IM" => AIM,
            "Accept" => Accept,
            "Accept-Charset" => AcceptCharset,
            "Accept-Encoding" => AcceptEncoding,
            "Accept-Language" => AcceptLanguage,
            "Access-Control-Request-Method" => AccessControlRequestMethod,
            "Access-Control-Request-Headers" => AccessControlRequestHeaders,
            "Authorization" => Authorization,
            "Cache-Control" => CacheControl,
            "Content-Encoding" => ContentEncoding,
            "Content-Length" => ContentLength,
            "Content-MD5" => ContentMD5,
            "Content-Type" => ContentType,
            "Cookie" => Cookie,
            "Date" => Date,
            "Expect" => Expect,
            "Forwarded" => Forwarded,
            "From" => From,
            "HTTP2-Settings" => HTTP2Settings,
            "If-Match" => IfMatch,
            "If-Modified-Since" => IfModifiedSince,
            "If-None-Match" => IfNoneMatch,
            "If-Unmodified-Since" => IfUnmodifiedSince,
            "Max-Forwards" => MaxForwards,
            "Origin" => Origin,
            "Pragma" => Pragma,
            "Prefer" => Prefer,
            "Proxy-Authorization" => ProxyAuthorization,
            "Range" => Range,
            "Referer" => Referer,
            "TE" => TE,
            "Trailer" => Trailer,
            "Transfer-Encoding" => TransferEncoding,
            "User-Agent" => UserAgent,
            "Upgrade" => Upgrade,
            "Via" => Via,
            "Warning" => Warning,
            header => Other(header.to_string())
        };
        Ok(res)
    }
}

pub struct Server(TcpListener);
pub struct WebSocket(TcpStream);
pub struct HttpStream(TcpStream);
impl HttpStream {
    pub fn respond(&mut self, response: Response) -> std::io::Result<usize> {
        let num = self.0.write(&response.as_bytes())?;
        self.0.flush(); 
        Ok(num)
    }
    pub fn get_requests(&mut self) -> std::io::Result<Vec<Request>> {
        let mut lines = BufReader::new(&mut *self).lines();
        let line1 = lines.next().expect("No line")?;
        let mut method_path_version = line1.split(" ");
        let method: Method = method_path_version
            .next()
            .unwrap()
            .into();
        i!("{}", method);
        /*if len == 0 {
            return Ok(Vec::new());
        }*/
        Ok(Vec::new())
    }
    pub fn try_get_requests(&self, timeout: Duration) -> std::io::Result<Vec<Request>> {
        todo!()
    }
}
impl Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}
pub struct Request<'a> {
    method: Method,
    path: Rc<str>,
    version: Version,
    headers: Vec<Header<'a>>,
    content: &'a [u8]
}
impl<'a> Request<'a> {
    pub fn method(&self) -> Method {
        self.method.clone()
    }
    pub fn path(&self) -> Rc<str> {
        self.path.clone()
    }
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn headers(&self) -> &Vec<Header<'a>> {
        &self.headers
    }
    pub fn headers_mut(&mut self) -> &mut Vec<Header<'a>> {
        &mut self.headers
    }
    pub fn content(&self) -> &'a [u8] {
        self.content
    }
    pub fn set_content(&mut self, content: &'a [u8]) {
        let len = content.len();
        self.content = content; 
        for header in self.headers.iter_mut() {
            if let ContentLength = header.0 {
                header.1 = Number(len);
            } 
        }
    }
}
pub struct Response<'a> {
    version: Version,
    code: HttpCode,
    headers: Vec<Header<'a>>,
    content: &'a [u8]
}
impl<'a> Response<'a> {
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn http_code(&self) -> HttpCode {
        self.code
    }
    pub fn headers(&mut self) -> &mut Vec<Header<'a>> {
        &mut self.headers
    }
    pub fn content(&self) -> &'a [u8] {
        self.content
    }
    pub fn set_content(&mut self, content: &'a [u8]) {
        let len = content.len();
        self.content = content; 
        for header in self.headers.iter_mut() {
            if let ContentLength = header.0 {
                header.1 = Number(len);
            } 
        }
    }
    pub fn add_header(&mut self, header: Header<'a>) {
        self.headers.push(header);
    }
    pub fn rm_header(&mut self, index: usize) {
        self.headers.swap_remove(index);
    }
}
impl<'a> From<Request<'a>> for Response<'a> {
    fn from(value: Request<'a>) -> Self {
        Response {
            version: value.version,
            code: HttpCode::OK,
            headers: value.headers,
            content: value.content
        }
    }
}
#[derive(Clone)]
pub struct Header<'a>(HeaderKey, HeaderContent<'a>);
#[derive(Clone, Copy)]
pub struct Version(u8, u8);
#[derive(Clone)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Connect,
    Options,
    Trace,
    Patch,
    Other(String)
}
impl From<&str> for Method {
    fn from(value: &str) -> Method {
        use Method::*;
        match value {
            "GET" => Get,
            "POST" => Post,
            "PUT" => Put,
            "DELETE" => Delete,
            "HEAD" => Head,
            "CONNECT" => Connect,
            "OPTIONS" => Options,
            "TRACE" => Trace,
            "PATCH" => Patch,
            other => Other(other.to_string())
        }
    } 
}
impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use Method::*;
        let write = match self {
            Get => "GET",
            Post => "POST",
            Put => "PUT",
            Delete => "DELETE",
            Head => "HEAD",
            Connect => "CONNECT",
            Options => "OPTIONS",
            Trace => "TRACE",
            Patch => "PATCH",
            Other(other) => &other
        };
        write!(f, "{}", write)
    }
}
#[repr(u16)]
#[derive(Clone, Copy)]
pub enum HttpCode {
    Continue = 100,
    SwitchingProtocols,
    Processing,
    EarlyHints,
    OK = 200,
    Created,
    Accepted,
    NonAuthorativeInformation,
    NoContent,
    ResetContent,
    PartialContent,
    MultiStatus,
    AlreadyReported,
    IMUsed = 226,
    MultipleChoices = 300,
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    UseProxy,
    SwitchProxy,
    TemporaryRedirect,
    PermanentRedirect,
    BadRequest = 400,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeOut,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed = 412,
    PayloadTooLarge,
    URITooLong,
    UnsupportedMediaType,
    RangeNotSatisfiable,
    ExpectationFailed,
    MisdirectedRequest = 421,
    UnprocessableRequest,
    Locked,
    FailedDependency,
    TooEarly,
    UpgradeRequired,
    PreconditionRequired,
    TooManyRequests,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,
    InternalServerError = 500,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HttpVersionNotSupported,
    VariantAlsoNegotiates,
    InsufficientStorage,
    LoopDetected,
    NotExtended = 510,
    NetworkAuthenticationRequired,
}
impl TryFrom<u16> for HttpCode {
    type Error = ();
    fn try_from(value: u16) -> Result<HttpCode, ()> {
        use HttpCode::*;
        match value {
            100 => Ok(Continue),
            101 => Ok(SwitchingProtocols),
            102 => Ok(Processing),
            103 => Ok(EarlyHints),
            200 => Ok(OK),
            201 => Ok(Created),
            202 => Ok(Accepted),
            203 => Ok(NonAuthorativeInformation),
            204 => Ok(NoContent),
            205 => Ok(ResetContent),
            206 => Ok(PartialContent),
            207 => Ok(MultiStatus),
            208 => Ok(AlreadyReported),
            226 => Ok(IMUsed),
            300 => Ok(MultipleChoices),
            301 => Ok(MovedPermanently),
            302 => Ok(Found),
            303 => Ok(SeeOther),
            304 => Ok(NotModified),
            305 => Ok(UseProxy),
            306 => Ok(SwitchProxy),
            307 => Ok(TemporaryRedirect),
            308 => Ok(PermanentRedirect),
            400 => Ok(BadRequest),
            401 => Ok(Unauthorized),
            402 => Ok(PaymentRequired),
            403 => Ok(Forbidden),
            404 => Ok(NotFound),
            405 => Ok(MethodNotAllowed),
            406 => Ok(NotAcceptable),
            407 => Ok(ProxyAuthenticationRequired),
            408 => Ok(RequestTimeOut),
            409 => Ok(Conflict),
            410 => Ok(Gone),
            411 => Ok(LengthRequired),
            412 => Ok(PreconditionFailed),
            413 => Ok(PayloadTooLarge),
            414 => Ok(URITooLong),
            415 => Ok(UnsupportedMediaType),
            416 => Ok(RangeNotSatisfiable),
            417 => Ok(ExpectationFailed),
            421 => Ok(MisdirectedRequest),
            422 => Ok(UnprocessableRequest),
            423 => Ok(Locked),
            424 => Ok(FailedDependency),
            425 => Ok(TooEarly),
            426 => Ok(UpgradeRequired),
            428 => Ok(PreconditionRequired),
            429 => Ok(TooManyRequests),
            431 => Ok(RequestHeaderFieldsTooLarge),
            451 => Ok(UnavailableForLegalReasons),
            500 => Ok(InternalServerError),
            501 => Ok(NotImplemented),
            502 => Ok(BadGateway),
            503 => Ok(ServiceUnavailable),
            504 => Ok(GatewayTimeout),
            505 => Ok(HttpVersionNotSupported),
            506 => Ok(VariantAlsoNegotiates),
            507 => Ok(InsufficientStorage),
            508 => Ok(LoopDetected),
            510 => Ok(NotExtended),
            511 => Ok(NetworkAuthenticationRequired),
            _ => Err(())
        }
    }
}
impl From<&HttpCode> for String {
    fn from(value: &HttpCode) -> String {
        use HttpCode::*;
        match value {
            Continue => "100 Continue",
            SwitchingProtocols => "101 Switching Protocols",
            Processing => "102 Processing",
            EarlyHints => "103 Early Hints",
            OK => "200 OK",
            Created => "201 Created",
            Accepted => "202 Accepted",
            NonAuthorativeInformation => "203 Non-Authorative Information",
            NoContent => "204 No Content",
            ResetContent => "205 Reset Content",
            PartialContent => "206 Partial Content",
            MultiStatus => "207 Multi-Status",
            AlreadyReported => "208 Already Reported",
            IMUsed => "226 IM Used",
            MultipleChoices => "300 Multiple Choices",
            MovedPermanently => "301 Moved Permanently",
            Found => "302 Found",
            SeeOther => "303 See Other",
            NotModified => "304 Not Modified",
            UseProxy => "305 Use Proxy",
            SwitchProxy => "306 Switch Proxy",
            TemporaryRedirect => "307 Temporary Redirect",
            PermanentRedirect => "308 Permanent Redirect",
            BadRequest => "400 Bad Request",
            Unauthorized => "401 Unauthorized",
            PaymentRequired => "402 Payment Required",
            Forbidden => "403 Forbidden",
            NotFound => "404 Not Found",
            MethodNotAllowed => "405 Method Not Allowed",
            NotAcceptable => "406 Method Not Acceptable",
            ProxyAuthenticationRequired => "407 Proxy Authentication Required",
            RequestTimeOut => "408 Request Timeout",
            Conflict => "409 Conflict",
            Gone => "410 Gone",
            LengthRequired => "411 Length Required",
            PreconditionFailed => "412 Precondition Failed",
            PayloadTooLarge => "413 Payload Too Large",
            URITooLong => "414 URI Too Long",
            UnsupportedMediaType => "415 Unsupported Media Type",
            RangeNotSatisfiable => "416 Range Not Satisfiable",
            ExpectationFailed => "417 Expectation Failed",
            MisdirectedRequest => "421 Misdirected Request",
            UnprocessableRequest => "422 Unprocessable Request",
            Locked => "423 Locked",
            FailedDependency => "424 Failed Dependency",
            TooEarly => "425 Too Early",
            UpgradeRequired => "426 Upgrade Required",
            PreconditionRequired => "428 Precondition Required",
            TooManyRequests => "429 Too Many Requests",
            RequestHeaderFieldsTooLarge => "431 Request Header Fields Too Large",
            UnavailableForLegalReasons => "451 Unavailable For Legal Reasons",
            InternalServerError => "500 Internal Server Error",
            NotImplemented => "501 Not Implemented",
            BadGateway => "502 Bad Gateway",
            ServiceUnavailable => "503 Service Unavailable",
            GatewayTimeout => "504 Gateway Timeout",
            HttpVersionNotSupported => "505 HTTP Version Not Supported",
            VariantAlsoNegotiates => "506 Variant Also Negotiates",
            InsufficientStorage => "507 Insufficient Storage",
            LoopDetected => "508 Loop Detected",
            NotExtended => "510 Not Extended",
            NetworkAuthenticationRequired => "511 Network Authentication Required",
        }.to_string()
    }
}
impl<'a> From<&'a str> for Response<'a> {
    fn from(value: &str) -> Response {
        let mut packet = Response::default();
        packet.content = value.as_bytes();
        packet
    }
}
impl<'a> From<&'a [u8]> for Response<'a> {
    fn from(value: &[u8]) -> Response {
        let mut packet = Response::default();
        packet.content = value;
        packet
    }
}
//pub struct Response<'a> {
//    version: Version,
//    code: HttpCode,
//    headers: Vec<Header<'a>>,
//    content: &'a [u8]
//}
impl AsBytes for Response<'_> {
    fn as_bytes(&self) -> Vec<u8> {
        let version = self.version.as_bytes();
        let code = self.code.as_bytes();
        let mut headers = Vec::<u8>::new();
        for header in &self.headers {
            headers.extend(header.as_bytes().into_iter());
            headers.extend(b"\r\n");
        }
        let content = self.content;
        let len: usize = &version.len() + &code.len() + &headers.len() + &content.len();
        let mut vec: Vec<u8> = Vec::with_capacity(len);
        vec.extend(version);
        vec.extend(code);
        vec.extend(b"\r\n");
        vec.extend(headers);
        vec.extend(b"\r\n");
        vec.extend(content);
        vec
    }
}
impl AsBytes for Version {
    fn as_bytes(&self) -> Vec<u8> {
        format!("HTTP/{}.{} ", self.0, self.1).as_bytes().to_vec()
    }
}
impl AsBytes for HttpCode {
    fn as_bytes(&self) -> Vec<u8> {
        String::from(self).as_bytes().to_vec()
    }
}
#[derive(Clone)]
pub enum HeaderContent<'a> {
    Bytes(&'a [u8]),
    Text(String),
    Boolean(bool),
    Number(usize),
    List(&'a [HeaderContent<'a>])
}
impl Display for HeaderContent<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let write = match self {
            Bytes(inner) => {
                let mut string: String = Default::default();
                for character in inner.into_iter() {
                    string.push(char::from(*character)); 
                }
                string
            },
            Text(inner) => inner.to_owned(),
            Boolean(inner) => match inner {
                true => "true".to_string(),
                false => "false".to_string() 
            },
            Number(inner) => inner.to_string(),
            List(inner) => {
                let mut string: String = Default::default();
                for element in inner.iter() {
                    string.push_str(&format!("{}, ", &element.to_string()));
                } 
                string
            },
        };
        write!(f, "{}",  write)
    }
}
/*impl FromStr for HeaderContent<'_> {
    type Err = ();
    fn from_str(s: &str) -> Result<HeaderContent, Self::Err> {
        
    }
}*/
impl<'a> AsBytes for HeaderContent<'a> {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Bytes(inner) => inner.to_vec(),
            Text(inner) => inner.as_bytes().to_vec(),
            Boolean(inner) => match inner {
                true => "true".as_bytes(),
                false => "false".as_bytes()
            }.to_vec(),
            Number(inner) => inner.to_be_bytes().to_vec(),
            List(inner) => {
                let mut bytes: Vec<u8> = Default::default();
                for element in inner.into_iter() {
                    bytes.extend_from_slice(element.as_bytes().as_slice());
                }
                bytes
            }
        }
    }
}
impl AsBytes for Header<'_> {
    fn as_bytes(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}
impl Default for Response<'_> {
    fn default() -> Self {
        let headers: Vec<Header> = vec![
            Header(ContentType, Text("text/plain".to_string())),
            Header(ContentLength, Number(3)),
        ];
        Response {
            version: Version(1, 1),
            code: HttpCode::OK,
            headers: headers,
            content: b"pog" 
        }
    } 
}
#[derive(Clone)]
pub enum HeaderKey {
    Host,
    Connection,
    AIM,
    Accept,
    AcceptCharset,
    AcceptEncoding,
    AcceptLanguage,
    AccessControlRequestMethod,
    AccessControlRequestHeaders,
    Authorization,
    CacheControl,
    ContentEncoding,
    ContentLength,
    ContentMD5,
    ContentType,
    Cookie,
    Date,
    Expect,
    Forwarded,
    From,
    HTTP2Settings,
    IfMatch,
    IfModifiedSince,
    IfNoneMatch,
    IfUnmodifiedSince,
    MaxForwards,
    Origin,
    Pragma,
    Prefer,
    ProxyAuthorization,
    Range,
    Referer,
    TE,
    Trailer,
    TransferEncoding,
    UserAgent,
    Upgrade,
    Via,
    Warning,
    Other(String)
}
impl Server {
    pub fn start_fn<T: ToSocketAddrs, F: Fn(TcpStream) + Send + Copy + 'static>(address: T, fun: F) -> std::io::Result<()> {
        let tcp_stream = TcpListener::bind(address)?; 
        for res in tcp_stream.incoming() {
            if let Ok(mut stream) = res {
                thread::spawn(move || {
                    fun(stream);
                }); 
            }
        }
        Ok(())
    }
    pub fn start<T: ToSocketAddrs>(address: T) -> std::io::Result<()> {
        Server::start_fn(address, handle_http)
    }
    //pub fn inner(&self) -> &TcpListener {
    //    &self.0
    //}
    //pub fn inner_mut(&mut self) -> &mut TcpListener {
    //    &mut self.0
    //}
}
fn handle_http(tcp_stream: TcpStream) {
    println!("Received Request");
    let mut stream = HttpStream(tcp_stream);
    let packet = Response::from("pog");
    let _ = stream.respond(packet);            
}
#[cfg(test)]
mod tests {
    use super::*;
    /*#[test]
    fn start_server() {
        Server::start("192.168.1.133:6970").expect("Could not bind to address"); 
    }*/
    #[test]
    fn get_request() {
        let mut stream = create_stream(); 
        stream.get_requests().unwrap();
        panic!("INTENDED")
    }
    fn create_stream() -> HttpStream {
        let server = TcpListener::bind(ADDRESS).unwrap(); 
        let stream = server.accept().ok().unzip().0.unwrap();
        HttpStream(stream)
    }
}
