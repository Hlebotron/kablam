use std::{
    net::{ TcpListener, TcpStream, IpAddr, ToSocketAddrs },
    io::{ Read, Write, },
    time::Duration,
    thread,
    default::Default,
    fmt::{ Display, Formatter },
    str::FromStr,
    convert::{ Infallible, From },
    rc::Rc,
};
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
//trait ToBytes {
//    fn to_bytes(&self) -> Vec<u8>;
//}
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
pub struct Request<'a> {
    method: Method,
    url: Rc<str>,
    version: Version,
    headers: Vec<Header<'a>>,
    content: &'a [u8]
}
pub struct Response<'a> {
    version: Version,
    code: HttpCode,
    headers: Vec<Header<'a>>,
    content: &'a [u8]
}
#[derive(Clone, Copy)]
pub struct Version(u8, u8);
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
    IMUsed,
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
    PreconditionFailed,
    PayloadTooLarge,
    URITooLong,
    UnsupportedMediaType,
    RangeNotSatisfiable,
    ExpectationFailed,
    MisdirectedRequest,
    UnprocessableRequest,
    Locked,
    FailedDependency,
    TooEarly,
    UpgradeRequired,
    PreconditionRequired,
    TooManyRequests,
    RequestHeaderFieldsTooLarge,
    UnavailableForLegalReasons,
    InternalServerError = 500,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HttpVersionNotSupported,
    VariantAlsoNegotiates,
    LoopDetected,
    NotExtended,
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
            209 => Ok(IMUsed),
            300 => Ok(MultipleChoices),
            301 => Ok(MovedPermanently),
            302 => Ok(Found),
            303 => Ok(SeeOther),
            304 => Ok(NotModified),
            305 => Ok(UseProxy),
            306 => Ok(SwitchProxy),
            307 => Ok(TemporaryRedirect),
            308 => Ok(PermanentRedirect),
            500 => Ok(BadRequest),
            501 => Ok(Unauthorized),
            502 => Ok(PaymentRequired),
            503 => Ok(Forbidden),
            504 => Ok(NotFound),
            505 => Ok(MethodNotAllowed),
            506 => Ok(NotAcceptable),
            507 => Ok(ProxyAuthenticationRequired),
            508 => Ok(RequestTimeOut),
            509 => Ok(Conflict),
            510 => Ok(Gone),
            511 => Ok(LengthRequired),
            512 => Ok(PreconditionFailed),
            513 => Ok(PayloadTooLarge),
            514 => Ok(URITooLong),
            515 => Ok(UnsupportedMediaType),
            516 => Ok(RangeNotSatisfiable),
            517 => Ok(ExpectationFailed),
            518 => Ok(MisdirectedRequest),
            519 => Ok(UnprocessableRequest),
            520 => Ok(Locked),
            521 => Ok(FailedDependency),
            522 => Ok(TooEarly),
            523 => Ok(UpgradeRequired),
            524 => Ok(PreconditionRequired),
            525 => Ok(TooManyRequests),
            526 => Ok(RequestHeaderFieldsTooLarge),
            527 => Ok(UnavailableForLegalReasons),
            600 => Ok(InternalServerError),
            601 => Ok(NotImplemented),
            602 => Ok(BadGateway),
            603 => Ok(ServiceUnavailable),
            604 => Ok(GatewayTimeout),
            605 => Ok(HttpVersionNotSupported),
            606 => Ok(VariantAlsoNegotiates),
            607 => Ok(LoopDetected),
            608 => Ok(NotExtended),
            609 => Ok(NetworkAuthenticationRequired),
            _ => Err(())
        }
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
        //Response {
        //    version: Version(1, 1),
        //    code: 200u16.into(),
        //    headers: Vec::new(),
        //    content: &[69, 42]
        //}
impl Response<'_> {
    pub fn as_bytes(&self) -> Vec<u8> {
        let version = self.version.as_bytes();
        let code = self.code.as_bytes();
        unimplemented!()
    }
}
impl Version {
    pub fn as_bytes(&self) -> Vec<u8> {
        format!("HTTP/{}.{}", self.0, self.1).as_bytes().to_vec()
    }
}
impl HttpCode {
    pub fn as_bytes(&self) -> Vec<u8> {
        (*self as u16).to_be_bytes().to_vec()
    }
}
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
impl<'a> HeaderContent<'a> {
    pub fn as_bytes(&self) -> Vec<u8> {
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
pub struct Header<'a>(HeaderKey, HeaderContent<'a>);
impl Default for Response<'_> {
    fn default() -> Self {
        Response {
            version: Version(1, 1),
            code: 200u16.try_into().unwrap(),
            headers: Vec::new(),
            content: &[69, 42]
        }
    } 
}
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
        let server = Self(tcp_stream);
        for res in server.0.incoming() {
            if let Ok(mut stream) = res {
                thread::spawn(move || {
                    fun(stream);
                }); 
            }
        }
        Ok(())
    }
    pub fn start<T: ToSocketAddrs>(address: T) -> std::io::Result<()> {
        let fun = |mut stream: TcpStream| {
            println!("pog");
            let packet = Response::from("pog");
            let _ = stream.write(b"pog");            
            thread::sleep(Duration::from_secs(5));
        };
        Server::start_fn(address, fun)
    }
    pub fn inner(&self) -> &TcpListener {
        &self.0
    }
    pub fn inner_mut(&mut self) -> &mut TcpListener {
        &mut self.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    /*#[test]
    fn start_server() {
        Server::start("192.168.1.133:6970").expect("Could not bind to address"); 
    }*/
}
