//! The SFTP version 3 requests over an SSH channel (ADR 0047).
//!
//! The source is draft-ietf-secsh-filexfer-02 and the OpenSSH file
//! `PROTOCOL`, section 4. The connector of the SFTP host uses this module.
//! A function sends one request and reads its response.

use std::fmt::Display;
use std::io::{Read, Write};

use super::{ConnectorError, ConnectorErrorKind};

/// The size of each part that a read or a write moves.
const PART: usize = 32 * 1024;
/// The largest packet that Studio accepts from a server.
const MAX_PACKET: usize = 4 * 1024 * 1024;
/// The protocol version that Studio speaks.
const VERSION: u32 = 3;

const FXP_INIT: u8 = 1;
const FXP_VERSION: u8 = 2;
const FXP_OPEN: u8 = 3;
const FXP_CLOSE: u8 = 4;
const FXP_READ: u8 = 5;
const FXP_WRITE: u8 = 6;
const FXP_LSTAT: u8 = 7;
const FXP_OPENDIR: u8 = 11;
const FXP_READDIR: u8 = 12;
const FXP_REMOVE: u8 = 13;
const FXP_MKDIR: u8 = 14;
const FXP_RENAME: u8 = 18;
const FXP_STATUS: u8 = 101;
const FXP_HANDLE: u8 = 102;
const FXP_DATA: u8 = 103;
const FXP_NAME: u8 = 104;
const FXP_ATTRS: u8 = 105;
const FXP_EXTENDED: u8 = 200;

const FX_OK: u32 = 0;
const FX_EOF: u32 = 1;
const FX_NO_SUCH_FILE: u32 = 2;
const FX_PERMISSION_DENIED: u32 = 3;
const FX_FAILURE: u32 = 4;

const OPEN_READ: u32 = 0x01;
const OPEN_WRITE: u32 = 0x02;
const OPEN_CREAT: u32 = 0x08;
const OPEN_EXCL: u32 = 0x20;

const ATTR_SIZE: u32 = 0x01;
const ATTR_UIDGID: u32 = 0x02;
const ATTR_PERMISSIONS: u32 = 0x04;
const ATTR_ACMODTIME: u32 = 0x08;
const ATTR_EXTENDED: u32 = 0x8000_0000;

const POSIX_RENAME: &str = "posix-rename@openssh.com";
const FSYNC: &str = "fsync@openssh.com";
const HARDLINK: &str = "hardlink@openssh.com";

/// The kind of a file on the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileKind {
    File,
    Folder,
    SymbolicLink,
    Other,
}

/// The attributes of a file on the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Attrs {
    /// The size in bytes, when the server gave it.
    pub size: Option<u64>,
    /// The time of the last change in seconds, when the server gave it.
    pub mtime: Option<u32>,
    /// The kind, from the permission bits.
    pub kind: FileKind,
}

/// One entry of a folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DirEntry {
    pub name: String,
    pub attrs: Attrs,
}

/// A handle of an open file or folder. The server gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Handle(Vec<u8>);

/// A byte stream that reads and writes. An SSH channel is one. The stream
/// is `Send`, so the connector can move between threads.
trait Stream: Read + Write + Send {}

impl<T: Read + Write + Send> Stream for T {}

/// An open SFTP session.
pub(crate) struct SftpWire {
    stream: Box<dyn Stream>,
    next_id: u32,
    extensions: Vec<(String, String)>,
}

/// A response: its type and the bytes after the request ID.
struct Response {
    kind: u8,
    data: Vec<u8>,
}

impl SftpWire {
    /// Open a channel on `session`, start the subsystem `sftp` and do the
    /// version exchange.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::Unavailable`] when the channel or the
    /// subsystem fails. Gives [`ConnectorErrorKind::Io`] when the server
    /// does not speak version 3.
    pub(crate) fn open(session: &ssh2::Session) -> Result<Self, ConnectorError> {
        let mut channel = session.channel_session().map_err(channel_error)?;
        channel.subsystem("sftp").map_err(channel_error)?;
        Self::from_stream(Box::new(channel))
    }

    /// Do the version exchange on `stream`.
    fn from_stream(stream: Box<dyn Stream>) -> Result<Self, ConnectorError> {
        let mut wire = Self {
            stream,
            next_id: 1,
            extensions: Vec::new(),
        };
        wire.send(FXP_INIT, &VERSION.to_be_bytes())?;
        let (kind, data) = wire.receive()?;
        if kind != FXP_VERSION {
            return Err(protocol(format!("expected a version, got type {kind}")));
        }
        let (version, extensions) = parse_version(&data)?;
        if version != VERSION {
            return Err(protocol(format!("the server speaks version {version}")));
        }
        wire.extensions = extensions;
        Ok(wire)
    }

    /// The extension names and versions that the server gave.
    #[cfg(test)]
    pub(crate) fn extensions(&self) -> &[(String, String)] {
        &self.extensions
    }

    /// Tell if the server gave the extension `name`.
    pub(crate) fn has(&self, name: &str) -> bool {
        self.extensions.iter().any(|(n, _)| n == name)
    }

    /// The attributes of a path. A symbolic link is not followed. Gives
    /// `None` when the path does not exist.
    pub(crate) fn stat(&mut self, path: &str) -> Result<Option<Attrs>, ConnectorError> {
        let mut body = Vec::new();
        put_string(&mut body, path.as_bytes());
        let response = self.call(FXP_LSTAT, &body)?;
        match response.kind {
            FXP_ATTRS => Ok(Some(Reader::new(&response.data).attrs()?)),
            FXP_STATUS if status_code(&response)? == FX_NO_SUCH_FILE => Ok(None),
            _ => Err(failure(&response, false)),
        }
    }

    /// Open a file to read it.
    pub(crate) fn open_read(&mut self, path: &str) -> Result<Handle, ConnectorError> {
        self.open_file(path, OPEN_READ, false)
    }

    /// Make a new file and open it for writing.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::Conflict`] when the file exists.
    pub(crate) fn open_create(&mut self, path: &str) -> Result<Handle, ConnectorError> {
        self.open_file(path, OPEN_WRITE | OPEN_CREAT | OPEN_EXCL, true)
    }

    fn open_file(
        &mut self,
        path: &str,
        flags: u32,
        conflict: bool,
    ) -> Result<Handle, ConnectorError> {
        let mut body = Vec::new();
        put_string(&mut body, path.as_bytes());
        body.extend_from_slice(&flags.to_be_bytes());
        // No attributes.
        body.extend_from_slice(&0u32.to_be_bytes());
        let response = self.call(FXP_OPEN, &body)?;
        handle_of(&response, conflict)
    }

    /// Read `len` bytes from `offset`. The function asks for parts of
    /// 32 KiB. It gives fewer bytes at the end of the file, and an empty
    /// result when `offset` is at the end.
    pub(crate) fn read(
        &mut self,
        handle: &Handle,
        offset: u64,
        len: usize,
    ) -> Result<Vec<u8>, ConnectorError> {
        let mut out = Vec::new();
        while out.len() < len {
            let want = (len - out.len()).min(PART);
            let mut body = Vec::new();
            put_string(&mut body, &handle.0);
            body.extend_from_slice(&(offset + out.len() as u64).to_be_bytes());
            body.extend_from_slice(&u32::try_from(want).unwrap_or(0).to_be_bytes());
            let response = self.call(FXP_READ, &body)?;
            match response.kind {
                FXP_DATA => {
                    let bytes = Reader::new(&response.data).string()?;
                    if bytes.is_empty() {
                        break;
                    }
                    if bytes.len() > want {
                        return Err(protocol("the server sent more bytes than asked"));
                    }
                    out.extend_from_slice(bytes);
                }
                FXP_STATUS if status_code(&response)? == FX_EOF => break,
                _ => return Err(failure(&response, false)),
            }
        }
        Ok(out)
    }

    /// Write `bytes` at `offset`, in parts of 32 KiB.
    pub(crate) fn write(
        &mut self,
        handle: &Handle,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), ConnectorError> {
        let mut at = offset;
        for part in bytes.chunks(PART) {
            let mut body = Vec::new();
            put_string(&mut body, &handle.0);
            body.extend_from_slice(&at.to_be_bytes());
            put_string(&mut body, part);
            let response = self.call(FXP_WRITE, &body)?;
            done(&response, false)?;
            at += part.len() as u64;
        }
        Ok(())
    }

    /// Close a handle.
    pub(crate) fn close(&mut self, handle: &Handle) -> Result<(), ConnectorError> {
        let mut body = Vec::new();
        put_string(&mut body, &handle.0);
        let response = self.call(FXP_CLOSE, &body)?;
        done(&response, false)
    }

    /// The entries of a folder, with no `.` and no `..`.
    pub(crate) fn read_dir(&mut self, path: &str) -> Result<Vec<DirEntry>, ConnectorError> {
        let mut body = Vec::new();
        put_string(&mut body, path.as_bytes());
        let response = self.call(FXP_OPENDIR, &body)?;
        let handle = handle_of(&response, false)?;
        let listed = self.list_handle(&handle);
        let closed = self.close(&handle);
        let entries = listed?;
        closed?;
        Ok(entries)
    }

    fn list_handle(&mut self, handle: &Handle) -> Result<Vec<DirEntry>, ConnectorError> {
        let mut entries = Vec::new();
        loop {
            let mut body = Vec::new();
            put_string(&mut body, &handle.0);
            let response = self.call(FXP_READDIR, &body)?;
            match response.kind {
                FXP_NAME => entries.extend(
                    parse_names(&response.data)?
                        .into_iter()
                        .filter(|entry| entry.name != "." && entry.name != ".."),
                ),
                FXP_STATUS if status_code(&response)? == FX_EOF => return Ok(entries),
                _ => return Err(failure(&response, false)),
            }
        }
    }

    /// Remove a file.
    pub(crate) fn remove(&mut self, path: &str) -> Result<(), ConnectorError> {
        let mut body = Vec::new();
        put_string(&mut body, path.as_bytes());
        let response = self.call(FXP_REMOVE, &body)?;
        done(&response, false)
    }

    /// Rename a path.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::Conflict`] when the target exists.
    pub(crate) fn rename(&mut self, from: &str, to: &str) -> Result<(), ConnectorError> {
        let mut body = Vec::new();
        put_string(&mut body, from.as_bytes());
        put_string(&mut body, to.as_bytes());
        let response = self.call(FXP_RENAME, &body)?;
        done(&response, true)
    }

    /// Make a folder.
    pub(crate) fn mkdir(&mut self, path: &str) -> Result<(), ConnectorError> {
        let mut body = Vec::new();
        put_string(&mut body, path.as_bytes());
        // No attributes.
        body.extend_from_slice(&0u32.to_be_bytes());
        let response = self.call(FXP_MKDIR, &body)?;
        done(&response, false)
    }

    /// Rename a path and replace the target.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::Unsupported`], with no request, when the
    /// server did not give `posix-rename@openssh.com`.
    pub(crate) fn posix_rename(&mut self, from: &str, to: &str) -> Result<(), ConnectorError> {
        self.require(POSIX_RENAME)?;
        let mut body = Vec::new();
        put_string(&mut body, POSIX_RENAME.as_bytes());
        put_string(&mut body, from.as_bytes());
        put_string(&mut body, to.as_bytes());
        let response = self.call(FXP_EXTENDED, &body)?;
        done(&response, false)
    }

    /// Make a hard link `to` for the file `from`.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::Unsupported`], with no request, when the
    /// server did not give `hardlink@openssh.com`. Gives
    /// [`ConnectorErrorKind::Conflict`] when the target exists.
    pub(crate) fn hardlink(&mut self, from: &str, to: &str) -> Result<(), ConnectorError> {
        self.require(HARDLINK)?;
        let mut body = Vec::new();
        put_string(&mut body, HARDLINK.as_bytes());
        put_string(&mut body, from.as_bytes());
        put_string(&mut body, to.as_bytes());
        let response = self.call(FXP_EXTENDED, &body)?;
        done(&response, true)
    }

    /// Tell the server to write the data of a file to the disk.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::Unsupported`], with no request, when the
    /// server did not give `fsync@openssh.com`.
    pub(crate) fn fsync(&mut self, handle: &Handle) -> Result<(), ConnectorError> {
        self.require(FSYNC)?;
        let mut body = Vec::new();
        put_string(&mut body, FSYNC.as_bytes());
        put_string(&mut body, &handle.0);
        let response = self.call(FXP_EXTENDED, &body)?;
        done(&response, false)
    }

    fn require(&self, name: &str) -> Result<(), ConnectorError> {
        if self.has(name) {
            Ok(())
        } else {
            Err(ConnectorError::new(
                ConnectorErrorKind::Unsupported,
                format!("the SFTP server does not give {name}"),
            ))
        }
    }

    /// Send a request with a new ID and read its response. A response with a
    /// different ID is a protocol error.
    fn call(&mut self, kind: u8, body: &[u8]) -> Result<Response, ConnectorError> {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let mut payload = Vec::with_capacity(body.len() + 4);
        payload.extend_from_slice(&id.to_be_bytes());
        payload.extend_from_slice(body);
        self.send(kind, &payload)?;
        let (response_kind, data) = self.receive()?;
        let mut reader = Reader::new(&data);
        let response_id = reader.u32()?;
        if response_id != id {
            return Err(protocol(format!(
                "the response has ID {response_id}, the request has ID {id}"
            )));
        }
        Ok(Response {
            kind: response_kind,
            data: reader.rest().to_vec(),
        })
    }

    fn send(&mut self, kind: u8, payload: &[u8]) -> Result<(), ConnectorError> {
        let length = u32::try_from(payload.len() + 1).map_err(|_| protocol("packet too large"))?;
        let mut packet = Vec::with_capacity(payload.len() + 5);
        packet.extend_from_slice(&length.to_be_bytes());
        packet.push(kind);
        packet.extend_from_slice(payload);
        self.stream.write_all(&packet).map_err(channel_error)?;
        self.stream.flush().map_err(channel_error)
    }

    /// Read one packet. Gives its type and the bytes after the type.
    fn receive(&mut self) -> Result<(u8, Vec<u8>), ConnectorError> {
        let mut length = [0u8; 4];
        self.stream.read_exact(&mut length).map_err(channel_error)?;
        let length = u32::from_be_bytes(length) as usize;
        if length == 0 || length > MAX_PACKET {
            return Err(protocol(format!("packet length {length}")));
        }
        let mut packet = vec![0u8; length];
        self.stream.read_exact(&mut packet).map_err(channel_error)?;
        let kind = packet.remove(0);
        Ok((kind, packet))
    }
}

/// An error of the channel.
fn channel_error(error: impl Display) -> ConnectorError {
    ConnectorError::new(
        ConnectorErrorKind::Unavailable,
        format!("SFTP channel error: {error}"),
    )
}

/// An error in the data that the server sent.
fn protocol(text: impl Display) -> ConnectorError {
    ConnectorError::new(
        ConnectorErrorKind::Io,
        format!("SFTP protocol error: {text}"),
    )
}

/// The code of a status response.
fn status_code(response: &Response) -> Result<u32, ConnectorError> {
    Ok(parse_status(&response.data)?.0)
}

/// The code and the message of a status response.
fn parse_status(data: &[u8]) -> Result<(u32, String), ConnectorError> {
    let mut reader = Reader::new(data);
    let code = reader.u32()?;
    let message = reader
        .string()
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .unwrap_or_default();
    Ok((code, message))
}

/// The error for a response that is not the answer that the caller wants.
/// With `conflict`, the status `SSH_FX_FAILURE` means that the target
/// exists.
fn failure(response: &Response, conflict: bool) -> ConnectorError {
    if response.kind != FXP_STATUS {
        return protocol(format!("unexpected response type {}", response.kind));
    }
    let (code, message) = match parse_status(&response.data) {
        Ok(status) => status,
        Err(error) => return error,
    };
    match code {
        FX_OK => protocol("unexpected status OK"),
        FX_NO_SUCH_FILE => ConnectorError::new(ConnectorErrorKind::NotFound, message),
        FX_PERMISSION_DENIED => ConnectorError::new(ConnectorErrorKind::Unauthorized, message),
        FX_FAILURE if conflict => ConnectorError::new(ConnectorErrorKind::Conflict, message),
        _ => ConnectorError::new(
            ConnectorErrorKind::Io,
            format!("SFTP status {code}: {message}"),
        ),
    }
}

/// Accept a status response with the code OK.
fn done(response: &Response, conflict: bool) -> Result<(), ConnectorError> {
    if response.kind == FXP_STATUS && status_code(response)? == FX_OK {
        Ok(())
    } else {
        Err(failure(response, conflict))
    }
}

/// Accept a handle response.
fn handle_of(response: &Response, conflict: bool) -> Result<Handle, ConnectorError> {
    if response.kind == FXP_HANDLE {
        Ok(Handle(Reader::new(&response.data).string()?.to_vec()))
    } else {
        Err(failure(response, conflict))
    }
}

/// Add a string with its length.
fn put_string(out: &mut Vec<u8>, bytes: &[u8]) {
    // A string is shorter than 4 GiB. The packet limit is smaller.
    out.extend_from_slice(&u32::try_from(bytes.len()).unwrap_or(u32::MAX).to_be_bytes());
    out.extend_from_slice(bytes);
}

/// The data of a `SSH_FXP_VERSION` packet: the version and the extension
/// pairs.
fn parse_version(data: &[u8]) -> Result<(u32, Vec<(String, String)>), ConnectorError> {
    let mut reader = Reader::new(data);
    let version = reader.u32()?;
    let mut extensions = Vec::new();
    while !reader.rest().is_empty() {
        let name = String::from_utf8_lossy(reader.string()?).into_owned();
        let value = String::from_utf8_lossy(reader.string()?).into_owned();
        extensions.push((name, value));
    }
    Ok((version, extensions))
}

/// The entries of a `SSH_FXP_NAME` response, after the request ID.
fn parse_names(data: &[u8]) -> Result<Vec<DirEntry>, ConnectorError> {
    let mut reader = Reader::new(data);
    let count = reader.u32()?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let name = String::from_utf8_lossy(reader.string()?).into_owned();
        // The long name is for a person to read. Studio does not use it.
        reader.string()?;
        let attrs = reader.attrs()?;
        entries.push(DirEntry { name, attrs });
    }
    Ok(entries)
}

/// A reader of the fields of a packet.
struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn rest(&self) -> &'a [u8] {
        self.bytes
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], ConnectorError> {
        if self.bytes.len() < count {
            return Err(protocol("the packet is too short"));
        }
        let (head, tail) = self.bytes.split_at(count);
        self.bytes = tail;
        Ok(head)
    }

    fn u32(&mut self) -> Result<u32, ConnectorError> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn u64(&mut self) -> Result<u64, ConnectorError> {
        let high = u64::from(self.u32()?);
        let low = u64::from(self.u32()?);
        Ok(high << 32 | low)
    }

    fn string(&mut self) -> Result<&'a [u8], ConnectorError> {
        let length = self.u32()? as usize;
        self.take(length)
    }

    fn attrs(&mut self) -> Result<Attrs, ConnectorError> {
        let flags = self.u32()?;
        let size = if flags & ATTR_SIZE != 0 {
            Some(self.u64()?)
        } else {
            None
        };
        if flags & ATTR_UIDGID != 0 {
            self.take(8)?;
        }
        let permissions = if flags & ATTR_PERMISSIONS != 0 {
            Some(self.u32()?)
        } else {
            None
        };
        let mtime = if flags & ATTR_ACMODTIME != 0 {
            // The access time, then the modification time.
            self.take(4)?;
            Some(self.u32()?)
        } else {
            None
        };
        if flags & ATTR_EXTENDED != 0 {
            for _ in 0..self.u32()? {
                self.string()?;
                self.string()?;
            }
        }
        let kind = match permissions.map(|mode| mode & 0o170_000) {
            Some(0o100_000) => FileKind::File,
            Some(0o040_000) => FileKind::Folder,
            Some(0o120_000) => FileKind::SymbolicLink,
            _ => FileKind::Other,
        };
        Ok(Attrs { size, mtime, kind })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::sshd::TestSshd;
    use std::fs;
    use std::io::Cursor;
    use std::net::{Ipv4Addr, TcpStream};
    use std::path::Path;

    /// A fake stream: the server bytes to read, and the bytes written.
    struct Script {
        input: Cursor<Vec<u8>>,
        output: Vec<u8>,
    }

    impl Read for Script {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.input.read(buf)
        }
    }

    impl Write for Script {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.output.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn packet(kind: u8, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&u32::try_from(payload.len() + 1).unwrap().to_be_bytes());
        out.push(kind);
        out.extend_from_slice(payload);
        out
    }

    fn version_packet(extensions: &[(&str, &str)]) -> Vec<u8> {
        let mut payload = 3u32.to_be_bytes().to_vec();
        for (name, value) in extensions {
            put_string(&mut payload, name.as_bytes());
            put_string(&mut payload, value.as_bytes());
        }
        packet(FXP_VERSION, &payload)
    }

    fn attrs_bytes(size: u64, mode: u32) -> Vec<u8> {
        let mut out = (ATTR_SIZE | ATTR_PERMISSIONS | ATTR_ACMODTIME)
            .to_be_bytes()
            .to_vec();
        out.extend_from_slice(&size.to_be_bytes());
        out.extend_from_slice(&mode.to_be_bytes());
        out.extend_from_slice(&1u32.to_be_bytes());
        out.extend_from_slice(&2u32.to_be_bytes());
        out
    }

    fn status_packet(id: u32, code: u32, message: &str) -> Vec<u8> {
        let mut payload = id.to_be_bytes().to_vec();
        payload.extend_from_slice(&code.to_be_bytes());
        put_string(&mut payload, message.as_bytes());
        put_string(&mut payload, b"");
        packet(FXP_STATUS, &payload)
    }

    fn scripted(server: &[u8]) -> Result<SftpWire, ConnectorError> {
        SftpWire::from_stream(Box::new(Script {
            input: Cursor::new(server.to_vec()),
            output: Vec::new(),
        }))
    }

    #[test]
    fn decodes_a_version_with_two_extensions() {
        let wire = scripted(&version_packet(&[
            ("posix-rename@openssh.com", "1"),
            ("fsync@openssh.com", "1"),
        ]))
        .expect("a session");
        assert_eq!(
            wire.extensions(),
            [
                ("posix-rename@openssh.com".to_owned(), "1".to_owned()),
                ("fsync@openssh.com".to_owned(), "1".to_owned()),
            ]
        );
        assert!(wire.has("fsync@openssh.com"));
        assert!(!wire.has("hardlink@openssh.com"));
    }

    #[test]
    fn refuses_another_version() {
        let server = packet(FXP_VERSION, &6u32.to_be_bytes());
        let error = scripted(&server).map(|_| ()).expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Io);
    }

    #[test]
    fn decodes_a_name_packet_with_two_entries() {
        let mut data = 2u32.to_be_bytes().to_vec();
        for (name, size, mode) in [("a.mp3", 7u64, 0o100_644u32), ("sub", 0, 0o040_755)] {
            put_string(&mut data, name.as_bytes());
            put_string(&mut data, b"long name");
            data.extend_from_slice(&attrs_bytes(size, mode));
        }
        let entries = parse_names(&data).expect("entries");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "a.mp3");
        assert_eq!(entries[0].attrs.size, Some(7));
        assert_eq!(entries[0].attrs.mtime, Some(2));
        assert_eq!(entries[0].attrs.kind, FileKind::File);
        assert_eq!(entries[1].name, "sub");
        assert_eq!(entries[1].attrs.kind, FileKind::Folder);
    }

    #[test]
    fn a_short_packet_is_an_error() {
        let error = parse_names(&[0, 0, 0, 1, 0, 0]).expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Io);
    }

    #[test]
    fn a_response_with_another_id_is_an_error() {
        let mut server = version_packet(&[]);
        server.extend_from_slice(&status_packet(99, FX_OK, ""));
        let mut wire = scripted(&server).expect("a session");
        let error = wire.remove("/x").expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Io);
        assert!(error.to_string().contains("ID 99"));
    }

    #[test]
    fn an_extension_without_the_server_name_sends_no_request() {
        let mut wire = scripted(&version_packet(&[])).expect("a session");
        let handle = Handle(vec![1]);
        for result in [
            wire.posix_rename("/a", "/b"),
            wire.hardlink("/a", "/b"),
            wire.fsync(&handle),
        ] {
            assert_eq!(
                result.expect_err("an error").kind(),
                ConnectorErrorKind::Unsupported
            );
        }
    }

    #[test]
    fn maps_the_status_codes() {
        let response = |code| Response {
            kind: FXP_STATUS,
            data: status_packet(1, code, "text")[9..].to_vec(),
        };
        let kind = |code, conflict| failure(&response(code), conflict).kind();
        assert_eq!(kind(FX_NO_SUCH_FILE, false), ConnectorErrorKind::NotFound);
        assert_eq!(
            kind(FX_PERMISSION_DENIED, false),
            ConnectorErrorKind::Unauthorized
        );
        assert_eq!(kind(FX_FAILURE, true), ConnectorErrorKind::Conflict);
        assert_eq!(kind(FX_FAILURE, false), ConnectorErrorKind::Io);
        assert_eq!(kind(8, true), ConnectorErrorKind::Io);
    }

    /// A server, a session and a new folder.
    struct Setup {
        _server: TestSshd,
        wire: SftpWire,
        root: String,
    }

    fn setup() -> Setup {
        let mut server = TestSshd::start();
        let stream = TcpStream::connect((Ipv4Addr::LOCALHOST, server.port())).expect("connect");
        let mut session = ssh2::Session::new().expect("a session");
        session.set_tcp_stream(stream);
        session.handshake().expect("handshake");
        session
            .userauth_pubkey_file(&server.user(), None, &server.key_path(), None)
            .expect("login");
        let wire = SftpWire::open(&session).expect("an SFTP session");
        let root = server.root().to_str().expect("a path").to_owned();
        Setup {
            _server: server,
            wire,
            root,
        }
    }

    fn create(wire: &mut SftpWire, path: &str, bytes: &[u8]) {
        let handle = wire.open_create(path).expect("a new file");
        wire.write(&handle, 0, bytes).expect("a write");
        wire.close(&handle).expect("a close");
    }

    #[test]
    fn the_server_gives_the_openssh_extensions() {
        let setup = setup();
        for name in [POSIX_RENAME, FSYNC, HARDLINK] {
            assert!(setup.wire.has(name), "{name}");
            assert!(setup.wire.extensions().iter().any(|(n, _)| n == name));
        }
    }

    #[test]
    fn a_large_file_reads_back() {
        let Setup {
            mut wire,
            root,
            _server,
        } = setup();
        let bytes: Vec<u8> = (0..100u32 * 1024)
            .map(|i| u8::try_from(i % 251).unwrap())
            .collect();
        let path = format!("{root}/big.bin");
        let handle = wire.open_create(&path).expect("a new file");
        wire.write(&handle, 0, &bytes).expect("a write");
        wire.fsync(&handle).expect("an fsync");
        wire.close(&handle).expect("a close");
        let attrs = wire.stat(&path).expect("stat").expect("a file");
        assert_eq!(attrs.size, Some(bytes.len() as u64));
        assert_eq!(attrs.kind, FileKind::File);
        let handle = wire.open_read(&path).expect("open");
        let back = wire.read(&handle, 0, bytes.len() + 10).expect("a read");
        assert_eq!(back, bytes);
        assert_eq!(
            wire.read(&handle, bytes.len() as u64, 5).expect("a read"),
            Vec::<u8>::new()
        );
        assert_eq!(
            wire.read(&handle, 100, 20).expect("a read"),
            bytes[100..120]
        );
        wire.close(&handle).expect("a close");
        assert_eq!(fs::read(&path).expect("the file"), bytes);
    }

    #[test]
    fn open_create_on_an_existing_file_gives_conflict() {
        let Setup {
            mut wire,
            root,
            _server,
        } = setup();
        let path = format!("{root}/a.txt");
        create(&mut wire, &path, b"one");
        let error = wire.open_create(&path).expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
    }

    #[test]
    fn open_read_of_a_missing_file_gives_not_found() {
        let Setup {
            mut wire,
            root,
            _server,
        } = setup();
        let error = wire
            .open_read(&format!("{root}/missing"))
            .expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::NotFound);
    }

    #[test]
    fn hardlink_and_rename_to_an_existing_target_give_conflict() {
        let Setup {
            mut wire,
            root,
            _server,
        } = setup();
        let (a, b) = (format!("{root}/a"), format!("{root}/b"));
        create(&mut wire, &a, b"A");
        create(&mut wire, &b, b"B");
        assert_eq!(
            wire.hardlink(&a, &b).expect_err("an error").kind(),
            ConnectorErrorKind::Conflict
        );
        assert_eq!(
            wire.rename(&a, &b).expect_err("an error").kind(),
            ConnectorErrorKind::Conflict
        );
        assert_eq!(fs::read(&b).expect("b"), b"B");
        let c = format!("{root}/c");
        wire.hardlink(&a, &c).expect("a link");
        assert_eq!(fs::read(&c).expect("c"), b"A");
        let d = format!("{root}/d");
        wire.rename(&c, &d).expect("a rename");
        assert!(!Path::new(&c).exists());
        assert_eq!(fs::read(&d).expect("d"), b"A");
    }

    #[test]
    fn posix_rename_replaces_the_target() {
        let Setup {
            mut wire,
            root,
            _server,
        } = setup();
        let (a, b) = (format!("{root}/a"), format!("{root}/b"));
        create(&mut wire, &a, b"A");
        create(&mut wire, &b, b"B");
        wire.posix_rename(&a, &b).expect("a rename");
        assert!(!Path::new(&a).exists());
        assert_eq!(fs::read(&b).expect("b"), b"A");
    }

    #[test]
    fn stat_gives_none_for_a_missing_path_and_the_kinds() {
        let Setup {
            mut wire,
            root,
            _server,
        } = setup();
        assert_eq!(wire.stat(&format!("{root}/missing")).expect("stat"), None);
        let file = format!("{root}/f");
        create(&mut wire, &file, b"x");
        let link = format!("{root}/l");
        std::os::unix::fs::symlink(&file, &link).expect("a link");
        let folder = format!("{root}/d");
        wire.mkdir(&folder).expect("a folder");
        let kind =
            |wire: &mut SftpWire, path: &str| wire.stat(path).expect("stat").expect("found").kind;
        assert_eq!(kind(&mut wire, &link), FileKind::SymbolicLink);
        assert_eq!(kind(&mut wire, &folder), FileKind::Folder);
        assert_eq!(kind(&mut wire, &file), FileKind::File);
    }

    #[test]
    fn read_dir_lists_the_entries_and_remove_deletes() {
        let Setup {
            mut wire,
            root,
            _server,
        } = setup();
        create(&mut wire, &format!("{root}/one"), b"1");
        create(&mut wire, &format!("{root}/two"), b"22");
        wire.mkdir(&format!("{root}/sub")).expect("a folder");
        let mut entries = wire.read_dir(&root).expect("entries");
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["one", "sub", "two"]);
        assert_eq!(entries[2].attrs.size, Some(2));
        assert_eq!(entries[1].attrs.kind, FileKind::Folder);
        wire.remove(&format!("{root}/one")).expect("a remove");
        assert_eq!(wire.read_dir(&root).expect("entries").len(), 2);
        let error = wire
            .read_dir(&format!("{root}/missing"))
            .expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::NotFound);
    }
}
