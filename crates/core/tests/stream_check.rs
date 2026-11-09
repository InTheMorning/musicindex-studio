//! The stream check of ADR 0162 items 2 and 3, against a local server.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use studio_core::net::NetPolicy;
use studio_core::stream::{StreamCheck, StreamStatus};
use studio_core::web::Web;

/// How a route answers.
#[derive(Clone)]
enum Answer {
    /// A status, a content type, and a body with a `Content-Length`.
    File(u16, &'static str, &'static str),
    /// A 200 answer with no `Content-Length`: the body goes in one chunk,
    /// and the connection stays open for a time.
    Open(&'static str, Vec<u8>),
    /// The server accepts the request and sends nothing.
    Silent,
}

fn serve(routes: Vec<(&'static str, Answer)>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let routes = routes.clone();
            thread::spawn(move || {
                let mut line = String::new();
                {
                    let mut reader = BufReader::new(&mut stream);
                    reader.read_line(&mut line).unwrap();
                    let mut header = String::new();
                    while reader.read_line(&mut header).is_ok() && header.trim() != "" {
                        header.clear();
                    }
                }
                let path = line.split_whitespace().nth(1).unwrap_or("/").to_owned();
                let answer = routes
                    .iter()
                    .find(|(p, _)| *p == path)
                    .map(|(_, a)| a.clone());
                match answer {
                    Some(Answer::File(status, kind, body)) => {
                        let _ = write!(
                            stream,
                            "HTTP/1.1 {status} X\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        );
                    }
                    Some(Answer::Open(kind, body)) => {
                        let _ = write!(
                            stream,
                            "HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n",
                            body.len()
                        );
                        let _ = stream.write_all(&body);
                        let _ = stream.write_all(b"\r\n");
                        let _ = stream.flush();
                        thread::sleep(Duration::from_secs(3));
                    }
                    Some(Answer::Silent) => thread::sleep(Duration::from_secs(5)),
                    None => {
                        let _ = write!(
                            stream,
                            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        );
                    }
                }
            });
        }
    });
    port
}

fn check(routes: Vec<(&'static str, Answer)>, path: &str) -> StreamStatus {
    let port = serve(routes);
    Web::new().check_stream(&format!("http://127.0.0.1:{port}{path}"))
}

const LIVE_HLS: &str = "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:6\n#EXT-X-MEDIA-SEQUENCE:7\n#EXTINF:6.0,\nseg7.ts\n#EXTINF:6.0,\nseg8.ts\n";
const HLS_TYPE: &str = "application/vnd.apple.mpegurl";

#[test]
fn a_hls_playlist_with_a_segment_and_no_end_is_live() {
    let status = check(
        vec![("/a.m3u8", Answer::File(200, HLS_TYPE, LIVE_HLS))],
        "/a.m3u8",
    );
    assert!(status.live, "{status:?}");
    assert_eq!(status.server_type.as_deref(), Some(HLS_TYPE));
    assert_eq!(status.cause, None);
}

#[test]
fn a_hls_playlist_with_endlist_is_a_recording() {
    let body = "#EXTM3U\n#EXTINF:6.0,\nseg7.ts\n#EXT-X-ENDLIST\n";
    let status = check(
        vec![("/a.m3u8", Answer::File(200, HLS_TYPE, body))],
        "/a.m3u8",
    );
    assert!(!status.live);
    assert!(status.cause.unwrap().contains("#EXT-X-ENDLIST"));
}

#[test]
fn a_hls_playlist_with_no_segment_is_not_live() {
    let body = "#EXTM3U\n#EXT-X-TARGETDURATION:6\n";
    let status = check(
        vec![("/a.m3u8", Answer::File(200, HLS_TYPE, body))],
        "/a.m3u8",
    );
    assert!(!status.live);
}

#[test]
fn a_main_playlist_is_judged_by_its_first_variant() {
    let main = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nlow/v.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=2000\nhigh/v.m3u8\n";
    let routes = vec![
        (
            "/main.m3u8",
            Answer::File(200, "application/x-mpegURL", main),
        ),
        ("/low/v.m3u8", Answer::File(200, HLS_TYPE, LIVE_HLS)),
    ];
    let status = check(routes, "/main.m3u8");
    assert!(status.live, "{status:?}");
    assert_eq!(status.server_type.as_deref(), Some("application/x-mpegURL"));

    let ended = "#EXTM3U\n#EXTINF:6.0,\ns.ts\n#EXT-X-ENDLIST\n";
    let routes = vec![
        ("/main.m3u8", Answer::File(200, HLS_TYPE, main)),
        ("/low/v.m3u8", Answer::File(200, HLS_TYPE, ended)),
    ];
    assert!(!check(routes, "/main.m3u8").live);
    let routes = vec![("/main.m3u8", Answer::File(200, HLS_TYPE, main))];
    let status = check(routes, "/main.m3u8");
    assert!(!status.live);
    assert!(status.cause.unwrap().contains("404"));
}

#[test]
fn a_dash_manifest_is_live_when_dynamic_and_not_when_static() {
    let dynamic = "<?xml version=\"1.0\"?><MPD xmlns=\"urn:mpeg:dash:schema:mpd:2011\" type=\"dynamic\"><Period/></MPD>";
    let fixed = "<?xml version=\"1.0\"?><MPD xmlns=\"urn:mpeg:dash:schema:mpd:2011\" type=\"static\"><Period/></MPD>";
    let routes = vec![
        (
            "/live.mpd",
            Answer::File(200, "application/dash+xml", dynamic),
        ),
        (
            "/file.mpd",
            Answer::File(200, "application/dash+xml", fixed),
        ),
    ];
    let port = serve(routes);
    let web = Web::new();
    let live = web.check_stream(&format!("http://127.0.0.1:{port}/live.mpd"));
    assert!(live.live, "{live:?}");
    assert_eq!(live.server_type.as_deref(), Some("application/dash+xml"));
    let fixed = web.check_stream(&format!("http://127.0.0.1:{port}/file.mpd"));
    assert!(!fixed.live);
}

#[test]
fn an_audio_answer_with_no_length_is_live_and_with_a_length_is_a_file() {
    let routes = vec![
        (
            "/live",
            Answer::Open("audio/mpeg; charset=x", vec![0xFF; 4096]),
        ),
        (
            "/file.mp3",
            Answer::File(200, "audio/mpeg", "ID3 not really audio"),
        ),
    ];
    let port = serve(routes);
    let web = Web::new();
    let live = web.check_stream(&format!("http://127.0.0.1:{port}/live"));
    assert!(live.live, "{live:?}");
    assert_eq!(live.server_type.as_deref(), Some("audio/mpeg"));
    let file = web.check_stream(&format!("http://127.0.0.1:{port}/file.mp3"));
    assert!(!file.live);
    assert_eq!(file.server_type.as_deref(), Some("audio/mpeg"));
    assert!(file.cause.unwrap().contains("length"));
}

#[test]
fn a_404_and_a_refused_connection_are_not_live() {
    let status = check(Vec::new(), "/missing.m3u8");
    assert!(!status.live);
    assert!(status.cause.unwrap().contains("404"));
    let unused = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = unused.local_addr().unwrap().port();
    drop(unused);
    let status = Web::new().check_stream(&format!("http://127.0.0.1:{port}/a"));
    assert!(!status.live);
    assert!(status.cause.is_some());
    let status = Web::new().check_stream("ftp://a.example/a");
    assert!(!status.live);
}

#[test]
fn a_server_that_does_not_answer_gives_a_timeout() {
    let port = serve(vec![("/slow", Answer::Silent)]);
    let started = std::time::Instant::now();
    let status = Web::new().check_stream_within(
        &format!("http://127.0.0.1:{port}/slow"),
        Duration::from_millis(500),
        1024,
    );
    assert!(!status.live);
    assert!(status.cause.unwrap().contains("did not answer"));
    assert!(started.elapsed() < Duration::from_secs(4));
}

#[test]
fn the_policy_public_refuses_a_private_address() {
    let port = serve(vec![("/a.m3u8", Answer::File(200, HLS_TYPE, LIVE_HLS))]);
    let web = Web::with_policy(&NetPolicy::public());
    let status = web.check_stream(&format!("http://127.0.0.1:{port}/a.m3u8"));
    assert!(!status.live);
    assert!(status.cause.unwrap().contains("not a public address"));
}

#[test]
fn the_refusal_text_names_the_cause() {
    let status = StreamStatus {
        live: false,
        server_type: None,
        cause: Some("The stream server answered with status 404.".to_owned()),
    };
    assert_eq!(
        status.refusal().as_deref(),
        Some(
            "Studio cannot go live, because the stream is not live: The stream server answered with status 404."
        )
    );
}
