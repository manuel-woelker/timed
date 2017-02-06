
extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate byteorder;
extern crate time;

use std::io;
use std::str;
use tokio_proto::pipeline::ServerProto;
use tokio_core::io::Io;

use tokio_service::Service;
use tokio_proto::TcpServer;

use futures::{future, Future, BoxFuture, Stream, Sink, Poll, Async, StartSend, AsyncSink};

use byteorder::{BigEndian, ByteOrder};


pub struct TimeProto;

pub struct TimeTransport<T: Io + 'static> {
    io: T,
    done: bool,
}

impl<T: Io + 'static> Stream for TimeTransport<T> {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<()>, io::Error> {
        if !self.done {
            self.done = true;
            return Ok(Async::Ready(Some(())));
        }
        return Ok(Async::Ready(None));
    }

}


impl<T: Io + 'static> Sink for TimeTransport<T> {
    type SinkItem = u32;
    type SinkError = io::Error;

    fn start_send(&mut self, item: u32) -> StartSend<u32, io::Error> {

        let mut buffer = [0u8; 4];
        BigEndian::write_u32(&mut buffer, item);
        self.io.write(&buffer)?;
        self.io.flush()?;

        return Ok(AsyncSink::Ready);
    }

    fn poll_complete(&mut self) -> Poll<(), io::Error> {
        return Ok(Async::Ready(()));
    }
}

impl<T: Io + 'static> ServerProto<T> for TimeProto {
    /// For this protocol style, `Request` matches the codec `In` type
    type Request = ();

    /// For this protocol style, `Response` matches the coded `Out` type
    type Response = u32;

    /// A bit of boilerplate to hook in the codec:
    type Transport = TimeTransport<T>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(TimeTransport{io: io, done: false})
    }
}

pub struct TimeService;

impl Service for TimeService {
    // These types must match the corresponding protocol types:
    type Request = ();
    type Response = u32;

    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;

    // The future for computing the response; box it for simplicity.
    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, _: Self::Request) -> Self::Future {
        // In this case, the response is immediate.
        future::ok(time::get_time().sec as u32 + 2208988800).boxed()
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