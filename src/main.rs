use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode, Uri},
    response::Response,
    routing::get,
    Router,
};

mod cover_assets;
mod learning_assets;

async fn serve(uri: Uri) -> Response {
    if uri.path() == "/hosted-voice.js" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
            .body(Body::from(include_str!("../web/hosted-voice.js")))
            .expect("hosted narration adapter response");
    }
    if let Some((mime, bytes)) = learning_assets::asset(uri.path()) {
        return Response::builder()
            .header(header::CONTENT_TYPE, mime)
            .body(Body::from(bytes))
            .expect("learning asset response");
    }
    if uri.path() == "/codex.js" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
            .body(Body::from(include_str!("../web/codex.js")))
            .unwrap();
    }
    if uri.path() == "/icons/magnifying-glass-thin.svg" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "image/svg+xml")
            .body(Body::from(include_str!(
                "../web/icons/magnifying-glass-thin.svg"
            )))
            .unwrap();
    }
    if uri.path() == "/progress.js" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
            .body(Body::from(include_str!("../web/progress.js")))
            .expect("progress adapter response");
    }
    if uri.path() == "/journey.js" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
            .body(Body::from(include_str!("../web/journey.js")))
            .expect("journey adapter response");
    }
    if uri.path() == "/art/today/psalms.webp" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "image/webp")
            .body(Body::from(
                include_bytes!("../web/art/today/psalms.webp").as_slice(),
            ))
            .expect("today art response");
    }
    let book_material = match uri.path() {
        "/art/book-binding.webp" => Some(include_bytes!("../web/art/book-binding.webp").as_slice()),
        "/art/gold-foil.webp" => Some(include_bytes!("../web/art/gold-foil.webp").as_slice()),
        _ => None,
    };
    if let Some(bytes) = book_material {
        return Response::builder()
            .header(header::CONTENT_TYPE, "image/webp")
            .body(Body::from(bytes))
            .expect("book material response is valid");
    }
    if uri.path() == "/fonts/inter-variable.ttf" {
        return Response::builder()
            .header("content-type", "font/ttf")
            .body(Body::from(
                include_bytes!("../web/fonts/inter-variable.ttf").as_slice(),
            ))
            .unwrap();
    }
    if uri.path() == "/art/today/psalm23-shepherd.webp" {
        return Response::builder()
            .header("content-type", "image/webp")
            .body(Body::from(
                include_bytes!("../web/art/today/psalm23-shepherd.webp").as_slice(),
            ))
            .unwrap();
    }
    if uri.path() == "/fonts/cormorant-garamond.ttf" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "font/ttf")
            .body(Body::from(
                include_bytes!("../web/fonts/cormorant-garamond.ttf").as_slice(),
            ))
            .expect("display typeface response is valid");
    }
    if uri.path() == "/fonts/eb-garamond.ttf" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "font/ttf")
            .body(Body::from(
                include_bytes!("../web/fonts/eb-garamond.ttf").as_slice(),
            ))
            .expect("book typeface response is valid");
    }
    if let Some(bytes) = cover_assets::get(uri.path()) {
        return Response::builder()
            .header(header::CONTENT_TYPE, "image/webp")
            .body(Body::from(bytes))
            .expect("cover illustration response is valid");
    }
    if uri.path() == "/library.js" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
            .body(Body::from(include_str!("../web/library.js")))
            .expect("library script response is valid");
    }
    let object_art = match uri.path() {
        "/art/objects/courtyard.webp" => {
            Some(include_bytes!("../web/art/objects/courtyard.webp").as_slice())
        }
        "/art/objects/altar.webp" => {
            Some(include_bytes!("../web/art/objects/altar.webp").as_slice())
        }
        "/art/objects/basin.webp" => {
            Some(include_bytes!("../web/art/objects/basin.webp").as_slice())
        }
        "/art/objects/tent.webp" => Some(include_bytes!("../web/art/objects/tent.webp").as_slice()),
        "/art/objects/table.webp" => {
            Some(include_bytes!("../web/art/objects/table.webp").as_slice())
        }
        "/art/objects/lamp.webp" => Some(include_bytes!("../web/art/objects/lamp.webp").as_slice()),
        "/art/objects/veil.webp" => Some(include_bytes!("../web/art/objects/veil.webp").as_slice()),
        "/art/objects/ark.webp" => Some(include_bytes!("../web/art/objects/ark.webp").as_slice()),
        _ => None,
    };
    if let Some(bytes) = object_art {
        return Response::builder()
            .header(header::CONTENT_TYPE, "image/webp")
            .body(Body::from(bytes))
            .expect("object illustration response is valid");
    }
    if uri.path() == "/art/tabernacle.webp" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "image/webp")
            .body(Body::from(
                include_bytes!("../web/art/tabernacle.webp").as_slice(),
            ))
            .expect("static illustration response is valid");
    }
    if uri.path() == "/logo.png" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "image/png")
            .body(Body::from(include_bytes!("../web/logo.png").as_slice()))
            .expect("static logo response is valid");
    }
    let map_asset = match uri.path() {
        "/reader-context.js" => Some((
            "text/javascript; charset=utf-8",
            include_str!("../web/reader-context.js"),
        )),
        "/atlas.js" => Some((
            "text/javascript; charset=utf-8",
            include_str!("../web/atlas.js"),
        )),
        "/terrain.js" => Some((
            "text/javascript; charset=utf-8",
            include_str!("../web/terrain.js"),
        )),
        "/vendor/maplibre/maplibre-gl.mjs" => Some((
            "text/javascript; charset=utf-8",
            include_str!("../web/vendor/maplibre/maplibre-gl.mjs"),
        )),
        "/vendor/maplibre/maplibre-gl-worker.mjs" => Some((
            "text/javascript; charset=utf-8",
            include_str!("../web/vendor/maplibre/maplibre-gl-worker.mjs"),
        )),
        _ => None,
    };
    if let Some((mime, content)) = map_asset {
        return Response::builder()
            .header(header::CONTENT_TYPE, mime)
            .body(Body::from(content))
            .expect("map asset response is valid");
    }
    if let Some(asset) = uri.path().strip_prefix("/vendor/kokoro/") {
        let safe = !asset.is_empty()
            && asset.split('/').all(|part| {
                !part.is_empty()
                    && part != "."
                    && part != ".."
                    && part
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
            });
        if safe {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("web/vendor/kokoro")
                .join(asset);
            if let Ok(content) = std::fs::read(path) {
                let mime = if asset.ends_with(".wasm") {
                    "application/wasm"
                } else if asset.ends_with(".js") || asset.ends_with(".mjs") {
                    "text/javascript; charset=utf-8"
                } else if asset.ends_with(".json") {
                    "application/json"
                } else {
                    "text/plain; charset=utf-8"
                };
                return Response::builder()
                    .header(header::CONTENT_TYPE, mime)
                    .body(Body::from(content))
                    .expect("voice asset response is valid");
            }
        }
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Voice asset not found"))
            .expect("missing voice asset response is valid");
    }
    let voice_asset = match uri.path() {
        "/local-voice.js" => Some(include_str!("../web/local-voice.js")),
        "/local-voice-worker.js" => Some(include_str!("../web/local-voice-worker.js")),
        _ => None,
    };
    if let Some(content) = voice_asset {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
            .body(Body::from(content))
            .expect("voice script response is valid");
    }
    if uri.path() == "/experience.js" {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
            .body(Body::from(include_str!("../web/experience.js")))
            .expect("experience script response is valid");
    }
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
