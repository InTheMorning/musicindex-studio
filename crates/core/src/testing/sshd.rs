//! An OpenSSH server for tests. It runs as a normal user.
//!
//! It uses the `sshd` program of the computer, on a free loopback port.
//! It has a temporary configuration and temporary keys. Only key login
//! works. It sends no request to another computer.

use std::fs;
use std::io::Read;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// The places where `sshd` can be, in the order of the search.
const SSHD_PATHS: [&str; 3] = ["/usr/sbin/sshd", "/usr/bin/sshd", "/usr/local/sbin/sshd"];

/// The passphrase of the encrypted user key.
pub const KEY_PASSPHRASE: &str = "test passphrase";

/// A number that makes each folder name different in this process.
static FOLDERS: AtomicUsize = AtomicUsize::new(0);

/// An `sshd` child process with its folder. Dropping the value stops the
/// process and removes the folder.
#[derive(Debug)]
pub struct TestSshd {
    folder: PathBuf,
    sshd: PathBuf,
    port: u16,
    child: Option<Child>,
    roots: usize,
}

impl TestSshd {
    /// Start `sshd` with a new host key and two new user keys.
    ///
    /// # Panics
    ///
    /// Panics when `sshd` is not on the computer, when a step of the setup
    /// fails, or when `sshd` does not accept connections in 10 seconds.
    #[must_use]
    pub fn start() -> Self {
        let sshd = SSHD_PATHS
            .iter()
            .map(PathBuf::from)
            .find(|path| path.exists())
            .expect("the SFTP tests need the OpenSSH sshd program. Install openssh-server.");
        let folder = std::env::temp_dir().join(format!(
            "studio-sshd-{}-{}",
            std::process::id(),
            FOLDERS.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&folder).expect("a test folder");
        keygen(&folder.join("host_key"), "");
        keygen(&folder.join("user_key"), "");
        keygen(&folder.join("encrypted_key"), KEY_PASSPHRASE);
        let mut keys = String::new();
        for name in ["user_key.pub", "encrypted_key.pub"] {
            keys.push_str(&fs::read_to_string(folder.join(name)).expect("a public key"));
        }
        fs::write(folder.join("authorized_keys"), keys).expect("authorized_keys");
        let mut server = Self {
            folder,
            sshd,
            port: 0,
            child: None,
            roots: 0,
        };
        // Bind to port 0, read the port and close the listener. Another
        // process can take the port before sshd binds it. Then sshd stops,
        // and the harness tries again with a new port.
        let mut last = String::new();
        for _ in 0..START_ATTEMPTS {
            server.port = free_port();
            server.write_config();
            match server.spawn() {
                Ok(()) => return server,
                Err(text) => last = text,
            }
        }
        panic!("sshd did not accept connections: {last}");
    }

    /// The loopback port of the server.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The name of the user that runs the test. It is the only user that
    /// can log in.
    ///
    /// # Panics
    ///
    /// Panics when the name is not available.
    #[must_use]
    pub fn user(&self) -> String {
        if let Ok(user) = std::env::var("USER")
            && !user.is_empty()
        {
            return user;
        }
        let output = Command::new("whoami").output().expect("whoami");
        String::from_utf8(output.stdout)
            .expect("a user name")
            .trim()
            .to_owned()
    }

    /// The private user key, with no passphrase.
    #[must_use]
    pub fn key_path(&self) -> PathBuf {
        self.folder.join("user_key")
    }

    /// The private user key with the passphrase [`KEY_PASSPHRASE`].
    #[must_use]
    pub fn encrypted_key_path(&self) -> PathBuf {
        self.folder.join("encrypted_key")
    }

    /// A new empty folder for the files of a test.
    ///
    /// # Panics
    ///
    /// Panics when the folder cannot be made.
    pub fn root(&mut self) -> PathBuf {
        self.roots += 1;
        let root = self.folder.join(format!("root-{}", self.roots));
        fs::create_dir_all(&root).expect("a root folder");
        root
    }

    /// The fingerprint of the host key in the OpenSSH form:
    /// `SHA256:` and the base64 of the hash, with no padding.
    ///
    /// # Panics
    ///
    /// Panics when `ssh-keygen` fails.
    #[must_use]
    pub fn host_key_fingerprint(&self) -> String {
        let output = Command::new("ssh-keygen")
            .args(["-l", "-E", "sha256", "-f"])
            .arg(self.folder.join("host_key.pub"))
            .output()
            .expect("ssh-keygen");
        assert!(output.status.success(), "ssh-keygen -l failed");
        String::from_utf8(output.stdout)
            .expect("text")
            .split_whitespace()
            .nth(1)
            .expect("a fingerprint")
            .to_owned()
    }

    /// Replace the host key with a new key. The server starts again on the
    /// same port.
    ///
    /// # Panics
    ///
    /// Panics when the new key or the new process fails.
    pub fn with_host_key_change(&mut self) {
        self.stop();
        let host = self.folder.join("host_key");
        fs::remove_file(&host).expect("the old host key");
        fs::remove_file(self.folder.join("host_key.pub")).expect("the old public host key");
        keygen(&host, "");
        if let Err(text) = self.spawn() {
            panic!("sshd did not accept connections: {text}");
        }
    }

    /// Write `sshd_config` with the current port.
    fn write_config(&self) {
        let config = format!(
            "ListenAddress 127.0.0.1\n\
             Port {port}\n\
             HostKey {host}\n\
             AuthorizedKeysFile {keys}\n\
             PasswordAuthentication no\n\
             KbdInteractiveAuthentication no\n\
             PubkeyAuthentication yes\n\
             UsePAM no\n\
             StrictModes no\n\
             PidFile none\n\
             Subsystem sftp internal-sftp\n\
             LogLevel ERROR\n",
            port = self.port,
            host = self.folder.join("host_key").display(),
            keys = self.folder.join("authorized_keys").display(),
        );
        fs::write(self.folder.join("sshd_config"), config).expect("sshd_config");
    }

    /// Start the child process and wait until the port accepts connections.
    /// Gives the error output of `sshd` when it stops or does not accept
    /// connections in 10 seconds.
    fn spawn(&mut self) -> Result<(), String> {
        let errors = self.folder.join("sshd.log");
        let log = fs::File::create(&errors).expect("the error output file");
        let mut child = Command::new(&self.sshd)
            .args(["-D", "-e", "-f"])
            .arg(self.folder.join("sshd_config"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(log)
            .spawn()
            .expect("the sshd process");
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, self.port));
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            if TcpStream::connect_timeout(&address, Duration::from_millis(200)).is_ok() {
                break;
            }
            let exited = child.try_wait().expect("the sshd state").is_some();
            if exited || Instant::now() > end {
                let _ = child.kill();
                let _ = child.wait();
                let mut text = String::new();
                let _ = fs::File::open(&errors).and_then(|mut file| file.read_to_string(&mut text));
                return Err(text);
            }
            thread::sleep(Duration::from_millis(50));
        }
        self.child = Some(child);
        Ok(())
    }

    /// Stop the child process and wait for its end.
    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// The number of ports that `TestSshd::start` tries.
const START_ATTEMPTS: usize = 5;

/// A loopback port that was free a moment ago.
fn free_port() -> u16 {
    TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("a loopback port")
        .local_addr()
        .expect("a local address")
        .port()
}

impl Drop for TestSshd {
    fn drop(&mut self) {
        self.stop();
        let _ = fs::remove_dir_all(&self.folder);
    }
}

/// Make an Ed25519 key pair at `file`, with `passphrase` (can be empty).
fn keygen(file: &Path, passphrase: &str) {
    let status = Command::new("ssh-keygen")
        .args(["-t", "ed25519", "-N", passphrase, "-q", "-f"])
        .arg(file)
        .status()
        .expect("ssh-keygen");
    assert!(status.success(), "ssh-keygen failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD_NO_PAD;
    use ssh2::{HashType, Session};

    /// A session that has done the handshake with the server.
    fn connect(server: &TestSshd) -> Session {
        let stream = TcpStream::connect((Ipv4Addr::LOCALHOST, server.port())).expect("connect");
        let mut session = Session::new().expect("a session");
        session.set_tcp_stream(stream);
        session.handshake().expect("handshake");
        session
    }

    /// The fingerprint that the client sees.
    fn seen_fingerprint(session: &Session) -> String {
        let hash = session.host_key_hash(HashType::Sha256).expect("a hash");
        format!("SHA256:{}", STANDARD_NO_PAD.encode(hash))
    }

    #[test]
    fn fingerprint_matches_the_client() {
        let server = TestSshd::start();
        let session = connect(&server);
        assert_eq!(seen_fingerprint(&session), server.host_key_fingerprint());
    }

    #[test]
    fn plain_key_logs_in() {
        let server = TestSshd::start();
        let session = connect(&server);
        session
            .userauth_pubkey_file(&server.user(), None, &server.key_path(), None)
            .expect("login");
        assert!(session.authenticated());
    }

    #[test]
    fn encrypted_key_logs_in() {
        let server = TestSshd::start();
        let session = connect(&server);
        session
            .userauth_pubkey_file(
                &server.user(),
                None,
                &server.encrypted_key_path(),
                Some(KEY_PASSPHRASE),
            )
            .expect("login");
        assert!(session.authenticated());
    }

    #[test]
    fn encrypted_key_needs_its_passphrase() {
        let server = TestSshd::start();
        let session = connect(&server);
        let result =
            session.userauth_pubkey_file(&server.user(), None, &server.encrypted_key_path(), None);
        assert!(result.is_err());
    }

    #[test]
    fn host_key_change_gives_a_new_fingerprint() {
        let mut server = TestSshd::start();
        let before = server.host_key_fingerprint();
        server.with_host_key_change();
        let after = server.host_key_fingerprint();
        assert_ne!(before, after);
        let session = connect(&server);
        assert_eq!(seen_fingerprint(&session), after);
    }

    #[test]
    fn drop_stops_the_process_and_removes_the_folder() {
        let server = TestSshd::start();
        let folder = server.folder.clone();
        let pid = server.child.as_ref().expect("a child").id();
        drop(server);
        assert!(!folder.exists());
        // The process is waited for, so its entry is gone.
        assert!(!Path::new(&format!("/proc/{pid}")).exists());
    }

    #[test]
    fn roots_are_new_and_empty() {
        let mut server = TestSshd::start();
        let first = server.root();
        let second = server.root();
        assert_ne!(first, second);
        assert_eq!(fs::read_dir(&first).expect("a folder").count(), 0);
    }
}
