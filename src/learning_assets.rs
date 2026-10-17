// Production illustrations are embedded in the native server as well as shipped to Sites.
pub fn asset(path: &str) -> Option<(&'static str, &'static [u8])> {
    match path {
        "/art/journey/genesis.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis.webp").as_slice(),
        )),
        "/art/learn/ark/construction.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/learn/ark/construction.webp").as_slice(),
        )),
        "/art/learn/ark/door.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/learn/ark/door.webp").as_slice(),
        )),
        "/art/learn/ark/food.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/learn/ark/food.webp").as_slice(),
        )),
        "/art/learn/ark/scene.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/learn/ark/scene.webp").as_slice(),
        )),
        "/art/learn/ark/window.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/learn/ark/window.webp").as_slice(),
        )),
        "/art/learn/sower/good-soil.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/learn/sower/good-soil.webp").as_slice(),
        )),
        "/art/learn/sower/path.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/learn/sower/path.webp").as_slice(),
        )),
        "/art/learn/sower/rocky.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/learn/sower/rocky.webp").as_slice(),
        )),
        "/art/learn/sower/scene.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/learn/sower/scene.webp").as_slice(),
        )),
        "/art/learn/sower/thorns.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/learn/sower/thorns.webp").as_slice(),
        )),
        "/social-preview.jpg" => Some((
            "image/jpeg",
            include_bytes!("../web/social-preview.jpg").as_slice(),
        )),
        _ => None,
    }
}
