use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use super::{NetworkError, NetworkResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum T3pMethod {
    Get,
    Put,
    Post,
    Delete,
    Subscribe,
    Witness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum T3pStatus {
    Ok = 200,
    Created = 201,
    Accepted = 202,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    InternalError = 500,
    ServiceUnavailable = 503,
}

impl T3pStatus {
    pub fn code(&self) -> u16 {
        *self as u16
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct T3pHeader {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct T3pRequest {
    pub method: T3pMethod,
    pub path: String,
    pub headers: Vec<T3pHeader>,
    pub body: Vec<u8>,
    pub ternary_encoding: bool,
    pub phase_encrypted: bool,
    pub request_id: u64,
}

impl T3pRequest {
    pub fn new(method: T3pMethod, path: &str) -> Self {
        Self {
            method,
            path: String::from(path),
            headers: Vec::new(),
            body: Vec::new(),
            ternary_encoding: false,
            phase_encrypted: false,
            request_id: 0,
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push(T3pHeader {
            key: String::from(key),
            value: String::from(value),
        });
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn with_ternary_encoding(mut self) -> Self {
        self.ternary_encoding = true;
        self
    }

    pub fn with_phase_encryption(mut self) -> Self {
        self.phase_encrypted = true;
        self
    }

    pub fn content_length(&self) -> usize {
        self.body.len()
    }

    pub fn get_header(&self, key: &str) -> Option<&str> {
        self.headers.iter()
            .find(|h| h.key == key)
            .map(|h| h.value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct T3pResponse {
    pub status: T3pStatus,
    pub request_id: u64,
    pub headers: Vec<T3pHeader>,
    pub body: Vec<u8>,
    pub ternary_encoding: bool,
    pub phase_encrypted: bool,
}

impl T3pResponse {
    pub fn new(status: T3pStatus, request_id: u64) -> Self {
        Self {
            status,
            request_id,
            headers: Vec::new(),
            body: Vec::new(),
            ternary_encoding: false,
            phase_encrypted: false,
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push(T3pHeader {
            key: String::from(key),
            value: String::from(value),
        });
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn is_success(&self) -> bool {
        let code = self.status.code();
        (200..300).contains(&code)
    }

    pub fn is_error(&self) -> bool {
        let code = self.status.code();
        code >= 400
    }
}

#[derive(Debug)]
pub struct T3pSession {
    pub session_id: u64,
    is_active: bool,
    pub requests_sent: u64,
    pub responses_received: u64,
    pub ternary_mode: bool,
    pub phase_encryption_enabled: bool,
    pending_requests: BTreeMap<u64, T3pRequest>,
    next_request_id: u64,
}

impl T3pSession {
    pub fn new(session_id: u64) -> Self {
        Self {
            session_id,
            is_active: true,
            requests_sent: 0,
            responses_received: 0,
            ternary_mode: false,
            phase_encryption_enabled: false,
            pending_requests: BTreeMap::new(),
            next_request_id: 1,
        }
    }

    pub fn send_request(&mut self, mut request: T3pRequest) -> NetworkResult<u64> {
        if !self.is_active {
            return Err(NetworkError::SessionInactive);
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
        request.request_id = id;
        if self.ternary_mode {
            request.ternary_encoding = true;
        }
        if self.phase_encryption_enabled {
            request.phase_encrypted = true;
        }
        self.pending_requests.insert(id, request);
        self.requests_sent += 1;
        Ok(id)
    }

    pub fn receive_response(&mut self, response: T3pResponse) -> NetworkResult<()> {
        if !self.is_active {
            return Err(NetworkError::SessionInactive);
        }
        if self.pending_requests.remove(&response.request_id).is_none() {
            return Err(NetworkError::ProtocolError(String::from("No matching pending request")));
        }
        self.responses_received += 1;
        Ok(())
    }

    pub fn get(&mut self, path: &str) -> NetworkResult<u64> {
        let request = T3pRequest::new(T3pMethod::Get, path);
        self.send_request(request)
    }

    pub fn put(&mut self, path: &str, body: &[u8]) -> NetworkResult<u64> {
        let request = T3pRequest::new(T3pMethod::Put, path)
            .with_body(body.to_vec());
        self.send_request(request)
    }

    pub fn witness(&mut self, path: &str, body: &[u8]) -> NetworkResult<u64> {
        let request = T3pRequest::new(T3pMethod::Witness, path)
            .with_body(body.to_vec());
        self.send_request(request)
    }

    pub fn enable_ternary_mode(&mut self) {
        self.ternary_mode = true;
    }

    pub fn enable_phase_encryption(&mut self) {
        self.phase_encryption_enabled = true;
    }

    pub fn pending_count(&self) -> usize {
        self.pending_requests.len()
    }

    pub fn close(&mut self) {
        self.is_active = false;
        self.pending_requests.clear();
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_request_creation() {
        let req = T3pRequest::new(T3pMethod::Get, "/api/data");
        assert_eq!(req.method, T3pMethod::Get);
        assert_eq!(req.path, "/api/data");
        assert!(req.body.is_empty());
        assert!(!req.ternary_encoding);
        assert!(!req.phase_encrypted);
    }

    #[test]
    fn test_response_creation() {
        let resp = T3pResponse::new(T3pStatus::Ok, 1);
        assert_eq!(resp.status, T3pStatus::Ok);
        assert_eq!(resp.request_id, 1);
        assert!(resp.body.is_empty());
    }

    #[test]
    fn test_request_builder() {
        let req = T3pRequest::new(T3pMethod::Post, "/api/submit")
            .with_header("Content-Type", "application/ternary")
            .with_body(vec![1, 2, 3])
            .with_ternary_encoding()
            .with_phase_encryption();
        assert_eq!(req.method, T3pMethod::Post);
        assert_eq!(req.content_length(), 3);
        assert!(req.ternary_encoding);
        assert!(req.phase_encrypted);
        assert_eq!(req.get_header("Content-Type"), Some("application/ternary"));
    }

    #[test]
    fn test_response_success_error() {
        let ok = T3pResponse::new(T3pStatus::Ok, 1);
        assert!(ok.is_success());
        assert!(!ok.is_error());

        let created = T3pResponse::new(T3pStatus::Created, 2);
        assert!(created.is_success());

        let not_found = T3pResponse::new(T3pStatus::NotFound, 3);
        assert!(!not_found.is_success());
        assert!(not_found.is_error());

        let internal = T3pResponse::new(T3pStatus::InternalError, 4);
        assert!(internal.is_error());
    }

    #[test]
    fn test_session_creation() {
        let session = T3pSession::new(42);
        assert_eq!(session.session_id, 42);
        assert!(session.is_active());
        assert_eq!(session.requests_sent, 0);
        assert_eq!(session.responses_received, 0);
        assert_eq!(session.pending_count(), 0);
    }

    #[test]
    fn test_session_send_request() {
        let mut session = T3pSession::new(1);
        let req = T3pRequest::new(T3pMethod::Get, "/test");
        let id = session.send_request(req).unwrap();
        assert_eq!(id, 1);
        assert_eq!(session.requests_sent, 1);
        assert_eq!(session.pending_count(), 1);
    }

    #[test]
    fn test_session_receive_response() {
        let mut session = T3pSession::new(1);
        let req = T3pRequest::new(T3pMethod::Get, "/test");
        let id = session.send_request(req).unwrap();

        let resp = T3pResponse::new(T3pStatus::Ok, id);
        assert!(session.receive_response(resp).is_ok());
        assert_eq!(session.responses_received, 1);
        assert_eq!(session.pending_count(), 0);
    }

    #[test]
    fn test_session_receive_response_no_match() {
        let mut session = T3pSession::new(1);
        let resp = T3pResponse::new(T3pStatus::Ok, 999);
        assert!(session.receive_response(resp).is_err());
    }

    #[test]
    fn test_session_pending() {
        let mut session = T3pSession::new(1);
        session.send_request(T3pRequest::new(T3pMethod::Get, "/a")).unwrap();
        session.send_request(T3pRequest::new(T3pMethod::Get, "/b")).unwrap();
        session.send_request(T3pRequest::new(T3pMethod::Get, "/c")).unwrap();
        assert_eq!(session.pending_count(), 3);
    }

    #[test]
    fn test_session_get_convenience() {
        let mut session = T3pSession::new(1);
        let id = session.get("/api/resource").unwrap();
        assert_eq!(id, 1);
        assert_eq!(session.pending_count(), 1);
    }

    #[test]
    fn test_session_put_convenience() {
        let mut session = T3pSession::new(1);
        let id = session.put("/api/resource", &[1, 2, 3]).unwrap();
        assert_eq!(id, 1);
        assert_eq!(session.pending_count(), 1);
    }

    #[test]
    fn test_session_witness() {
        let mut session = T3pSession::new(1);
        let id = session.witness("/blockchain/witness", &[0xDE, 0xAD]).unwrap();
        assert_eq!(id, 1);
        assert_eq!(session.pending_count(), 1);
    }

    #[test]
    fn test_session_ternary_mode() {
        let mut session = T3pSession::new(1);
        session.enable_ternary_mode();
        assert!(session.ternary_mode);

        let id = session.get("/test").unwrap();
        let resp = T3pResponse::new(T3pStatus::Ok, id);
        session.receive_response(resp).unwrap();
    }

    #[test]
    fn test_session_phase_encryption() {
        let mut session = T3pSession::new(1);
        session.enable_phase_encryption();
        assert!(session.phase_encryption_enabled);
    }

    #[test]
    fn test_session_close() {
        let mut session = T3pSession::new(1);
        session.get("/test").unwrap();
        assert!(session.is_active());
        assert_eq!(session.pending_count(), 1);

        session.close();
        assert!(!session.is_active());
        assert_eq!(session.pending_count(), 0);
    }

    #[test]
    fn test_session_closed_send() {
        let mut session = T3pSession::new(1);
        session.close();
        assert!(session.get("/test").is_err());
    }

    #[test]
    fn test_request_headers() {
        let req = T3pRequest::new(T3pMethod::Get, "/")
            .with_header("Accept", "application/ternary")
            .with_header("Authorization", "Bearer token123");
        assert_eq!(req.get_header("Accept"), Some("application/ternary"));
        assert_eq!(req.get_header("Authorization"), Some("Bearer token123"));
        assert_eq!(req.get_header("Missing"), None);
    }

    #[test]
    fn test_t3p_method_variants() {
        let methods = [
            T3pMethod::Get,
            T3pMethod::Put,
            T3pMethod::Post,
            T3pMethod::Delete,
            T3pMethod::Subscribe,
            T3pMethod::Witness,
        ];
        assert_eq!(methods.len(), 6);
        assert_ne!(T3pMethod::Get, T3pMethod::Post);
    }

    #[test]
    fn test_t3p_status_variants() {
        assert_eq!(T3pStatus::Ok.code(), 200);
        assert_eq!(T3pStatus::Created.code(), 201);
        assert_eq!(T3pStatus::Accepted.code(), 202);
        assert_eq!(T3pStatus::BadRequest.code(), 400);
        assert_eq!(T3pStatus::Unauthorized.code(), 401);
        assert_eq!(T3pStatus::Forbidden.code(), 403);
        assert_eq!(T3pStatus::NotFound.code(), 404);
        assert_eq!(T3pStatus::InternalError.code(), 500);
        assert_eq!(T3pStatus::ServiceUnavailable.code(), 503);
    }

    #[test]
    fn test_response_with_headers_and_body() {
        let resp = T3pResponse::new(T3pStatus::Ok, 1)
            .with_header("X-Ternary", "true")
            .with_body(vec![10, 20, 30]);
        assert_eq!(resp.headers.len(), 1);
        assert_eq!(resp.body.len(), 3);
    }
}
