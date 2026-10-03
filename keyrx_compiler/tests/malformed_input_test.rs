//! Malformed and hostile input: scripts and `.krx` files must be rejected
//! with an error - never a panic, an abort (stack overflow), a hang or an
//! unbounded read. The daemon compiles user scripts in-process, so any abort
//! here would take the whole keyboard remapper down.

use std::fs;
use std::path::Path;

use keyrx_compiler::error::DeserializeError;
use keyrx_compiler::parser::Parser;
use keyrx_compiler::serialize::{deserialize, read_krx, serialize, HEADER_SIZE, KRX_MAX_BYTES};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const SIMPLE: &str = "device_start(\"rfz*\");\nmap(\"VK_F13\", \"VK_A\");\ndevice_end();\n";

fn parse(dir: &TempDir, name: &str, bytes: &[u8]) -> Result<(), String> {
    let path = dir.path().join(name);
    fs::write(&path, bytes).unwrap();
    Parser::new()
        .parse_script(&path)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn compiled_simple() -> Vec<u8> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("s.rhai");
    fs::write(&path, SIMPLE).unwrap();
    let config = Parser::new().parse_script(&path).unwrap();
    serialize(&config).unwrap()
}

/// Rewrites size and hash so a corrupted body gets past the integrity
/// checks and reaches rkyv's structural validation.
fn reseal(bytes: &mut [u8]) {
    let hash: [u8; 32] = Sha256::digest(&bytes[HEADER_SIZE..]).into();
    bytes[8..40].copy_from_slice(&hash);
    let len = (bytes.len() - HEADER_SIZE) as u64;
    bytes[40..48].copy_from_slice(&len.to_le_bytes());
}

// --- scripts ---------------------------------------------------------------

#[test]
fn self_import_is_an_error_not_a_stack_overflow() {
    let dir = TempDir::new().unwrap();
    let err = parse(&dir, "self.rhai", b"load(\"self.rhai\");").unwrap_err();
    assert!(err.contains("Circular import"), "{err}");
}

#[test]
fn mutual_import_is_an_error_not_a_stack_overflow() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("pong.rhai"), "load(\"ping.rhai\");").unwrap();
    let err = parse(&dir, "ping.rhai", b"load(\"pong.rhai\");").unwrap_err();
    assert!(err.contains("Circular import"), "{err}");
}

#[test]
fn load_cannot_reach_outside_the_scripts_directory() {
    let outer = TempDir::new().unwrap();
    fs::write(outer.path().join("secret.rhai"), "let leaked = 1;").unwrap();
    let inner = outer.path().join("profiles");
    fs::create_dir(&inner).unwrap();
    for path in ["../secret.rhai", "/etc/hostname", "a/../../secret.rhai"] {
        let script = inner.join("main.rhai");
        fs::write(&script, format!("load(\"{path}\");")).unwrap();
        let err = Parser::new().parse_script(&script).unwrap_err().to_string();
        assert!(err.contains("stay inside"), "{path}: {err}");
    }
    // Subdirectories are fine.
    fs::create_dir(inner.join("lib")).unwrap();
    fs::write(inner.join("lib/ok.rhai"), "").unwrap();
    let script = inner.join("main.rhai");
    fs::write(&script, "load(\"lib/ok.rhai\"); load(\"./lib/ok.rhai\");").unwrap();
    Parser::new().parse_script(&script).unwrap();
}

#[test]
fn import_chain_depth_is_bounded() {
    let dir = TempDir::new().unwrap();
    for i in 0..100 {
        let next = format!("load(\"c{}.rhai\");", i + 1);
        fs::write(dir.path().join(format!("c{i}.rhai")), next).unwrap();
    }
    fs::write(dir.path().join("c100.rhai"), "").unwrap();
    let err = parse(&dir, "main.rhai", b"load(\"c0.rhai\");").unwrap_err();
    assert!(err.contains("deep"), "{err}");
}

#[test]
fn the_same_file_imported_twice_in_a_row_is_not_a_cycle() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("common.rhai"), "").unwrap();
    let script = b"load(\"common.rhai\");\nload(\"common.rhai\");\n";
    parse(&dir, "main.rhai", script).unwrap();
}

#[test]
fn endless_loops_and_runaway_recursion_stop_with_an_error() {
    let dir = TempDir::new().unwrap();
    for (name, src) in [
        ("w.rhai", "while true {}"),
        ("l.rhai", "let i = 0; loop { i += 1; }"),
        ("r.rhai", "fn f(n) { f(n + 1) }\nf(0);"),
        (
            "p.rhai",
            &format!("let x = {}1{};", "(".repeat(5000), ")".repeat(5000)),
        ),
    ] {
        assert!(parse(&dir, name, src.as_bytes()).is_err(), "{name}");
    }
}

#[test]
fn a_large_legitimate_config_is_not_cut_off_by_the_operation_limit() {
    // 20 layers x 50 keys plus 8 devices: well over what the old 10 000
    // operation limit allowed (about 2 500 mappings).
    let keys: Vec<String> = ('A'..='Z')
        .map(|c| format!("VK_{c}"))
        .chain((1..=24).map(|n| format!("VK_F{n}")))
        .collect();
    let mut src = String::new();
    for device in 0..8 {
        src.push_str(&format!("device_start(\"rfz{device}*\");\n"));
        for layer in 0..60 {
            src.push_str(&format!("when_start(\"MD_{layer:02X}\");\n"));
            for (i, key) in keys.iter().enumerate() {
                src.push_str(&format!(
                    "map(\"{key}\", \"{}\");\n",
                    keys[(i + 1) % keys.len()]
                ));
            }
            src.push_str("when_end();\n");
        }
        src.push_str("device_end();\n");
    }
    let dir = TempDir::new().unwrap();
    parse(&dir, "big.rhai", src.as_bytes()).unwrap();
}

#[test]
fn byte_order_mark_and_crlf_are_accepted() {
    let dir = TempDir::new().unwrap();
    let mut bom = vec![0xEF, 0xBB, 0xBF];
    bom.extend_from_slice(SIMPLE.as_bytes());
    parse(&dir, "bom.rhai", &bom).unwrap();
    parse(&dir, "crlf.rhai", SIMPLE.replace('\n', "\r\n").as_bytes()).unwrap();
}

#[test]
fn unreadable_scripts_say_why_instead_of_not_found() {
    let dir = TempDir::new().unwrap();
    let err = parse(&dir, "bad.rhai", b"map(\"VK_A\", \"VK_B\"); // \xff\xfe").unwrap_err();
    assert!(err.contains("UTF-8"), "{err}");

    fs::create_dir(dir.path().join("dir.rhai")).unwrap();
    let err = Parser::new()
        .parse_script(&dir.path().join("dir.rhai"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("not a regular file"), "{err}");

    let err = Parser::new()
        .parse_script(&dir.path().join("missing.rhai"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("not found"), "{err}");
}

#[test]
fn oversized_script_is_refused_without_being_read() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("huge.rhai");
    let file = fs::File::create(&path).unwrap();
    file.set_len(keyrx_core::parser::limits::MAX_SOURCE_BYTES + 1)
        .unwrap();
    let err = Parser::new().parse_script(&path).unwrap_err().to_string();
    assert!(err.contains("limit"), "{err}");
}

#[test]
fn garbage_and_empty_scripts_do_not_panic() {
    let dir = TempDir::new().unwrap();
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let _ = parse(&dir, "empty.rhai", b"");
    for round in 0..200 {
        let len = (next() % 300) as usize;
        let bytes: Vec<u8> = (0..len).map(|_| next() as u8).collect();
        let _ = parse(&dir, &format!("g{round}.rhai"), &bytes);
    }
}

// --- .krx ------------------------------------------------------------------

#[test]
fn every_truncation_of_a_krx_is_rejected() {
    let good = compiled_simple();
    assert!(deserialize(&good).is_ok());
    for len in 0..good.len() {
        assert!(deserialize(&good[..len]).is_err(), "length {len}");
        let mut sealed = good[..len].to_vec();
        if len >= HEADER_SIZE {
            reseal(&mut sealed);
            // Whatever the verdict, it must not panic.
            let _ = deserialize(&sealed);
        }
    }
}

#[test]
fn random_bit_flips_never_panic() {
    let good = compiled_simple();
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..4000 {
        let mut bytes = good.clone();
        for _ in 0..=(next() % 3) {
            let bit = (next() as usize) % (bytes.len() * 8);
            bytes[bit / 8] ^= 1 << (bit % 8);
        }
        // Raw: integrity checks must catch it.
        let raw = deserialize(&bytes);
        if bytes != good {
            assert!(raw.is_err());
        }
        // Resealed: reaches rkyv's structural validation; if it is still a
        // valid archive, turning it into an owned config must not panic.
        reseal(&mut bytes);
        if let Ok(archived) = deserialize(&bytes) {
            let _: keyrx_core::config::ConfigRoot =
                match rkyv::Deserialize::deserialize(archived, &mut rkyv::Infallible) {
                    Ok(config) => config,
                    Err(never) => match never {},
                };
        }
    }
}

#[test]
fn wrong_version_and_magic_tell_the_user_to_recompile() {
    let mut good = compiled_simple();
    good[4..8].copy_from_slice(&2u32.to_le_bytes());
    let err = deserialize(&good).err().unwrap();
    assert!(matches!(err, DeserializeError::VersionMismatch { .. }));
    assert!(err.to_string().contains("recompile"), "{err}");

    let mut good = compiled_simple();
    good[0] = 0;
    let err = deserialize(&good).err().unwrap();
    assert!(matches!(err, DeserializeError::InvalidMagic { .. }));
    assert!(err.to_string().contains("keyrx_compiler compile"), "{err}");
}

#[test]
fn oversized_krx_is_refused_from_its_size_alone() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("huge.krx");
    fs::File::create(&path)
        .unwrap()
        .set_len(KRX_MAX_BYTES + 1)
        .unwrap();
    let err = read_krx(&path).unwrap_err().to_string();
    assert!(err.contains("limit"), "{err}");
    assert!(read_krx(Path::new("/definitely/not/here.krx")).is_err());
    assert!(read_krx(dir.path()).is_err());
}
