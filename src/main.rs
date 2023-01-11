use async_std::io::{self, BufRead, BufReader};
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use chrono;
use clap::{Parser, ValueEnum};
use colored::*;
use futures::future::join;
use futures::{ready, AsyncRead, AsyncReadExt, AsyncWrite};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The address to bind local TCP server, i.e. 127.0.0.1:5000
    listener_addr: String,

    /// The target address to proxy TCP connections to, i.e. 10.0.0.1:6000
    target_addr: String,

    /// Name of the person to greet
    #[arg(short, long, value_enum, default_value_t = Encoding::Utf8)]
    encoding: Encoding,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Encoding {
    /// Display data as UTF8 lossy
    Utf8,
    /// Display data as hex lowercase
    Hex,
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("Listening {}", &args.listener_addr);
    println!("Target {}", &args.target_addr);

    let listener = TcpListener::bind(&args.listener_addr).await?;

    loop {
        let (incoming_stream, incoming_address) = listener.accept().await?;
        let target_addr = args.target_addr.clone();

        async_std::task::spawn(async move {
            let target_stream = TcpStream::connect(&target_addr).await?;

            // let (reader, writer) = &mut (&stream, &stream);
            // io::copy(reader, writer).await?;

            let (mut incoming_read, mut incoming_write) = incoming_stream.split();
            let (mut target_read, mut target_write) = target_stream.split();

            let pipe_read_write = copy(
                &mut incoming_read,
                &mut target_write,
                args.encoding,
                format!("{} -> {}", &incoming_address, &target_addr),
            );
            let pipe_write_read = copy(
                &mut target_read,
                &mut incoming_write,
                args.encoding,
                format!("{} -> {}", &target_addr, &incoming_address),
            );

            let (res1, res2) = join(pipe_read_write, pipe_write_read).await;
            res1?;
            res2?;

            std::io::Result::Ok(())
        });
    }
}

async fn copy<R, W>(
    reader: &mut R,
    writer: &mut W,
    encoding: Encoding,
    prefix: String,
) -> io::Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    struct CopyFuture<R, W> {
        reader: R,
        writer: W,
        amt: u64,
        encoding: Encoding,
        prefix: String,
    }

    impl<R, W> Future for CopyFuture<R, W>
    where
        R: BufRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        type Output = io::Result<u64>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut this = &mut *self;

            loop {
                let buffer = ready!(Pin::new(&mut this.reader).poll_fill_buf(cx))?;
                if buffer.is_empty() {
                    ready!(Pin::new(&mut this.writer).poll_flush(cx))?;
                    return Poll::Ready(Ok(this.amt));
                }

                let buf_str = match this.encoding {
                    Encoding::Utf8 => String::from_utf8_lossy(buffer),
                    Encoding::Hex => hex::encode(buffer).into(),
                };

                let now = chrono::Utc::now().to_string();
                println!("{}  {}\n{}", now.red(), &this.prefix.red(), buf_str);

                let i = ready!(Pin::new(&mut this.writer).poll_write(cx, buffer))?;
                if i == 0 {
                    return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                }
                this.amt += i as u64;
                Pin::new(&mut this.reader).consume(i);
            }
        }
    }

    let future = CopyFuture {
        reader: BufReader::new(reader),
        writer,
        amt: 0,
        encoding,
        prefix: prefix.to_owned(),
    };

    future.await
}
