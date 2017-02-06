
extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate byteorder;

use std::io;
use std::io::Write;
use std::str;
use tokio_core::io::{Codec, EasyBuf};
use tokio_proto::pipeline::ServerProto;
use tokio_core::io::{Io, Framed};

use tokio_service::Service;
use tokio_proto::TcpServer;

use futures::{future, Future, BoxFuture};

use byteorder::{BigEndian, WriteBytesExt};

pub struct TimeCodec;

impl Codec for TimeCodec {
    type In = u32;
    type Out = u32;
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        let len = buf.len();
        println!("DECODE {}", len);
        if len <= 0 {
            return Ok(None);
        }
        buf.drain_to(len);
        Ok(Some(0))
        //        Ok(None)
    }
    fn encode(&mut self, msg: u32, buf: &mut Vec<u8>)
              -> io::Result<()>
    {
        println!("ENCODE {}", buf.capacity());
        buf.resize(4, 0);
//        buf.extend(msg.as_bytes());
        let mut writer = io::Cursor::new(buf.as_mut());
        writer.write_u32::<BigEndian>(msg).unwrap();
        Ok(())
    }

}

pub struct TimeProto;

impl<T: Io + 'static> ServerProto<T> for TimeProto {
    /// For this protocol style, `Request` matches the codec `In` type
    type Request = u32;

    /// For this protocol style, `Response` matches the coded `Out` type
    type Response = u32;

    /// A bit of boilerplate to hook in the codec:
    type Transport = Framed<T, TimeCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(TimeCodec))
    }
}

pub struct TimeService;

impl Service for TimeService {
    // These types must match the corresponding protocol types:
    type Request = u32;
    type Response = u32;

    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;

    // The future for computing the response; box it for simplicity.
    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        println!("CALL");
        // In this case, the response is immediate.
        future::ok(1634952804).boxed()
    }
}

fn main() {
    // Specify the localhost address
    let addr = "0.0.0.0:12345".parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(TimeProto, addr);

    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    println!("Listening on {}", addr);
    server.serve(|| Ok(TimeService));
}