use std::collections::VecDeque;
use std::fmt;
use std::fs::{self, File};
use std::hash::Hasher;
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SyncClient {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncRequest {
    pub path: String,
    pub hash: String,
}

#[derive(Debug)]
pub enum SyncError {
    InvalidBaseUrl(String),
    Connection(std::io::Error),
    Io(std::io::Error),
    Protocol(String),
    Server(u16, String),
    InvalidPath(String),
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBaseUrl(url) => write!(f, "invalid base url: {url}"),
            Self::Connection(err) => write!(f, "network error: {err}"),
            Self::Io(err) => write!(f, "file system error: {err}"),
            Self::Protocol(message) => write!(f, "protocol error: {message}"),
            Self::Server(status, body) => write!(f, "server returned {status}: {body}"),
            Self::InvalidPath(path) => write!(f, "invalid path: {path}"),
        }
    }
}

impl std::error::Error for SyncError {}

impl SyncClient {
    pub fn new(base_url: &str) -> Result<Self, SyncError> {
        let without_scheme = base_url
            .strip_prefix("http://")
            .ok_or_else(|| SyncError::InvalidBaseUrl(base_url.to_string()))?;

        if without_scheme.contains('/') {
            return Err(SyncError::InvalidBaseUrl(base_url.to_string()));
        }

        let (host, port) = if let Some((host, port)) = without_scheme.split_once(':') {
            let parsed_port = port
                .parse::<u16>()
                .map_err(|_| SyncError::InvalidBaseUrl(base_url.to_string()))?;
            (host.to_string(), parsed_port)
        } else {
            (without_scheme.to_string(), 80)
        };

        if host.is_empty() {
            return Err(SyncError::InvalidBaseUrl(base_url.to_string()));
        }

        Ok(Self { host, port })
    }

    pub fn health_check(&self) -> Result<bool, SyncError> {
        let response = self.send("GET", "/v1/health", "")?;
        Ok(response.status == 200)
    }

    pub fn sync_file(&self, req: &SyncRequest) -> Result<(), SyncError> {
        let body = format!("path={}\nhash={}\n", req.path, req.hash);
        let response = self.send("POST", "/v1/sync", &body)?;

        if response.status == 200 || response.status == 202 {
            return Ok(());
        }

        Err(SyncError::Server(response.status, response.body))
    }

    fn send(&self, method: &str, path: &str, body: &str) -> Result<HttpResponse, SyncError> {
        let address = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(address).map_err(SyncError::Connection)?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(SyncError::Connection)?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(SyncError::Connection)?;

        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.host,
            body.len(),
            body
        );

        stream
            .write_all(request.as_bytes())
            .map_err(SyncError::Connection)?;
        stream.flush().map_err(SyncError::Connection)?;

        let mut buffer = String::new();
        stream
            .read_to_string(&mut buffer)
            .map_err(SyncError::Connection)?;

        HttpResponse::parse(&buffer)
    }
}

#[derive(Debug)]
struct HttpResponse {
    status: u16,
    body: String,
}

impl HttpResponse {
    fn parse(raw: &str) -> Result<Self, SyncError> {
        let (head, body) = raw
            .split_once("\r\n\r\n")
            .ok_or_else(|| SyncError::Protocol("missing response separator".to_string()))?;

        let status_line = head
            .lines()
            .next()
            .ok_or_else(|| SyncError::Protocol("missing response status line".to_string()))?;

        let status = status_line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| SyncError::Protocol("missing response status code".to_string()))?
            .parse::<u16>()
            .map_err(|_| SyncError::Protocol("invalid response status code".to_string()))?;

        Ok(Self {
            status,
            body: body.to_string(),
        })
    }
}

pub trait SyncTransport {
    fn health_check(&mut self) -> Result<bool, SyncError>;
    fn sync_file(&mut self, req: &SyncRequest) -> Result<(), SyncError>;
}

pub struct HttpTransport {
    client: SyncClient,
}

impl HttpTransport {
    pub fn new(base_url: &str) -> Result<Self, SyncError> {
        Ok(Self {
            client: SyncClient::new(base_url)?,
        })
    }
}

impl SyncTransport for HttpTransport {
    fn health_check(&mut self) -> Result<bool, SyncError> {
        self.client.health_check()
    }

    fn sync_file(&mut self, req: &SyncRequest) -> Result<(), SyncError> {
        self.client.sync_file(req)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueueEntry {
    request: SyncRequest,
    attempts: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlushReport {
    pub succeeded: usize,
    pub failed: usize,
    pub remaining: usize,
}

pub struct SyncManager<T: SyncTransport> {
    transport: T,
    queue: VecDeque<QueueEntry>,
}

impl<T: SyncTransport> SyncManager<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            queue: VecDeque::new(),
        }
    }

    pub fn from_snapshot(transport: T, snapshot: Vec<SyncRequest>) -> Self {
        let queue = snapshot
            .into_iter()
            .map(|request| QueueEntry {
                request,
                attempts: 0,
            })
            .collect::<VecDeque<_>>();

        Self { transport, queue }
    }

    pub fn queue_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<SyncRequest, SyncError> {
        let request = build_sync_request(file_path.as_ref())?;
        self.queue.push_back(QueueEntry {
            request: request.clone(),
            attempts: 0,
        });
        Ok(request)
    }

    pub fn queue_directory<P: AsRef<Path>>(&mut self, directory_path: P) -> Result<usize, SyncError> {
        let mut files = Vec::new();
        collect_files(directory_path.as_ref(), &mut files)?;

        for file_path in &files {
            let request = build_sync_request(file_path)?;
            self.queue.push_back(QueueEntry {
                request,
                attempts: 0,
            });
        }

        Ok(files.len())
    }

    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    pub fn snapshot_queue(&self) -> Vec<SyncRequest> {
        self.queue
            .iter()
            .map(|entry| entry.request.clone())
            .collect::<Vec<_>>()
    }

    pub fn health_check(&mut self) -> Result<bool, SyncError> {
        self.transport.health_check()
    }

    pub fn flush_once(&mut self) -> FlushReport {
        let mut succeeded = 0;
        let mut failed = 0;
        let mut remaining = VecDeque::new();

        while let Some(mut entry) = self.queue.pop_front() {
            match self.transport.sync_file(&entry.request) {
                Ok(()) => {
                    succeeded += 1;
                }
                Err(_) => {
                    failed += 1;
                    entry.attempts += 1;
                    remaining.push_back(entry);
                }
            }
        }

        self.queue = remaining;

        FlushReport {
            succeeded,
            failed,
            remaining: self.queue.len(),
        }
    }
}

fn build_sync_request(file_path: &Path) -> Result<SyncRequest, SyncError> {
    if !file_path.is_file() {
        return Err(SyncError::InvalidPath(file_path.display().to_string()));
    }

    let hash = hash_file_streaming(file_path)?;
    Ok(SyncRequest {
        path: file_path.to_string_lossy().to_string(),
        hash,
    })
}

fn hash_file_streaming(file_path: &Path) -> Result<String, SyncError> {
    let file = File::open(file_path).map_err(SyncError::Io)?;
    let mut reader = BufReader::new(file);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(SyncError::Io)?;
        if bytes_read == 0 {
            break;
        }
        hasher.write(&buffer[..bytes_read]);
    }

    Ok(format!("{:016x}", hasher.finish()))
}

fn collect_files(directory_path: &Path, files: &mut Vec<PathBuf>) -> Result<(), SyncError> {
    if !directory_path.is_dir() {
        return Err(SyncError::InvalidPath(directory_path.display().to_string()));
    }

    let entries = fs::read_dir(directory_path).map_err(SyncError::Io)?;
    for entry in entries {
        let path = entry.map_err(SyncError::Io)?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Clone, Copy)]
    enum MockOutcome {
        Ok,
        Fail,
    }

    struct MockTransport {
        outcomes: VecDeque<MockOutcome>,
        requests: Vec<SyncRequest>,
    }

    impl MockTransport {
        fn with_outcomes(outcomes: Vec<MockOutcome>) -> Self {
            Self {
                outcomes: outcomes.into(),
                requests: Vec::new(),
            }
        }

        fn sent(&self) -> &[SyncRequest] {
            &self.requests
        }
    }

    impl SyncTransport for MockTransport {
        fn health_check(&mut self) -> Result<bool, SyncError> {
            Ok(true)
        }

        fn sync_file(&mut self, req: &SyncRequest) -> Result<(), SyncError> {
            self.requests.push(req.clone());
            let outcome = self.outcomes.pop_front().unwrap_or(MockOutcome::Ok);
            match outcome {
                MockOutcome::Ok => Ok(()),
                MockOutcome::Fail => Err(SyncError::Server(503, "service unavailable".to_string())),
            }
        }
    }

    #[test]
    fn health_check_returns_true_when_server_is_healthy() {
        let captured_request = Arc::new(Mutex::new(String::new()));
        let (base_url, handle) = start_mock_server(
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok",
            Arc::clone(&captured_request),
        );

        let client = SyncClient::new(&base_url).expect("client should parse mock URL");
        let is_healthy = client
            .health_check()
            .expect("health check should succeed against mock server");

        handle.join().expect("mock server thread should finish");

        assert!(is_healthy);
        let request = captured_request.lock().expect("capture lock should work");
        assert!(request.starts_with("GET /v1/health HTTP/1.1\r\n"));
    }

    #[test]
    fn sync_file_sends_expected_payload_to_server() {
        let captured_request = Arc::new(Mutex::new(String::new()));
        let (base_url, handle) = start_mock_server(
            "HTTP/1.1 202 Accepted\r\nContent-Length: 8\r\n\r\naccepted",
            Arc::clone(&captured_request),
        );

        let client = SyncClient::new(&base_url).expect("client should parse mock URL");
        client
            .sync_file(&SyncRequest {
                path: "notes/todo.txt".to_string(),
                hash: "abc123".to_string(),
            })
            .expect("202 response should be treated as successful enqueue");

        handle.join().expect("mock server thread should finish");

        let request = captured_request.lock().expect("capture lock should work");
        assert!(request.starts_with("POST /v1/sync HTTP/1.1\r\n"));
        assert!(request.contains("path=notes/todo.txt\nhash=abc123\n"));
    }

    #[test]
    fn sync_file_returns_error_when_server_rejects_payload() {
        let captured_request = Arc::new(Mutex::new(String::new()));
        let (base_url, handle) = start_mock_server(
            "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 4\r\n\r\noops",
            Arc::clone(&captured_request),
        );

        let client = SyncClient::new(&base_url).expect("client should parse mock URL");
        let error = client
            .sync_file(&SyncRequest {
                path: "notes/todo.txt".to_string(),
                hash: "abc123".to_string(),
            })
            .expect_err("500 response should be surfaced as a server error");

        handle.join().expect("mock server thread should finish");

        match error {
            SyncError::Server(status, body) => {
                assert_eq!(status, 500);
                assert_eq!(body, "oops");
            }
            other => panic!("expected SyncError::Server, got {other:?}"),
        }
    }

    #[test]
    fn syncs_a_single_file_story_using_mock_transport() {
        let temp = temp_dir("single-file");
        let file_path = temp.join("todo.txt");
        fs::write(&file_path, "one file to sync").expect("test file should be written");

        let transport = MockTransport::with_outcomes(vec![MockOutcome::Ok]);
        let mut manager = SyncManager::new(transport);

        manager
            .queue_file(&file_path)
            .expect("single file should be queued");

        let report = manager.flush_once();

        assert_eq!(report.succeeded, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.remaining, 0);
    }

    #[test]
    fn syncs_a_directory_story_with_nested_files() {
        let temp = temp_dir("directory");
        let nested = temp.join("nested");
        fs::create_dir_all(&nested).expect("nested directory should be created");

        fs::write(temp.join("a.txt"), "A").expect("file A should be written");
        fs::write(nested.join("b.txt"), "B").expect("file B should be written");

        let transport = MockTransport::with_outcomes(vec![MockOutcome::Ok, MockOutcome::Ok]);
        let mut manager = SyncManager::new(transport);

        let queued = manager
            .queue_directory(&temp)
            .expect("directory should be traversed and queued");

        assert_eq!(queued, 2);

        let report = manager.flush_once();
        assert_eq!(report.succeeded, 2);
        assert_eq!(report.remaining, 0);
    }

    #[test]
    fn syncs_a_large_file_story_and_streams_hashing() {
        let temp = temp_dir("large-file");
        let file_path = temp.join("large.bin");

        let chunk = vec![b'x'; 1024 * 1024];
        let mut content = Vec::with_capacity(12 * 1024 * 1024);
        for _ in 0..12 {
            content.extend_from_slice(&chunk);
        }
        fs::write(&file_path, &content).expect("large file should be written");

        let transport = MockTransport::with_outcomes(vec![MockOutcome::Ok]);
        let mut manager = SyncManager::new(transport);
        let request = manager
            .queue_file(&file_path)
            .expect("large file should be queued");

        assert!(!request.hash.is_empty());

        let report = manager.flush_once();
        assert_eq!(report.succeeded, 1);
        assert_eq!(report.remaining, 0);
    }

    #[test]
    fn keeps_failed_sync_in_queue_for_recovery_story() {
        let temp = temp_dir("failure");
        let file_path = temp.join("unstable.txt");
        fs::write(&file_path, "content").expect("test file should be written");

        let transport = MockTransport::with_outcomes(vec![MockOutcome::Fail]);
        let mut manager = SyncManager::new(transport);

        manager
            .queue_file(&file_path)
            .expect("file should be queued for sync");

        let report = manager.flush_once();

        assert_eq!(report.succeeded, 0);
        assert_eq!(report.failed, 1);
        assert_eq!(manager.pending_count(), 1);
    }

    #[test]
    fn recovers_after_restart_by_restoring_queue_snapshot_story() {
        let temp = temp_dir("recovery");
        let file_path = temp.join("retry.txt");
        fs::write(&file_path, "content").expect("test file should be written");

        let first_transport = MockTransport::with_outcomes(vec![MockOutcome::Fail]);
        let mut first_manager = SyncManager::new(first_transport);
        first_manager
            .queue_file(&file_path)
            .expect("file should be queued");

        let first_report = first_manager.flush_once();
        assert_eq!(first_report.failed, 1);
        assert_eq!(first_manager.pending_count(), 1);

        let snapshot = first_manager.snapshot_queue();

        let second_transport = MockTransport::with_outcomes(vec![MockOutcome::Ok]);
        let mut recovered_manager = SyncManager::from_snapshot(second_transport, snapshot);

        let second_report = recovered_manager.flush_once();

        assert_eq!(second_report.succeeded, 1);
        assert_eq!(recovered_manager.pending_count(), 0);
    }

    #[test]
    fn returns_invalid_path_error_when_file_does_not_exist() {
        let temp = temp_dir("missing-file");
        let missing = temp.join("nope.txt");

        let transport = MockTransport::with_outcomes(vec![]);
        let mut manager = SyncManager::new(transport);
        let error = manager
            .queue_file(&missing)
            .expect_err("missing file should fail queue step");

        match error {
            SyncError::InvalidPath(path) => {
                assert!(path.contains("nope.txt"));
            }
            other => panic!("expected SyncError::InvalidPath, got {other:?}"),
        }
    }

    #[test]
    fn retries_failed_file_on_next_flush_round() {
        let temp = temp_dir("retry");
        let file_path = temp.join("retry-me.txt");
        fs::write(&file_path, "content").expect("test file should be written");

        let transport = MockTransport::with_outcomes(vec![MockOutcome::Fail, MockOutcome::Ok]);
        let mut manager = SyncManager::new(transport);

        manager
            .queue_file(&file_path)
            .expect("file should be queued for retry test");

        let first = manager.flush_once();
        assert_eq!(first.failed, 1);
        assert_eq!(manager.pending_count(), 1);

        let second = manager.flush_once();
        assert_eq!(second.succeeded, 1);
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn directory_sync_continues_after_single_file_failure_and_recovers() {
        let temp = temp_dir("partial-directory");
        fs::write(temp.join("a.txt"), "A").expect("file A should be written");
        fs::write(temp.join("b.txt"), "B").expect("file B should be written");
        fs::write(temp.join("c.txt"), "C").expect("file C should be written");

        let transport = MockTransport::with_outcomes(vec![
            MockOutcome::Ok,
            MockOutcome::Fail,
            MockOutcome::Ok,
            MockOutcome::Ok,
        ]);
        let mut manager = SyncManager::new(transport);

        manager
            .queue_directory(&temp)
            .expect("directory should queue all files");

        let first = manager.flush_once();
        assert_eq!(first.succeeded, 2);
        assert_eq!(first.failed, 1);
        assert_eq!(manager.pending_count(), 1);

        let second = manager.flush_once();
        assert_eq!(second.succeeded, 1);
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn captures_story_sequence_single_then_directory_then_large() {
        let temp = temp_dir("story-sequence");
        let single = temp.join("single.txt");
        fs::write(&single, "single").expect("single file should be written");

        let dir = temp.join("dir");
        fs::create_dir_all(&dir).expect("dir should be created");
        fs::write(dir.join("one.txt"), "1").expect("one should be written");
        fs::write(dir.join("two.txt"), "2").expect("two should be written");

        let large = temp.join("large.dat");
        fs::write(&large, vec![b'z'; 4 * 1024 * 1024]).expect("large test file should be written");

        let transport = MockTransport::with_outcomes(vec![
            MockOutcome::Ok,
            MockOutcome::Ok,
            MockOutcome::Ok,
            MockOutcome::Ok,
        ]);
        let mut manager = SyncManager::new(transport);

        manager
            .queue_file(&single)
            .expect("single file story step should queue");
        manager
            .queue_directory(&dir)
            .expect("directory story step should queue");
        manager
            .queue_file(&large)
            .expect("large file story step should queue");

        let report = manager.flush_once();
        assert_eq!(report.succeeded, 4);
        assert_eq!(report.remaining, 0);
    }

    fn start_mock_server(
        response: &'static str,
        captured_request: Arc<Mutex<String>>,
    ) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("mock server should bind on a random port");
        let address = listener.local_addr().expect("local addr should be available");

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("mock server should accept one connection");
            let request = read_http_request(&mut stream);
            *captured_request.lock().expect("capture lock should work") = request;

            stream
                .write_all(response.as_bytes())
                .expect("mock server should write response");
            stream.flush().expect("mock server should flush response");
        });

        (format!("http://{}", address), handle)
    }

    fn read_http_request(stream: &mut TcpStream) -> String {
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 1024];

        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    if request_is_complete(&buffer) {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        String::from_utf8_lossy(&buffer).to_string()
    }

    fn request_is_complete(buffer: &[u8]) -> bool {
        let marker = b"\r\n\r\n";
        if let Some(header_end) = buffer.windows(marker.len()).position(|window| window == marker) {
            let body_start = header_end + marker.len();
            let headers = String::from_utf8_lossy(&buffer[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    if name.eq_ignore_ascii_case("Content-Length") {
                        return value.trim().parse::<usize>().ok();
                    }
                    None
                })
                .unwrap_or(0);

            return buffer.len() >= body_start + content_length;
        }

        false
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();

        let path = std::env::temp_dir().join(format!("rust-client-{label}-{nanos}"));
        fs::create_dir_all(&path).expect("temp test dir should be created");
        path
    }
}
