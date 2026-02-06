//! Inter-Process Communication (IPC)
//!
//! Provides message-passing between processes in the PlenumNET kernel.
//! Supports typed messages with ternary data payloads and security mode
//! enforcement for sender/receiver privilege validation.

use alloc::collections::VecDeque;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use crate::memory::SecurityMode;
use crate::timing::FemtosecondTimestamp;
use super::{ProcessId, ProcessError, ProcessResult};

pub type ChannelId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Data,
    Signal,
    TernaryPayload,
    PhaseComponent,
    TimingSync,
    Control,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: ProcessId,
    pub receiver: ProcessId,
    pub kind: MessageKind,
    pub payload: Vec<u8>,
    pub timestamp: FemtosecondTimestamp,
    pub sequence: u64,
}

impl Message {
    pub fn new(
        sender: ProcessId,
        receiver: ProcessId,
        kind: MessageKind,
        payload: Vec<u8>,
        timestamp: FemtosecondTimestamp,
    ) -> Self {
        Self {
            sender,
            receiver,
            kind,
            payload,
            timestamp,
            sequence: 0,
        }
    }

    pub fn signal(sender: ProcessId, receiver: ProcessId, timestamp: FemtosecondTimestamp) -> Self {
        Self::new(sender, receiver, MessageKind::Signal, Vec::new(), timestamp)
    }

    pub fn payload_size(&self) -> usize {
        self.payload.len()
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: ChannelId,
    pub owner: ProcessId,
    pub security_mode: SecurityMode,
    pub capacity: usize,
    messages: VecDeque<Message>,
    next_sequence: u64,
    total_sent: u64,
    total_received: u64,
}

impl Channel {
    pub fn new(id: ChannelId, owner: ProcessId, security_mode: SecurityMode, capacity: usize) -> Self {
        Self {
            id,
            owner,
            security_mode,
            capacity,
            messages: VecDeque::new(),
            next_sequence: 0,
            total_sent: 0,
            total_received: 0,
        }
    }

    pub fn send(&mut self, mut message: Message, sender_mode: SecurityMode) -> ProcessResult<()> {
        if !sender_mode.can_access(&self.security_mode) {
            return Err(ProcessError::SecurityViolation {
                pid: message.sender,
                required: self.security_mode,
                actual: sender_mode,
            });
        }

        if self.messages.len() >= self.capacity {
            return Err(ProcessError::IpcError(String::from("Channel full")));
        }

        message.sequence = self.next_sequence;
        self.next_sequence += 1;
        self.total_sent += 1;
        self.messages.push_back(message);
        Ok(())
    }

    pub fn receive(&mut self, receiver_mode: SecurityMode) -> ProcessResult<Message> {
        if !receiver_mode.can_access(&self.security_mode) {
            return Err(ProcessError::SecurityViolation {
                pid: 0,
                required: self.security_mode,
                actual: receiver_mode,
            });
        }

        self.messages.pop_front()
            .map(|msg| {
                self.total_received += 1;
                msg
            })
            .ok_or(ProcessError::IpcError(String::from("Channel empty")))
    }

    pub fn peek(&self) -> Option<&Message> {
        self.messages.front()
    }

    pub fn pending_count(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.messages.len() >= self.capacity
    }

    pub fn total_sent(&self) -> u64 {
        self.total_sent
    }

    pub fn total_received(&self) -> u64 {
        self.total_received
    }
}

pub struct MessageBus {
    channels: BTreeMap<ChannelId, Channel>,
    next_channel_id: ChannelId,
    process_channels: BTreeMap<ProcessId, Vec<ChannelId>>,
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            channels: BTreeMap::new(),
            next_channel_id: 0,
            process_channels: BTreeMap::new(),
        }
    }

    pub fn create_channel(
        &mut self,
        owner: ProcessId,
        security_mode: SecurityMode,
        capacity: usize,
    ) -> ChannelId {
        let id = self.next_channel_id;
        self.next_channel_id += 1;

        let channel = Channel::new(id, owner, security_mode, capacity);
        self.channels.insert(id, channel);

        self.process_channels
            .entry(owner)
            .or_insert_with(Vec::new)
            .push(id);

        id
    }

    pub fn destroy_channel(&mut self, channel_id: ChannelId) -> ProcessResult<()> {
        let channel = self.channels.remove(&channel_id)
            .ok_or(ProcessError::IpcError(String::from("Channel not found")))?;

        if let Some(chans) = self.process_channels.get_mut(&channel.owner) {
            chans.retain(|&c| c != channel_id);
        }

        Ok(())
    }

    pub fn send(
        &mut self,
        channel_id: ChannelId,
        message: Message,
        sender_mode: SecurityMode,
    ) -> ProcessResult<()> {
        let channel = self.channels.get_mut(&channel_id)
            .ok_or(ProcessError::IpcError(String::from("Channel not found")))?;
        channel.send(message, sender_mode)
    }

    pub fn receive(
        &mut self,
        channel_id: ChannelId,
        receiver_mode: SecurityMode,
    ) -> ProcessResult<Message> {
        let channel = self.channels.get_mut(&channel_id)
            .ok_or(ProcessError::IpcError(String::from("Channel not found")))?;
        channel.receive(receiver_mode)
    }

    pub fn get_channel(&self, channel_id: ChannelId) -> Option<&Channel> {
        self.channels.get(&channel_id)
    }

    pub fn channels_for_process(&self, pid: ProcessId) -> Vec<ChannelId> {
        self.process_channels.get(&pid)
            .cloned()
            .unwrap_or_default()
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn cleanup_process(&mut self, pid: ProcessId) {
        if let Some(chans) = self.process_channels.remove(&pid) {
            for cid in chans {
                self.channels.remove(&cid);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ts() -> FemtosecondTimestamp {
        FemtosecondTimestamp::new(1_000_000)
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::new(1, 2, MessageKind::Data, vec![1, 2, 3], make_ts());
        assert_eq!(msg.sender, 1);
        assert_eq!(msg.receiver, 2);
        assert_eq!(msg.payload_size(), 3);
    }

    #[test]
    fn test_signal_message() {
        let msg = Message::signal(1, 2, make_ts());
        assert_eq!(msg.kind, MessageKind::Signal);
        assert_eq!(msg.payload_size(), 0);
    }

    #[test]
    fn test_channel_creation() {
        let chan = Channel::new(0, 1, SecurityMode::ModeOne, 16);
        assert_eq!(chan.id, 0);
        assert_eq!(chan.owner, 1);
        assert!(chan.is_empty());
        assert!(!chan.is_full());
    }

    #[test]
    fn test_channel_send_receive() {
        let mut chan = Channel::new(0, 1, SecurityMode::ModeOne, 16);
        let msg = Message::new(1, 2, MessageKind::Data, vec![42], make_ts());

        chan.send(msg, SecurityMode::ModePhi).unwrap();
        assert_eq!(chan.pending_count(), 1);

        let received = chan.receive(SecurityMode::ModePhi).unwrap();
        assert_eq!(received.payload, vec![42]);
        assert_eq!(received.sequence, 0);
        assert!(chan.is_empty());
    }

    #[test]
    fn test_channel_security_enforcement_send() {
        let mut chan = Channel::new(0, 1, SecurityMode::ModePhi, 16);
        let msg = Message::new(1, 2, MessageKind::Data, vec![1], make_ts());

        let result = chan.send(msg, SecurityMode::ModeZero);
        assert!(result.is_err());
    }

    #[test]
    fn test_channel_security_enforcement_receive() {
        let mut chan = Channel::new(0, 1, SecurityMode::ModePhi, 16);
        let msg = Message::new(1, 2, MessageKind::Data, vec![1], make_ts());
        chan.send(msg, SecurityMode::ModePhi).unwrap();

        let result = chan.receive(SecurityMode::ModeOne);
        assert!(result.is_err());
    }

    #[test]
    fn test_channel_capacity() {
        let mut chan = Channel::new(0, 1, SecurityMode::ModeOne, 2);

        chan.send(Message::signal(1, 2, make_ts()), SecurityMode::ModeOne).unwrap();
        chan.send(Message::signal(1, 2, make_ts()), SecurityMode::ModeOne).unwrap();
        assert!(chan.is_full());

        let result = chan.send(Message::signal(1, 2, make_ts()), SecurityMode::ModeOne);
        assert!(result.is_err());
    }

    #[test]
    fn test_channel_sequence_numbers() {
        let mut chan = Channel::new(0, 1, SecurityMode::ModeOne, 16);

        chan.send(Message::signal(1, 2, make_ts()), SecurityMode::ModeOne).unwrap();
        chan.send(Message::signal(1, 2, make_ts()), SecurityMode::ModeOne).unwrap();
        chan.send(Message::signal(1, 2, make_ts()), SecurityMode::ModeOne).unwrap();

        let m1 = chan.receive(SecurityMode::ModeOne).unwrap();
        let m2 = chan.receive(SecurityMode::ModeOne).unwrap();
        let m3 = chan.receive(SecurityMode::ModeOne).unwrap();

        assert_eq!(m1.sequence, 0);
        assert_eq!(m2.sequence, 1);
        assert_eq!(m3.sequence, 2);
    }

    #[test]
    fn test_channel_stats() {
        let mut chan = Channel::new(0, 1, SecurityMode::ModeOne, 16);

        chan.send(Message::signal(1, 2, make_ts()), SecurityMode::ModeOne).unwrap();
        chan.send(Message::signal(1, 2, make_ts()), SecurityMode::ModeOne).unwrap();
        assert_eq!(chan.total_sent(), 2);

        chan.receive(SecurityMode::ModeOne).unwrap();
        assert_eq!(chan.total_received(), 1);
    }

    #[test]
    fn test_channel_peek() {
        let mut chan = Channel::new(0, 1, SecurityMode::ModeOne, 16);
        assert!(chan.peek().is_none());

        chan.send(Message::new(1, 2, MessageKind::Data, vec![99], make_ts()), SecurityMode::ModeOne).unwrap();
        let peeked = chan.peek().unwrap();
        assert_eq!(peeked.payload, vec![99]);
        assert_eq!(chan.pending_count(), 1);
    }

    #[test]
    fn test_message_bus_creation() {
        let bus = MessageBus::new();
        assert_eq!(bus.channel_count(), 0);
    }

    #[test]
    fn test_message_bus_create_channel() {
        let mut bus = MessageBus::new();
        let cid = bus.create_channel(1, SecurityMode::ModeOne, 16);
        assert_eq!(cid, 0);
        assert_eq!(bus.channel_count(), 1);
    }

    #[test]
    fn test_message_bus_send_receive() {
        let mut bus = MessageBus::new();
        let cid = bus.create_channel(1, SecurityMode::ModeOne, 16);

        let msg = Message::new(1, 2, MessageKind::Data, vec![42], make_ts());
        bus.send(cid, msg, SecurityMode::ModeOne).unwrap();

        let received = bus.receive(cid, SecurityMode::ModeOne).unwrap();
        assert_eq!(received.payload, vec![42]);
    }

    #[test]
    fn test_message_bus_destroy_channel() {
        let mut bus = MessageBus::new();
        let cid = bus.create_channel(1, SecurityMode::ModeOne, 16);
        bus.destroy_channel(cid).unwrap();
        assert_eq!(bus.channel_count(), 0);
    }

    #[test]
    fn test_message_bus_channels_for_process() {
        let mut bus = MessageBus::new();
        let c1 = bus.create_channel(1, SecurityMode::ModeOne, 16);
        let c2 = bus.create_channel(1, SecurityMode::ModeOne, 8);
        bus.create_channel(2, SecurityMode::ModeOne, 16);

        let chans = bus.channels_for_process(1);
        assert_eq!(chans.len(), 2);
        assert!(chans.contains(&c1));
        assert!(chans.contains(&c2));
    }

    #[test]
    fn test_message_bus_cleanup_process() {
        let mut bus = MessageBus::new();
        bus.create_channel(1, SecurityMode::ModeOne, 16);
        bus.create_channel(1, SecurityMode::ModeOne, 8);
        bus.create_channel(2, SecurityMode::ModeOne, 16);

        bus.cleanup_process(1);
        assert_eq!(bus.channel_count(), 1);
        assert!(bus.channels_for_process(1).is_empty());
    }

    #[test]
    fn test_message_bus_nonexistent_channel() {
        let mut bus = MessageBus::new();
        let msg = Message::signal(1, 2, make_ts());
        assert!(bus.send(999, msg, SecurityMode::ModeOne).is_err());
        assert!(bus.receive(999, SecurityMode::ModeOne).is_err());
    }

    #[test]
    fn test_ternary_payload_message() {
        let mut bus = MessageBus::new();
        let cid = bus.create_channel(1, SecurityMode::ModeOne, 16);

        let ternary_data: Vec<u8> = vec![0, 1, 2, 0, 1, 2, 0, 1, 2]; // Representation B
        let msg = Message::new(1, 2, MessageKind::TernaryPayload, ternary_data.clone(), make_ts());
        bus.send(cid, msg, SecurityMode::ModeOne).unwrap();

        let received = bus.receive(cid, SecurityMode::ModeOne).unwrap();
        assert_eq!(received.kind, MessageKind::TernaryPayload);
        assert_eq!(received.payload, ternary_data);
    }
}
