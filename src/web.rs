// use mime_guess;
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use warp::{http::header::HeaderValue, path::Tail, reply::Response, Filter, Rejection, Reply};

#[derive(RustEmbed)]
#[folder = "www/"]
struct Asset;

pub async fn run_webserver(address: Option<String>) {
    match address {
        Some(addr) => {
            let index_html = warp::path::end().and_then(serve_index);
            let dist = warp::path("dist").and(warp::path::tail()).and_then(serve);
            let routes = index_html.or(dist);
            let addr: SocketAddr = addr.parse().unwrap();
            warp::serve(routes).run(addr).await;
        }
        None => {}
    }
}

async fn serve_index() -> Result<impl Reply, Rejection> {
    serve_impl("index.html")
}

async fn serve(path: Tail) -> Result<impl Reply, Rejection> {
    serve_impl(path.as_str())
}

fn serve_impl(path: &str) -> Result<impl Reply, Rejection> {
    let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = Response::new(asset.into());
    res.headers_mut().insert(
        "content-type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}
