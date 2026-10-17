//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

///
///
const AMM_SIDE: &[(&str, &str)] = &[
    ("programs/carpenter_amm/Cargo.toml", "programs/carpenter_amm/src"),
    ("tests/programs/mock_hook/Cargo.toml", "tests/programs/mock_hook/src"),
    ("crates/carpenter-types/Cargo.toml", "crates/carpenter-types/src"),


    ("crates/carpenter-mint-gate/Cargo.toml", "crates/carpenter-mint-gate/src"),
];

///
///
///
///
///
///
///
///
const FLOOR_CRATES: &[&str] = &["floor-types", "floor-math"];
const FLOOR_PATHS: &[&str] = &["floor_types::", "floor_math::"];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let p = entry.expect("dir entry").path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn strip_comment(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

fn manifest_dependency_lines(manifest: &str) -> Vec<String> {


    let mut out = Vec::new();
    let mut in_deps = false;
    for line in manifest.lines() {
        let code = line.split('#').next().unwrap_or("").trim();
        if code.starts_with('[') {
            in_deps = code.contains("dependencies");
            if in_deps && code.contains("dependencies.") {

                out.push(code.to_string());
            }
            continue;
        }
        if in_deps && !code.is_empty() {
            out.push(code.to_string());
        }
    }
    out
}

#[test]
fn the_amm_crates_link_no_floor_crate() {
    let root = repo_root();
    for (manifest, _) in AMM_SIDE {
        let path = root.join(manifest);
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{manifest}: {e}"));
        for dep in manifest_dependency_lines(&text) {
            for floor in FLOOR_CRATES {
                assert!(
                    !dep.contains(floor),
                    "{manifest} links `{floor}` (`{dep}`): the AMM side takes `carpenter-types` alone"
                );
            }
        }
    }
}

#[test]
fn the_amm_sources_spell_no_floor_path() {
    let root = repo_root();
    for (_, src) in AMM_SIDE {
        let mut files = Vec::new();
        rust_files(&root.join(src), &mut files);
        assert!(!files.is_empty(), "no .rs files under {src}");
        for f in files {
            let text = fs::read_to_string(&f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
            for (i, line) in text.lines().enumerate() {
                let code = strip_comment(line);
                for p in FLOOR_PATHS {
                    assert!(
                        !code.contains(p),
                        "{}:{}: spells `{p}` — the AMM side reaches its bytes through `carpenter_types::` only",
                        f.display(),
                        i + 1
                    );
                }
            }
        }
    }
}

#[test]
fn the_arrow_points_from_floor_types_to_carpenter_types() {
    let root = repo_root();
    let floor = fs::read_to_string(root.join("crates/floor-types/Cargo.toml")).expect("crates/floor-types/Cargo.toml");
    assert!(
        manifest_dependency_lines(&floor).iter().any(|l| l.starts_with("carpenter-types")),
        "crates/floor-types must depend on carpenter-types (the mechanism is built over the AMM's bytes)"
    );
    let carpenter = fs::read_to_string(root.join("crates/carpenter-types/Cargo.toml")).expect("crates/carpenter-types/Cargo.toml");
    let deps = manifest_dependency_lines(&carpenter);
    assert!(
        deps.iter().all(|l| !l.starts_with("floor-") && !l.contains("floor-types") && !l.contains("floor-math")),
        "crates/carpenter-types must not depend on any FLOOR crate: {deps:?}"
    );
}

///
///
#[test]
fn the_floor_names_the_doctests_refuse_are_defined_in_floor_types() {
    let root = repo_root().join("crates/floor-types/src");
    let seeds = fs::read_to_string(root.join("seeds.rs")).unwrap();
    let layout = fs::read_to_string(root.join("layout.rs")).unwrap();
    let constants = fs::read_to_string(root.join("constants.rs")).unwrap();
    let ids = fs::read_to_string(root.join("ids.rs")).unwrap();
    let disc = fs::read_to_string(root.join("disc.rs")).unwrap();
    assert!(seeds.contains("pub const FLOOR_POOL: &[u8]"));
    assert!(seeds.contains("pub const LAUNCH: &[u8]"));
    assert!(layout.contains("pub struct FloorPool"));
    assert!(layout.contains("pub struct Launch "));
    assert!(constants.contains("pub const SUPPLY:"));
    assert!(constants.contains("pub const FLOOR_HOOK_FLAGS:"));
    assert!(ids.contains("pub const FLOOR_HOOK_ID:"));
    assert!(disc.contains("pub const FLOOR_POOL: [u8; 8]"));

    let here = repo_root().join("crates/carpenter-types/src");
    for file in ["seeds.rs", "layout.rs", "constants.rs", "ids.rs", "disc.rs", "hook.rs", "keys.rs", "pod.rs", "lib.rs"] {
        let text = fs::read_to_string(here.join(file)).unwrap();
        for needle in [
            "pub const FLOOR_POOL",
            "pub struct FloorPool",
            "pub struct Launch",
            "pub const SUPPLY:",
            "pub const FLOOR_HOOK_FLAGS",
            "pub const FLOOR_HOOK_ID",
            "pub use floor_types",
        ] {
            assert!(!text.contains(needle), "crates/carpenter-types/src/{file} defines or re-exports `{needle}`");
        }
    }
}
