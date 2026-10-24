// Production illustrations are embedded in the native server as well as shipped to Sites.
pub fn asset(path: &str) -> Option<(&'static str, &'static [u8])> {
    match path {
        "/art/journey/exodus-signs-rods.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/exodus-signs-rods.svg").as_slice(),
        )),
        "/art/journey/exodus-signs-river.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/exodus-signs-river.webp").as_slice(),
        )),
        "/art/journey/exodus-signs-frogs.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/exodus-signs-frogs.webp").as_slice(),
        )),
        "/art/journey/exodus-signs-lice.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/exodus-signs-lice.svg").as_slice(),
        )),
        "/art/journey/exodus-signs-flies.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/exodus-signs-flies.svg").as_slice(),
        )),
        "/art/journey/exodus-expansion-basket.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/exodus-expansion-basket.webp").as_slice(),
        )),
        "/art/journey/exodus-expansion-bush.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/exodus-expansion-bush.webp").as_slice(),
        )),
        "/art/journey/exodus-expansion-stubble.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/exodus-expansion-stubble.webp").as_slice(),
        )),
        "/art/journey/exodus-expansion-midwives.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/exodus-expansion-midwives.svg").as_slice(),
        )),
        "/art/journey/exodus-expansion-rod-sequence.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/exodus-expansion-rod-sequence.svg").as_slice(),
        )),
        "/art/journey/exodus-expansion-family-line.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/exodus-expansion-family-line.svg").as_slice(),
        )),
        "/art/journey/abraham-sarah-visitors-v2.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/abraham-sarah-visitors-v2.webp").as_slice(),
        )),
        "/art/journey/judah-pledge-detail.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/judah-pledge-detail.webp").as_slice(),
        )),
        "/art/journey/judah-tamar-pledge.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/judah-tamar-pledge.webp").as_slice(),
        )),
        "/art/journey/ot-exodus-red-sea-crossing.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-exodus-red-sea-crossing.webp").as_slice(),
        )),
        "/art/journey/ot-leviticus-live-goat.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-leviticus-live-goat.webp").as_slice(),
        )),
        "/art/journey/numbers-balaam-donkey.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/numbers-balaam-donkey.webp").as_slice(),
        )),
        "/art/journey/deuteronomy-choose-life.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/deuteronomy-choose-life.webp").as_slice(),
        )),
        "/art/journey/ot-joshua-jordan-crossing.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-joshua-jordan-crossing.webp").as_slice(),
        )),
        "/art/journey/ot-judges-deborah-and-barak.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-judges-deborah-and-barak.webp").as_slice(),
        )),
        "/art/journey/ot-ruth-gleaning-encounter.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-ruth-gleaning-encounter.webp").as_slice(),
        )),
        "/art/journey/ot-1samuel-david-goliath.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-1samuel-david-goliath.webp").as_slice(),
        )),
        "/art/journey/ot-2samuel-ark-procession.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-2samuel-ark-procession.webp").as_slice(),
        )),
        "/art/journey/ot-1kings-temple-cloud.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-1kings-temple-cloud.webp").as_slice(),
        )),
        "/art/journey/ot-2kings-naaman-jordan.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-2kings-naaman-jordan.webp").as_slice(),
        )),
        "/art/journey/ot-2samuel-ark-procession.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-2samuel-ark-procession.webp").as_slice(),
        )),
        "/art/journey/ot-1kings-temple-cloud.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-1kings-temple-cloud.webp").as_slice(),
        )),
        "/art/journey/ot-ezra-temple-foundation.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-ezra-temple-foundation.webp").as_slice(),
        )),
        "/art/journey/ot-nehemiah-wall-watch.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-nehemiah-wall-watch.webp").as_slice(),
        )),
        "/art/journey/ot-esther-second-banquet.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-esther-second-banquet.webp").as_slice(),
        )),
        "/art/journey/ot-job-whirlwind.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-job-whirlwind.webp").as_slice(),
        )),
        "/art/journey/ot-psalms-shepherd.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-psalms-shepherd.webp").as_slice(),
        )),
        "/art/journey/ot-proverbs-wisdom-calls.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-proverbs-wisdom-calls.webp").as_slice(),
        )),
        "/art/journey/ot-ecclesiastes-times.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ot-ecclesiastes-times.webp").as_slice(),
        )),
        "/art/journey/song-beloved-spring.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/song-beloved-spring.webp").as_slice(),
        )),
        "/art/journey/isaiah-branch-and-peace.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/isaiah-branch-and-peace.webp").as_slice(),
        )),
        "/art/journey/jeremiah-potter.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/jeremiah-potter.webp").as_slice(),
        )),
        "/art/journey/lamentations-city-mourns.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/lamentations-city-mourns.webp").as_slice(),
        )),
        "/art/journey/ezekiel-dry-bones.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/ezekiel-dry-bones.webp").as_slice(),
        )),
        "/art/journey/daniel-lions-dawn.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/daniel-lions-dawn.webp").as_slice(),
        )),
        "/art/journey/hosea-family-sign.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/hosea-family-sign.webp").as_slice(),
        )),
        "/art/journey/joel-locust-devastation.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/joel-locust-devastation.webp").as_slice(),
        )),
        "/art/journey/amos-summer-fruit.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/amos-summer-fruit.webp").as_slice(),
        )),
        "/art/journey/obadiah-edom-pride.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/obadiah-edom-pride.webp").as_slice(),
        )),
        "/art/journey/passage-jonah.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/passage-jonah.webp").as_slice(),
        )),
        "/art/journey/micah-what-god-requires.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/micah-what-god-requires.webp").as_slice(),
        )),
        "/art/journey/nahum-nineveh-fall.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nahum-nineveh-fall.webp").as_slice(),
        )),
        "/art/journey/habakkuk-watchpost.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/habakkuk-watchpost.webp").as_slice(),
        )),
        "/art/journey/zephaniah-restoration.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/zephaniah-restoration.webp").as_slice(),
        )),
        "/art/journey/haggai-rebuild.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/haggai-rebuild.webp").as_slice(),
        )),
        "/art/journey/zechariah-lampstand.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/zechariah-lampstand.webp").as_slice(),
        )),
        "/art/journey/malachi-messenger-refiner.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/malachi-messenger-refiner.webp").as_slice(),
        )),
        "/art/journey/nt-matthew.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-matthew.webp").as_slice(),
        )),
        "/art/journey/nt-mark.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-mark.webp").as_slice(),
        )),
        "/art/journey/nt-luke.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-luke.webp").as_slice(),
        )),
        "/art/journey/nt-john.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-john.webp").as_slice(),
        )),
        "/art/journey/nt-acts.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-acts.webp").as_slice(),
        )),
        "/art/journey/nt-romans.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-romans.webp").as_slice(),
        )),
        "/art/journey/nt-1corinthians.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-1corinthians.webp").as_slice(),
        )),
        "/art/journey/nt-2-corinthians.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-2-corinthians.webp").as_slice(),
        )),
        "/art/journey/nt-galatians.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-galatians.webp").as_slice(),
        )),
        "/art/journey/nt-ephesians.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-ephesians.webp").as_slice(),
        )),
        "/art/journey/nt-philippians.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-philippians.webp").as_slice(),
        )),
        "/art/journey/nt-colossians.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-colossians.webp").as_slice(),
        )),
        "/art/journey/nt-1-thessalonians.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-1-thessalonians.webp").as_slice(),
        )),
        "/art/journey/nt-2-thessalonians.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-2-thessalonians.webp").as_slice(),
        )),
        "/art/journey/nt-1-timothy-riches-generosity.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-1-timothy-riches-generosity.webp").as_slice(),
        )),
        "/art/journey/nt-2-timothy-good-soldier.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-2-timothy-good-soldier.webp").as_slice(),
        )),
        "/art/journey/nt-titus-maintain-good-works.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-titus-maintain-good-works.webp").as_slice(),
        )),
        "/art/journey/nt-philemon-appeal-account.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-philemon-appeal-account.webp").as_slice(),
        )),
        "/art/journey/nt-hebrews-run-with-endurance.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-hebrews-run-with-endurance.webp").as_slice(),
        )),
        "/art/journey/nt-james-bit-and-rudder.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-james-bit-and-rudder.webp").as_slice(),
        )),
        "/art/journey/nt-1-peter-living-stones.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-1-peter-living-stones.webp").as_slice(),
        )),
        "/art/journey/nt-2-peter-lamp-daystar.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-2-peter-lamp-daystar.webp").as_slice(),
        )),
        "/art/journey/nt-1-john-walk-in-light.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-1-john-walk-in-light.webp").as_slice(),
        )),
        "/art/journey/nt-2-john-truth-and-love.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-2-john-truth-and-love.webp").as_slice(),
        )),
        "/art/journey/nt-3-john-hospitality-support.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-3-john-hospitality-support.webp").as_slice(),
        )),
        "/art/journey/nt-jude-mercy-rescue.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-jude-mercy-rescue.webp").as_slice(),
        )),
        "/art/journey/nt-revelation-river-of-life.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/nt-revelation-river-of-life.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-a-ark-building.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-a-ark-building.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-a-covenant-bow.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-a-covenant-bow.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-a-babel-bricks.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-a-babel-bricks.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-a-abram-departs.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-a-abram-departs.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-a-abram-lot-separate.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-a-abram-lot-separate.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-a-melchizedek-bread-wine.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-a-melchizedek-bread-wine.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-a-abram-stars.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-a-abram-stars.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-a-hagar-fountain.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-a-hagar-fountain.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-a-lot-zoar.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-a-lot-zoar.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-a-sarah-restored.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-a-sarah-restored.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-b-hagar-well.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-hagar-well.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-b-ram-thicket.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-ram-thicket.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-b-machpelah-silver.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-machpelah-silver.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-b-rebekah-camels.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-rebekah-camels.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-b-jacob-esau-stew.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-jacob-esau-stew.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-b-isaac-rehoboth.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-isaac-rehoboth.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-b-isaac-blesses-jacob.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-isaac-blesses-jacob.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-b-jacob-ladder-dream.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-jacob-ladder-dream.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-b-jacob-rachel-well.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-jacob-rachel-well.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-b-peeled-rods-flocks.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-peeled-rods-flocks.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-b-covenant-stone-heap.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-covenant-stone-heap.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-b-jacob-wrestles.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-b-jacob-wrestles.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-c-33-reunion.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-33-reunion.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-c-35-bethel-altar.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-35-bethel-altar.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-37-joseph-coat.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-37-joseph-coat.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-c-39-left-garment.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-39-left-garment.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-40-cupbearer-dream.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-40-cupbearer-dream.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-41-joseph-investiture.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-41-joseph-investiture.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-42-brothers-bow.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-42-brothers-bow.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-43-benjamin-meal.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-43-benjamin-meal.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-44-silver-cup.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-44-silver-cup.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-c-45-joseph-benjamin-embrace.webp" => Some((
            "image/webp",
            include_bytes!(
                "../web/art/journey/genesis-expansion-c-45-joseph-benjamin-embrace.webp"
            )
            .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-46-family-wagons.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-46-family-wagons.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-47-joseph-grain-exchange.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-47-joseph-grain-exchange.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-48-crossed-hands.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-48-crossed-hands.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-49-judah-lion-comparison.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-49-judah-lion-comparison.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-c-50-joseph-coffin.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-c-50-joseph-coffin.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-diagram-10.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/genesis-expansion-diagram-10.svg").as_slice(),
        )),
        "/art/journey/genesis-expansion-diagram-17.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/genesis-expansion-diagram-17.svg").as_slice(),
        )),
        "/art/journey/genesis-expansion-diagram-34.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/genesis-expansion-diagram-34.svg").as_slice(),
        )),
        "/art/journey/genesis-expansion-diagram-36.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/genesis-expansion-diagram-36.svg").as_slice(),
        )),
        "/art/journey/genesis-expansion-creation-land.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-creation-land.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-east-of-eden.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-east-of-eden.webp").as_slice(),
        )),
        "/art/journey/genesis-expansion-brothers-offerings.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/genesis-expansion-brothers-offerings.webp")
                .as_slice(),
        )),
        "/art/journey/genesis-expansion-family-line.svg" => Some((
            "image/svg+xml",
            include_bytes!("../web/art/journey/genesis-expansion-family-line.svg").as_slice(),
        )),
        "/art/journey/origins-patriarchs.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/origins-patriarchs.webp").as_slice(),
        )),
        "/art/journey/egypt.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/egypt.webp").as_slice(),
        )),
        "/art/journey/sinai-wilderness.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/sinai-wilderness.webp").as_slice(),
        )),
        "/art/journey/canaan-highlands.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/canaan-highlands.webp").as_slice(),
        )),
        "/art/journey/jerusalem-temple-return.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/jerusalem-temple-return.webp").as_slice(),
        )),
        "/art/journey/babylon-exile.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/babylon-exile.webp").as_slice(),
        )),
        "/art/journey/persian-court.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/persian-court.webp").as_slice(),
        )),
        "/art/journey/wisdom-poetry.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/wisdom-poetry.webp").as_slice(),
        )),
        "/art/journey/prophetic-symbolic.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/prophetic-symbolic.webp").as_slice(),
        )),
        "/art/journey/galilee-judea.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/galilee-judea.webp").as_slice(),
        )),
        "/art/journey/early-church-house.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/early-church-house.webp").as_slice(),
        )),
        "/art/journey/mediterranean-cities.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/mediterranean-cities.webp").as_slice(),
        )),
        "/art/journey/apocalyptic-vision.webp" => Some((
            "image/webp",
            include_bytes!("../web/art/journey/apocalyptic-vision.webp").as_slice(),
        )),
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
