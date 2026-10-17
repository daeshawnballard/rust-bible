// Production illustrations are embedded in the native server as well as shipped to Sites.
pub fn asset(path: &str) -> Option<(&'static str, &'static [u8])> {
    match path {
        "/art/journey/jordan.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/jordan.webp").as_slice(),
        )),
        "/art/journey/david-goliath.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/david-goliath.webp").as_slice(),
        )),
        "/art/journey/daniel.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/daniel.webp").as_slice(),
        )),
        "/art/journey/jonah.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/jonah.webp").as_slice(),
        )),
        "/art/journey/loaves-and-sea.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/loaves-and-sea.webp").as_slice(),
        )),
        "/art/journey/emmaus.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/emmaus.webp").as_slice(),
        )),
        "/art/journey/samaria.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/samaria.webp").as_slice(),
        )),
        "/art/journey/paul-voyage.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/paul-voyage.webp").as_slice(),
        )),
        "/art/journey/genesis-immersive-v1.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-immersive-v1.webp").as_slice(),
        )),
        "/art/journey/flood.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/flood.webp").as_slice(),
        )),
        "/art/journey/wilderness.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/wilderness.webp").as_slice(),
        )),
        "/art/journey/ruth.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ruth.webp").as_slice(),
        )),
        "/art/journey/bethlehem.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/bethlehem.webp").as_slice(),
        )),
        "/art/journey/galilee.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/galilee.webp").as_slice(),
        )),
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
