//! R2: golden emit x86_64 + optional host roundtrip.

use base_recomp::roundtrip::{emit_x86_64_body, host_supports_x86_64_roundtrip, smoke_x86_64};
use std::fs;
use std::path::PathBuf;

fn golden(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens")
        .join(name);
    fs::read_to_string(path).expect("golden readable")
}

#[test]
fn golden_nop_ret() {
    let body = emit_x86_64_body(&[0x90, 0xC3], "nop_ret").unwrap();
    assert_eq!(body, golden("x86_64_nop_ret.s"));
}

#[test]
fn golden_mov_add_ret() {
    // mov eax,1 ; add eax,2 ; ret
    let bytes = [
        0xB8, 0x01, 0x00, 0x00, 0x00, 0x05, 0x02, 0x00, 0x00, 0x00, 0xC3,
    ];
    let body = emit_x86_64_body(&bytes, "add3").unwrap();
    assert_eq!(body, golden("x86_64_add3.s"));
}

#[test]
fn host_roundtrip_add3() {
    if !host_supports_x86_64_roundtrip() {
        eprintln!("skip: host roundtrip tools/arch unavailable");
        return;
    }
    let bytes = [
        0xB8, 0x01, 0x00, 0x00, 0x00, 0x05, 0x02, 0x00, 0x00, 0x00, 0xC3,
    ];
    let dir = tempfile_dir("r2_add3");
    smoke_x86_64(&bytes, "add3", 3, &dir).expect("roundtrip add3");
}

#[test]
fn host_roundtrip_ret_zero() {
    if !host_supports_x86_64_roundtrip() {
        eprintln!("skip: host roundtrip tools/arch unavailable");
        return;
    }
    // xor eax,eax is unsupported — use mov 0 ; ret
    let bytes = [0xB8, 0x00, 0x00, 0x00, 0x00, 0xC3];
    let dir = tempfile_dir("r2_zero");
    smoke_x86_64(&bytes, "z", 0, &dir).expect("roundtrip zero");
}

fn tempfile_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "base-recomp-{}-{}",
        label,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
