use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode, Uri},
    response::Response,
    routing::get,
    Router,
};

async fn serve(uri: Uri) -> Response {
    let raw_uri = uri.path_and_query().map_or("/", |value| value.as_str());
    let rendered = rust_bible::route(raw_uri, false);
    let mut builder = Response::builder()
        .status(StatusCode::from_u16(rendered.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .header(header::CONTENT_TYPE, rendered.content_type);
    if let Some(location) = rendered.location {
        if let Ok(location) = HeaderValue::from_str(&location) {
            builder = builder.header(header::LOCATION, location);
        }
    }
    builder.body(Body::from(rendered.body)).unwrap_or_else(|_| {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("failed to render response"))
            .expect("static fallback response is valid")
    })
}

#[tokio::main]
async fn main() {
    let loaded = rust_bible::route("/", false);
    println!(
        "KJV reader initialized (home response: {} bytes)",
        loaded.body.len()
    );
    let app = Router::new().fallback(get(serve));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
