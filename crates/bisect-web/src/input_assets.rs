//! Eager input-module dependencies of lab.js, shared by native and catalog hosts.
pub const INPUT_ASSETS: &[(&str, &str)] = &[
    ("multiscale-input.js",include_str!("../../../web/lab/multiscale-input.js")),
    ("multiscale-ui.js",include_str!("../../../web/lab/multiscale-ui.js")),
    ("election-input.js",include_str!("../../../web/lab/election-input.js")),
    ("partisan-input.js",include_str!("../../../web/lab/partisan-input.js")),
    ("character-input.js",include_str!("../../../web/lab/character-input.js")),
    ("demographic-input.js",include_str!("../../../web/lab/demographic-input.js")),
    ("project.js",include_str!("../../../web/lab/project.js")),
    ("package-files.js",include_str!("../../../web/lab/package-files.js")),
];
