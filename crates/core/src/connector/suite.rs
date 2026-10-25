//! Checks that examine a connector through the `Connector` trait only
//! (ADR 0043 item 2). Each connector runs the same checks on a new empty
//! host.

#![expect(
    clippy::missing_panics_doc,
    reason = "each check panics when it fails, and that is the purpose of a check"
)]

use super::{Connector, ConnectorError, ConnectorErrorKind, Expected, Stamp};

/// Makes a connector on a new empty host.
type Make<'a> = &'a dyn Fn() -> Box<dyn Connector>;

/// Run each check.
pub fn run_all(make: Make<'_>) {
    read_of_an_absent_file_gives_none(make);
    write_then_read_gives_the_bytes_and_the_version(make);
    write_over_a_file_with_absent_is_a_conflict(make);
    write_with_an_old_version_is_a_conflict(make);
    write_with_the_current_version_replaces_the_file(make);
    write_from_gives_the_same_file_as_write(make);
    write_from_with_a_short_reader_writes_nothing(make);
    stat_gives_the_size_and_a_new_stamp_after_a_change(make);
    list_gives_each_file_in_path_sequence(make);
    delete_with_the_current_stamp_removes_the_file(make);
    delete_with_an_old_stamp_is_a_conflict(make);
    delete_of_an_absent_file_is_not_found(make);
    a_path_out_of_the_host_is_invalid(make);
    sha256_when_given_is_the_sha256_of_the_bytes(make);
    set_redirect_without_the_ability_is_unsupported(make);
}

/// The bytes of a file, or panic when `read` fails.
fn bytes_of(connector: &dyn Connector, path: &str) -> Option<Vec<u8>> {
    connector
        .read(path)
        .unwrap_or_else(|e| panic!("read of {path} must give Ok: {e}"))
        .map(|f| f.bytes)
}

/// `read` of a path with no file gives `Ok(None)`.
pub fn read_of_an_absent_file_gives_none(make: Make<'_>) {
    let connector = make();
    let read = connector
        .read("absent.txt")
        .expect("read of an absent file must give Ok(None), not an error");
    assert!(read.is_none(), "read of an absent file must give None");
}

/// `write` with `Absent` gives a version. `read` then gives the same bytes
/// and the same version.
pub fn write_then_read_gives_the_bytes_and_the_version(make: Make<'_>) {
    let connector = make();
    let version = connector
        .write("a/one.txt", b"one", &Expected::Absent)
        .expect("write with Absent on an empty host must give a version");
    let read = connector
        .read("a/one.txt")
        .expect("read after write must give Ok")
        .expect("read after write must give the file");
    assert_eq!(read.bytes, b"one", "read must give the bytes of the write");
    assert_eq!(
        read.version, version,
        "read must give the version that write gave"
    );
}

/// `write` with `Absent` on a path with a file gives `Conflict`. The file
/// does not change.
pub fn write_over_a_file_with_absent_is_a_conflict(make: Make<'_>) {
    let connector = make();
    connector
        .write("one.txt", b"one", &Expected::Absent)
        .expect("the first write with Absent must work");
    let error = connector
        .write("one.txt", b"two", &Expected::Absent)
        .expect_err("write with Absent on a path with a file must fail");
    assert_eq!(
        error.kind(),
        ConnectorErrorKind::Conflict,
        "write with Absent on a path with a file must give Conflict"
    );
    assert_eq!(
        bytes_of(&*connector, "one.txt").as_deref(),
        Some(&b"one"[..]),
        "write that gave Conflict must not change the file"
    );
}

/// `write` with the version before the last write gives `Conflict`. The
/// file does not change.
pub fn write_with_an_old_version_is_a_conflict(make: Make<'_>) {
    let connector = make();
    let v1 = connector
        .write("one.txt", b"one", &Expected::Absent)
        .expect("the first write must work");
    connector
        .write("one.txt", b"two", &Expected::Version(v1.clone()))
        .expect("write with the current version must work");
    let error = connector
        .write("one.txt", b"three", &Expected::Version(v1))
        .expect_err("write with an old version must fail");
    assert_eq!(
        error.kind(),
        ConnectorErrorKind::Conflict,
        "write with an old version must give Conflict"
    );
    assert_eq!(
        bytes_of(&*connector, "one.txt").as_deref(),
        Some(&b"two"[..]),
        "write that gave Conflict must not change the file"
    );
}

/// `write` with the current version gives a different version, and `read`
/// gives the new bytes.
pub fn write_with_the_current_version_replaces_the_file(make: Make<'_>) {
    let connector = make();
    let v1 = connector
        .write("one.txt", b"one", &Expected::Absent)
        .expect("the first write must work");
    let v2 = connector
        .write("one.txt", b"other", &Expected::Version(v1.clone()))
        .expect("write with the current version must work");
    assert_ne!(v1, v2, "write of different bytes must give a new version");
    let read = connector
        .read("one.txt")
        .expect("read must give Ok")
        .expect("read must give the file");
    assert_eq!(read.bytes, b"other", "read must give the new bytes");
    assert_eq!(read.version, v2, "read must give the new version");
}

/// `write_from` and `write` of the same bytes on two paths give the same
/// bytes on `read`.
pub fn write_from_gives_the_same_file_as_write(make: Make<'_>) {
    let connector = make();
    let bytes: Vec<u8> = (0..70_000_u32).map(|n| (n % 251) as u8).collect();
    connector
        .write("a.bin", &bytes, &Expected::Absent)
        .expect("write must work");
    connector
        .write_from(
            "b/b.bin",
            &mut bytes.as_slice(),
            bytes.len() as u64,
            &Expected::Absent,
        )
        .expect("write_from with a reader of the right length must work");
    assert_eq!(
        bytes_of(&*connector, "a.bin"),
        bytes_of(&*connector, "b/b.bin"),
        "write_from must give the same bytes as write"
    );
}

/// `write_from` with a reader that gives fewer bytes than `length` gives
/// `Io`, and `read` of the path gives `None`.
pub fn write_from_with_a_short_reader_writes_nothing(make: Make<'_>) {
    let connector = make();
    let error = connector
        .write_from("short.bin", &mut &b"abc"[..], 10, &Expected::Absent)
        .expect_err("write_from with a short reader must fail");
    assert_eq!(
        error.kind(),
        ConnectorErrorKind::Io,
        "write_from with a short reader must give Io"
    );
    assert!(
        bytes_of(&*connector, "short.bin").is_none(),
        "write_from that gave Io must leave no file for read"
    );
}

/// `stat` gives the size of the file. After a write of different bytes, the
/// stamp is different. `stat` of an absent path gives `None`.
pub fn stat_gives_the_size_and_a_new_stamp_after_a_change(make: Make<'_>) {
    let connector = make();
    assert!(
        connector
            .stat("one.txt")
            .expect("stat of an absent file must give Ok(None)")
            .is_none(),
        "stat of an absent path must give None"
    );
    let v1 = connector
        .write("one.txt", b"one", &Expected::Absent)
        .expect("the first write must work");
    let first = connector
        .stat("one.txt")
        .expect("stat must give Ok")
        .expect("stat of a file must give its facts");
    assert_eq!(first.size, 3, "stat must give the size of the file");
    connector
        .write("one.txt", b"longer bytes", &Expected::Version(v1))
        .expect("write with the current version must work");
    let second = connector
        .stat("one.txt")
        .expect("stat must give Ok")
        .expect("stat of a file must give its facts");
    assert_eq!(second.size, 12, "stat must give the new size");
    assert_ne!(
        first.stamp, second.stamp,
        "stat must give a different stamp after a write of different bytes"
    );
}

/// After writes to `b.txt`, `a/c.txt` and `a.txt`, `list` gives the three
/// paths in sorted sequence, each with its size, and `truncated` false.
pub fn list_gives_each_file_in_path_sequence(make: Make<'_>) {
    let connector = make();
    for (path, bytes) in [("b.txt", &b"bb"[..]), ("a/c.txt", b"c"), ("a.txt", b"aaa")] {
        connector
            .write(path, bytes, &Expected::Absent)
            .expect("write must work");
    }
    let list = connector.list().expect("list must give Ok");
    let files: Vec<(&str, u64)> = list
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.size))
        .collect();
    assert_eq!(
        files,
        [("a.txt", 3), ("a/c.txt", 1), ("b.txt", 2)],
        "list must give each file in the sequence of its path, with its size"
    );
    assert!(!list.truncated, "list of three files must not be truncated");
}

/// `delete` with the stamp from `stat` removes the file.
pub fn delete_with_the_current_stamp_removes_the_file(make: Make<'_>) {
    let connector = make();
    connector
        .write("one.txt", b"one", &Expected::Absent)
        .expect("write must work");
    let stamp = connector
        .stat("one.txt")
        .expect("stat must give Ok")
        .expect("stat must give the file")
        .stamp;
    connector
        .delete("one.txt", &stamp)
        .expect("delete with the current stamp must work");
    assert!(
        bytes_of(&*connector, "one.txt").is_none(),
        "delete with the current stamp must remove the file"
    );
}

/// `delete` with a stamp from before a change gives `Conflict`, and the
/// file stays.
pub fn delete_with_an_old_stamp_is_a_conflict(make: Make<'_>) {
    let connector = make();
    let v1 = connector
        .write("one.txt", b"one", &Expected::Absent)
        .expect("write must work");
    let old = connector
        .stat("one.txt")
        .expect("stat must give Ok")
        .expect("stat must give the file")
        .stamp;
    connector
        .write("one.txt", b"longer bytes", &Expected::Version(v1))
        .expect("write with the current version must work");
    let error = connector
        .delete("one.txt", &old)
        .expect_err("delete with an old stamp must fail");
    assert_eq!(
        error.kind(),
        ConnectorErrorKind::Conflict,
        "delete with an old stamp must give Conflict"
    );
    assert_eq!(
        bytes_of(&*connector, "one.txt").as_deref(),
        Some(&b"longer bytes"[..]),
        "delete that gave Conflict must leave the file"
    );
}

/// `delete` of a path with no file gives `NotFound`.
pub fn delete_of_an_absent_file_is_not_found(make: Make<'_>) {
    let connector = make();
    let error = connector
        .delete("absent.txt", &Stamp::new("0"))
        .expect_err("delete of an absent file must fail");
    assert_eq!(
        error.kind(),
        ConnectorErrorKind::NotFound,
        "delete of an absent file must give NotFound"
    );
}

/// `read`, `write`, `stat` and `delete` of `../x` and of `/x` each give
/// `InvalidPath`.
pub fn a_path_out_of_the_host_is_invalid(make: Make<'_>) {
    let connector = make();
    for path in ["../x", "/x"] {
        let kind = |result: Result<(), ConnectorError>, name: &str| {
            let error = result.expect_err(&format!("{name} of {path} must fail"));
            assert_eq!(
                error.kind(),
                ConnectorErrorKind::InvalidPath,
                "{name} of {path} must give InvalidPath"
            );
        };
        kind(connector.read(path).map(|_| ()), "read");
        kind(
            connector.write(path, b"x", &Expected::Absent).map(|_| ()),
            "write",
        );
        kind(connector.stat(path).map(|_| ()), "stat");
        kind(connector.delete(path, &Stamp::new("0")), "delete");
    }
}

/// When `sha256` gives a value, it is the lowercase hexadecimal SHA-256 of
/// the written bytes.
pub fn sha256_when_given_is_the_sha256_of_the_bytes(make: Make<'_>) {
    let connector = make();
    connector
        .write("one.txt", b"one", &Expected::Absent)
        .expect("write must work");
    let given = connector
        .sha256("one.txt")
        .expect("sha256 of a file must give Ok");
    if let Some(hex) = given {
        assert_eq!(
            hex,
            crate::digest::sha256_hex(&[b"one"]),
            "sha256 must give the lowercase hexadecimal SHA-256 of the bytes"
        );
    }
}

/// When `can_redirect` gives false, `set_redirect` gives `Unsupported`.
pub fn set_redirect_without_the_ability_is_unsupported(make: Make<'_>) {
    let connector = make();
    if connector.can_redirect() {
        return;
    }
    let error = connector
        .set_redirect("old.xml", "https://example.com/new.xml")
        .expect_err("set_redirect without the ability must fail");
    assert_eq!(
        error.kind(),
        ConnectorErrorKind::Unsupported,
        "set_redirect without the ability must give Unsupported"
    );
}
