use std::io::Write;
use std::sync::Mutex;

use failure::Error;
use futures::future::Future;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Chunk;
use hyper::Response;

fn router(path: &str) -> Response<Body> {
    let mut parts = path[1..].split('/');
    if let Some(frag_count) = parts.next().and_then(|s| s.parse().ok()) {
        if let Some(frag_size) = parts.next().and_then(|s| s.parse().ok()) {
            return stream(frag_count, frag_size);
        }
    }

    response(404, "usage: /count/size")
}

fn stream(frag_count: usize, frag_size: usize) -> Response<Body> {
    let mut parts = Vec::new();
    for i in 0..frag_count {
        let frag_content = ((i % 256) as u8) % 26 + b'a';
        parts.push(Chunk::from(vec![frag_content; frag_size]));
    }
    response(200, to_body(parts.into_iter()))
}

fn main() -> Result<(), Error> {
    let last = Mutex::new(chrono::Local::now());

    let mut args = std::env::args();
    let _ = args.next();

    env_logger::Builder::new()
        .format(move |buf, record| {
            let now = chrono::Local::now();
            let diff = {
                let mut lock = last.lock().unwrap();
                let diff = now.signed_duration_since(*lock);
                *lock = now;
                diff
            };
            writeln!(
                buf,
                "{} (+{}) {} {}",
                now.format("%Y-%m-%dT%H:%M:%S%.9f"),
                diff,
                record.target(),
                record.args()
            )
        })
        .filter(Some("hyper::proto::h1::io"), log::LevelFilter::Debug)
        .filter(Some("tokio_reactor::registration"), log::LevelFilter::Debug)
        .filter(Some("hyper::proto::h1::conn"), log::LevelFilter::Debug)
        .init();

    let addr = ([0, 0, 0, 0], 4432).into();

    let server = hyper::Server::bind(&addr)
        .serve(make_service_fn(move |_| {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                Ok::<_, hyper::Error>(router(req.uri().path()))
            }))
        }))
        .map_err(|e| eprintln!("server error: {}", e));

    println!("listening on http://localhost:4432/29/3");

    hyper::rt::run(server);

    Ok(())
}

fn response(status: u16, body: impl Into<Body>) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(body.into())
        .expect("static builder")
}

fn to_body<I: 'static + Send + Sync + Iterator<Item = Chunk>>(chunks: I) -> Body {
    Body::wrap_stream(futures::stream::iter_result(
        chunks.map(|c| -> Result<_, std::io::Error> { Ok(c) }),
    ))
}
