use alloc::string::String;
use alloc::vec::Vec;
use super::{NetworkError, NetworkResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    LastAck,
    TimeWait,
    Closing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TtpFlags {
    bits: u8,
}

impl TtpFlags {
    pub const SYN: u8 = 0x01;
    pub const ACK: u8 = 0x02;
    pub const FIN: u8 = 0x04;
    pub const RST: u8 = 0x08;
    pub const PSH: u8 = 0x10;
    pub const URG: u8 = 0x20;
    pub const TRN: u8 = 0x40;

    pub fn new(bits: u8) -> Self {
        Self { bits }
    }

    pub fn has_syn(&self) -> bool {
        self.bits & Self::SYN != 0
    }

    pub fn has_ack(&self) -> bool {
        self.bits & Self::ACK != 0
    }

    pub fn has_fin(&self) -> bool {
        self.bits & Self::FIN != 0
    }

    pub fn has_rst(&self) -> bool {
        self.bits & Self::RST != 0
    }

    pub fn has_psh(&self) -> bool {
        self.bits & Self::PSH != 0
    }

    pub fn has_trn(&self) -> bool {
        self.bits & Self::TRN != 0
    }

    pub fn from_flags(syn: bool, ack: bool, fin: bool, rst: bool, psh: bool, urg: bool, trn: bool) -> Self {
        let mut bits = 0u8;
        if syn { bits |= Self::SYN; }
        if ack { bits |= Self::ACK; }
        if fin { bits |= Self::FIN; }
        if rst { bits |= Self::RST; }
        if psh { bits |= Self::PSH; }
        if urg { bits |= Self::URG; }
        if trn { bits |= Self::TRN; }
        Self { bits }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtpHeader {
    pub source_port: u16,
    pub dest_port: u16,
    pub sequence_number: u32,
    pub ack_number: u32,
    pub flags: TtpFlags,
    pub window_size: u16,
    pub checksum: u32,
    pub urgent_pointer: u16,
}

impl TtpHeader {
    pub fn new(
        source_port: u16,
        dest_port: u16,
        sequence_number: u32,
        ack_number: u32,
        flags: TtpFlags,
        window_size: u16,
        checksum: u32,
        urgent_pointer: u16,
    ) -> Self {
        Self {
            source_port,
            dest_port,
            sequence_number,
            ack_number,
            flags,
            window_size,
            checksum,
            urgent_pointer,
        }
    }

    pub fn syn_packet(src: u16, dst: u16, seq: u32) -> Self {
        Self::new(src, dst, seq, 0, TtpFlags::new(TtpFlags::SYN), 65535, 0, 0)
    }

    pub fn syn_ack_packet(src: u16, dst: u16, seq: u32, ack: u32) -> Self {
        Self::new(src, dst, seq, ack, TtpFlags::new(TtpFlags::SYN | TtpFlags::ACK), 65535, 0, 0)
    }

    pub fn ack_packet(src: u16, dst: u16, seq: u32, ack: u32) -> Self {
        Self::new(src, dst, seq, ack, TtpFlags::new(TtpFlags::ACK), 65535, 0, 0)
    }

    pub fn data_packet(src: u16, dst: u16, seq: u32, ack: u32, window: u16) -> Self {
        Self::new(src, dst, seq, ack, TtpFlags::new(TtpFlags::ACK | TtpFlags::PSH), window, 0, 0)
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(19);
        bytes.extend_from_slice(&self.source_port.to_be_bytes());
        bytes.extend_from_slice(&self.dest_port.to_be_bytes());
        bytes.extend_from_slice(&self.sequence_number.to_be_bytes());
        bytes.extend_from_slice(&self.ack_number.to_be_bytes());
        bytes.push(self.flags.bits);
        bytes.extend_from_slice(&self.window_size.to_be_bytes());
        bytes.extend_from_slice(&self.urgent_pointer.to_be_bytes());
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtpSegment {
    pub header: TtpHeader,
    pub payload: Vec<u8>,
}

impl TtpSegment {
    pub fn new(header: TtpHeader, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    pub fn total_size(&self) -> usize {
        19 + self.payload.len()
    }

    pub fn compute_checksum(&self) -> u32 {
        let header_bytes = self.header.serialize();
        let mut sum: u32 = 0;
        for &byte in header_bytes.iter().chain(self.payload.iter()) {
            sum = sum.wrapping_add((byte % 3) as u32);
        }
        sum
    }

    pub fn verify_checksum(&self) -> bool {
        self.header.checksum == self.compute_checksum()
    }
}

#[derive(Debug)]
pub struct TtpConnection {
    state: TtpState,
    pub local_port: u16,
    pub remote_port: u16,
    local_seq: u32,
    remote_seq: u32,
    send_window: u16,
    recv_window: u16,
    send_buffer: Vec<TtpSegment>,
    recv_buffer: Vec<TtpSegment>,
    retransmit_count: u32,
    pub max_retransmits: u32,
    pub mss: u16,
    pub ternary_mode: bool,
}

impl TtpConnection {
    pub fn new(local_port: u16) -> Self {
        Self {
            state: TtpState::Closed,
            local_port,
            remote_port: 0,
            local_seq: 1000,
            remote_seq: 0,
            send_window: 65535,
            recv_window: 65535,
            send_buffer: Vec::new(),
            recv_buffer: Vec::new(),
            retransmit_count: 0,
            max_retransmits: 5,
            mss: 1458,
            ternary_mode: false,
        }
    }

    pub fn connect(&mut self, remote_port: u16) -> NetworkResult<TtpSegment> {
        if self.state != TtpState::Closed {
            return Err(NetworkError::InvalidState(String::from("Must be in Closed state to connect")));
        }
        self.remote_port = remote_port;
        let mut header = TtpHeader::syn_packet(self.local_port, remote_port, self.local_seq);
        if self.ternary_mode {
            header.flags = TtpFlags::new(TtpFlags::SYN | TtpFlags::TRN);
        }
        let segment = TtpSegment::new(header, Vec::new());
        self.state = TtpState::SynSent;
        self.send_buffer.push(segment.clone());
        Ok(segment)
    }

    pub fn listen(&mut self) -> NetworkResult<()> {
        if self.state != TtpState::Closed {
            return Err(NetworkError::InvalidState(String::from("Must be in Closed state to listen")));
        }
        self.state = TtpState::Listen;
        Ok(())
    }

    pub fn accept(&mut self, syn: &TtpSegment) -> NetworkResult<TtpSegment> {
        if self.state != TtpState::Listen {
            return Err(NetworkError::InvalidState(String::from("Must be in Listen state to accept")));
        }
        if !syn.header.flags.has_syn() {
            return Err(NetworkError::ProtocolError(String::from("Expected SYN packet")));
        }
        self.remote_port = syn.header.source_port;
        self.remote_seq = syn.header.sequence_number.wrapping_add(1);
        let header = TtpHeader::syn_ack_packet(
            self.local_port,
            self.remote_port,
            self.local_seq,
            self.remote_seq,
        );
        let segment = TtpSegment::new(header, Vec::new());
        self.state = TtpState::SynReceived;
        self.send_buffer.push(segment.clone());
        Ok(segment)
    }

    pub fn receive_segment(&mut self, segment: &TtpSegment) -> NetworkResult<Option<TtpSegment>> {
        match self.state {
            TtpState::SynSent => {
                if segment.header.flags.has_syn() && segment.header.flags.has_ack() {
                    self.remote_seq = segment.header.sequence_number.wrapping_add(1);
                    self.local_seq = self.local_seq.wrapping_add(1);
                    let header = TtpHeader::ack_packet(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                    );
                    self.state = TtpState::Established;
                    let ack = TtpSegment::new(header, Vec::new());
                    return Ok(Some(ack));
                }
                if segment.header.flags.has_rst() {
                    self.state = TtpState::Closed;
                    return Ok(None);
                }
                Ok(None)
            }
            TtpState::SynReceived => {
                if segment.header.flags.has_ack() {
                    self.state = TtpState::Established;
                    self.local_seq = self.local_seq.wrapping_add(1);
                    return Ok(None);
                }
                if segment.header.flags.has_rst() {
                    self.state = TtpState::Listen;
                    return Ok(None);
                }
                Ok(None)
            }
            TtpState::Established => {
                if segment.header.flags.has_fin() {
                    self.state = TtpState::CloseWait;
                    self.remote_seq = segment.header.sequence_number.wrapping_add(1);
                    let header = TtpHeader::ack_packet(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                    );
                    let ack = TtpSegment::new(header, Vec::new());
                    return Ok(Some(ack));
                }
                if segment.header.flags.has_rst() {
                    self.state = TtpState::Closed;
                    return Ok(None);
                }
                if !segment.payload.is_empty() {
                    self.recv_buffer.push(segment.clone());
                    self.remote_seq = segment.header.sequence_number
                        .wrapping_add(segment.payload.len() as u32);
                    let header = TtpHeader::ack_packet(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                    );
                    let ack = TtpSegment::new(header, Vec::new());
                    return Ok(Some(ack));
                }
                Ok(None)
            }
            TtpState::FinWait1 => {
                if segment.header.flags.has_ack() && segment.header.flags.has_fin() {
                    self.state = TtpState::TimeWait;
                    self.remote_seq = segment.header.sequence_number.wrapping_add(1);
                    let header = TtpHeader::ack_packet(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                    );
                    let ack = TtpSegment::new(header, Vec::new());
                    return Ok(Some(ack));
                }
                if segment.header.flags.has_ack() {
                    self.state = TtpState::FinWait2;
                    return Ok(None);
                }
                if segment.header.flags.has_fin() {
                    self.state = TtpState::Closing;
                    self.remote_seq = segment.header.sequence_number.wrapping_add(1);
                    let header = TtpHeader::ack_packet(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                    );
                    let ack = TtpSegment::new(header, Vec::new());
                    return Ok(Some(ack));
                }
                Ok(None)
            }
            TtpState::FinWait2 => {
                if segment.header.flags.has_fin() {
                    self.state = TtpState::TimeWait;
                    self.remote_seq = segment.header.sequence_number.wrapping_add(1);
                    let header = TtpHeader::ack_packet(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                    );
                    let ack = TtpSegment::new(header, Vec::new());
                    return Ok(Some(ack));
                }
                Ok(None)
            }
            TtpState::CloseWait => {
                Ok(None)
            }
            TtpState::LastAck => {
                if segment.header.flags.has_ack() {
                    self.state = TtpState::Closed;
                    return Ok(None);
                }
                Ok(None)
            }
            TtpState::Closing => {
                if segment.header.flags.has_ack() {
                    self.state = TtpState::TimeWait;
                    return Ok(None);
                }
                Ok(None)
            }
            TtpState::TimeWait => {
                Ok(None)
            }
            TtpState::Listen => {
                if segment.header.flags.has_syn() {
                    return self.accept(segment).map(Some);
                }
                Ok(None)
            }
            TtpState::Closed => {
                Ok(None)
            }
        }
    }

    pub fn send(&mut self, data: &[u8]) -> NetworkResult<Vec<TtpSegment>> {
        if self.state != TtpState::Established {
            return Err(NetworkError::InvalidState(String::from("Must be in Established state to send")));
        }
        let mut segments = Vec::new();
        let mss = self.mss as usize;
        let chunks: Vec<&[u8]> = data.chunks(mss).collect();
        for chunk in chunks {
            let header = TtpHeader::data_packet(
                self.local_port,
                self.remote_port,
                self.local_seq,
                self.remote_seq,
                self.send_window,
            );
            let segment = TtpSegment::new(header, chunk.to_vec());
            self.local_seq = self.local_seq.wrapping_add(chunk.len() as u32);
            self.send_buffer.push(segment.clone());
            segments.push(segment);
        }
        Ok(segments)
    }

    pub fn close(&mut self) -> NetworkResult<TtpSegment> {
        match self.state {
            TtpState::Established => {
                let header = TtpHeader::new(
                    self.local_port,
                    self.remote_port,
                    self.local_seq,
                    self.remote_seq,
                    TtpFlags::new(TtpFlags::FIN | TtpFlags::ACK),
                    self.send_window,
                    0,
                    0,
                );
                let segment = TtpSegment::new(header, Vec::new());
                self.state = TtpState::FinWait1;
                self.send_buffer.push(segment.clone());
                Ok(segment)
            }
            TtpState::CloseWait => {
                let header = TtpHeader::new(
                    self.local_port,
                    self.remote_port,
                    self.local_seq,
                    self.remote_seq,
                    TtpFlags::new(TtpFlags::FIN | TtpFlags::ACK),
                    self.send_window,
                    0,
                    0,
                );
                let segment = TtpSegment::new(header, Vec::new());
                self.state = TtpState::LastAck;
                self.send_buffer.push(segment.clone());
                Ok(segment)
            }
            _ => Err(NetworkError::InvalidState(String::from("Cannot close in current state"))),
        }
    }

    pub fn reset(&mut self) -> TtpSegment {
        let header = TtpHeader::new(
            self.local_port,
            self.remote_port,
            self.local_seq,
            self.remote_seq,
            TtpFlags::new(TtpFlags::RST),
            0,
            0,
            0,
        );
        self.state = TtpState::Closed;
        TtpSegment::new(header, Vec::new())
    }

    pub fn state(&self) -> &TtpState {
        &self.state
    }

    pub fn is_established(&self) -> bool {
        self.state == TtpState::Established
    }

    pub fn pending_data(&self) -> usize {
        self.recv_buffer.iter().map(|s| s.payload.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_ttp_flags() {
        let flags = TtpFlags::new(TtpFlags::SYN | TtpFlags::ACK);
        assert!(flags.has_syn());
        assert!(flags.has_ack());
        assert!(!flags.has_fin());
        assert!(!flags.has_rst());
        assert!(!flags.has_psh());
        assert!(!flags.has_trn());
    }

    #[test]
    fn test_ttp_flags_from_flags() {
        let flags = TtpFlags::from_flags(true, false, true, false, false, false, true);
        assert!(flags.has_syn());
        assert!(!flags.has_ack());
        assert!(flags.has_fin());
        assert!(!flags.has_rst());
        assert!(flags.has_trn());
    }

    #[test]
    fn test_ttp_header_creation() {
        let flags = TtpFlags::new(TtpFlags::SYN);
        let header = TtpHeader::new(8080, 80, 100, 0, flags, 65535, 0, 0);
        assert_eq!(header.source_port, 8080);
        assert_eq!(header.dest_port, 80);
        assert_eq!(header.sequence_number, 100);
        assert_eq!(header.ack_number, 0);
        assert!(header.flags.has_syn());
        assert_eq!(header.window_size, 65535);
    }

    #[test]
    fn test_syn_packet() {
        let header = TtpHeader::syn_packet(5000, 80, 1000);
        assert!(header.flags.has_syn());
        assert!(!header.flags.has_ack());
        assert_eq!(header.source_port, 5000);
        assert_eq!(header.dest_port, 80);
        assert_eq!(header.sequence_number, 1000);
    }

    #[test]
    fn test_syn_ack_packet() {
        let header = TtpHeader::syn_ack_packet(80, 5000, 2000, 1001);
        assert!(header.flags.has_syn());
        assert!(header.flags.has_ack());
        assert_eq!(header.source_port, 80);
        assert_eq!(header.dest_port, 5000);
        assert_eq!(header.sequence_number, 2000);
        assert_eq!(header.ack_number, 1001);
    }

    #[test]
    fn test_ack_packet() {
        let header = TtpHeader::ack_packet(5000, 80, 1001, 2001);
        assert!(!header.flags.has_syn());
        assert!(header.flags.has_ack());
        assert_eq!(header.sequence_number, 1001);
        assert_eq!(header.ack_number, 2001);
    }

    #[test]
    fn test_data_packet() {
        let header = TtpHeader::data_packet(5000, 80, 100, 200, 32768);
        assert!(header.flags.has_ack());
        assert!(header.flags.has_psh());
        assert_eq!(header.window_size, 32768);
    }

    #[test]
    fn test_segment_checksum() {
        let header = TtpHeader::syn_packet(8080, 80, 1000);
        let segment = TtpSegment::new(header, vec![1, 2, 3, 4, 5]);
        let checksum = segment.compute_checksum();
        assert!(checksum > 0);
    }

    #[test]
    fn test_segment_verify_checksum() {
        let header = TtpHeader::syn_packet(8080, 80, 1000);
        let mut segment = TtpSegment::new(header, vec![10, 20, 30]);
        let checksum = segment.compute_checksum();
        segment.header.checksum = checksum;
        assert!(segment.verify_checksum());
    }

    #[test]
    fn test_segment_total_size() {
        let header = TtpHeader::syn_packet(8080, 80, 1000);
        let segment = TtpSegment::new(header, vec![0u8; 100]);
        assert_eq!(segment.total_size(), 119);
    }

    #[test]
    fn test_connection_connect() {
        let mut conn = TtpConnection::new(5000);
        assert_eq!(*conn.state(), TtpState::Closed);
        let syn = conn.connect(80).unwrap();
        assert_eq!(*conn.state(), TtpState::SynSent);
        assert!(syn.header.flags.has_syn());
        assert_eq!(syn.header.source_port, 5000);
        assert_eq!(syn.header.dest_port, 80);
    }

    #[test]
    fn test_connection_listen() {
        let mut conn = TtpConnection::new(80);
        assert!(conn.listen().is_ok());
        assert_eq!(*conn.state(), TtpState::Listen);
    }

    #[test]
    fn test_connection_listen_invalid_state() {
        let mut conn = TtpConnection::new(80);
        conn.listen().unwrap();
        assert!(conn.listen().is_err());
    }

    #[test]
    fn test_connection_three_way_handshake() {
        let mut client = TtpConnection::new(5000);
        let mut server = TtpConnection::new(80);

        server.listen().unwrap();

        let syn = client.connect(80).unwrap();
        assert_eq!(*client.state(), TtpState::SynSent);

        let syn_ack = server.accept(&syn).unwrap();
        assert_eq!(*server.state(), TtpState::SynReceived);
        assert!(syn_ack.header.flags.has_syn());
        assert!(syn_ack.header.flags.has_ack());

        let ack = client.receive_segment(&syn_ack).unwrap();
        assert_eq!(*client.state(), TtpState::Established);
        assert!(ack.is_some());

        let ack_segment = ack.unwrap();
        server.receive_segment(&ack_segment).unwrap();
        assert_eq!(*server.state(), TtpState::Established);
    }

    #[test]
    fn test_connection_send_data() {
        let mut conn = TtpConnection::new(5000);
        conn.connect(80).unwrap();
        let syn_ack = TtpSegment::new(
            TtpHeader::syn_ack_packet(80, 5000, 2000, 1001),
            Vec::new(),
        );
        conn.receive_segment(&syn_ack).unwrap();
        assert!(conn.is_established());

        let data = vec![0u8; 100];
        let segments = conn.send(&data).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].payload.len(), 100);
    }

    #[test]
    fn test_connection_send_large_data() {
        let mut conn = TtpConnection::new(5000);
        conn.connect(80).unwrap();
        let syn_ack = TtpSegment::new(
            TtpHeader::syn_ack_packet(80, 5000, 2000, 1001),
            Vec::new(),
        );
        conn.receive_segment(&syn_ack).unwrap();

        let data = vec![0u8; 3000];
        let segments = conn.send(&data).unwrap();
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].payload.len(), 1458);
        assert_eq!(segments[1].payload.len(), 1458);
        assert_eq!(segments[2].payload.len(), 84);
    }

    #[test]
    fn test_connection_send_not_established() {
        let mut conn = TtpConnection::new(5000);
        let result = conn.send(&[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_connection_close() {
        let mut conn = TtpConnection::new(5000);
        conn.connect(80).unwrap();
        let syn_ack = TtpSegment::new(
            TtpHeader::syn_ack_packet(80, 5000, 2000, 1001),
            Vec::new(),
        );
        conn.receive_segment(&syn_ack).unwrap();
        assert!(conn.is_established());

        let fin = conn.close().unwrap();
        assert!(fin.header.flags.has_fin());
        assert_eq!(*conn.state(), TtpState::FinWait1);
    }

    #[test]
    fn test_connection_reset() {
        let mut conn = TtpConnection::new(5000);
        conn.connect(80).unwrap();
        let rst = conn.reset();
        assert!(rst.header.flags.has_rst());
        assert_eq!(*conn.state(), TtpState::Closed);
    }

    #[test]
    fn test_connection_ternary_mode() {
        let mut conn = TtpConnection::new(5000);
        conn.ternary_mode = true;
        let syn = conn.connect(80).unwrap();
        assert!(syn.header.flags.has_trn());
        assert!(syn.header.flags.has_syn());
    }

    #[test]
    fn test_ttp_state_transitions() {
        let mut conn = TtpConnection::new(5000);
        assert_eq!(*conn.state(), TtpState::Closed);
        conn.connect(80).unwrap();
        assert_eq!(*conn.state(), TtpState::SynSent);
    }

    #[test]
    fn test_connection_max_retransmits() {
        let conn = TtpConnection::new(5000);
        assert_eq!(conn.max_retransmits, 5);
    }

    #[test]
    fn test_connection_window_size() {
        let conn = TtpConnection::new(5000);
        assert_eq!(conn.send_window, 65535);
        assert_eq!(conn.recv_window, 65535);
    }

    #[test]
    fn test_connection_mss() {
        let conn = TtpConnection::new(5000);
        assert_eq!(conn.mss, 1458);
    }

    #[test]
    fn test_connection_pending_data() {
        let mut conn = TtpConnection::new(5000);
        assert_eq!(conn.pending_data(), 0);
        conn.recv_buffer.push(TtpSegment::new(
            TtpHeader::syn_packet(80, 5000, 100),
            vec![1, 2, 3, 4, 5],
        ));
        assert_eq!(conn.pending_data(), 5);
    }

    #[test]
    fn test_connection_connect_invalid_state() {
        let mut conn = TtpConnection::new(5000);
        conn.connect(80).unwrap();
        assert!(conn.connect(81).is_err());
    }

    #[test]
    fn test_segment_empty_payload_checksum() {
        let header = TtpHeader::syn_packet(8080, 80, 0);
        let segment = TtpSegment::new(header, Vec::new());
        let checksum = segment.compute_checksum();
        let mut segment2 = segment.clone();
        segment2.header.checksum = checksum;
        assert!(segment2.verify_checksum());
    }
}
