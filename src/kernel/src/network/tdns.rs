use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TdnsRecordType {
    A,
    AAAA,
    TRN,
    PTR,
    SRV,
    TXT,
    CNAME,
    NS,
}

impl TdnsRecordType {
    fn ordinal(&self) -> u8 {
        match self {
            TdnsRecordType::A => 0,
            TdnsRecordType::AAAA => 1,
            TdnsRecordType::TRN => 2,
            TdnsRecordType::PTR => 3,
            TdnsRecordType::SRV => 4,
            TdnsRecordType::TXT => 5,
            TdnsRecordType::CNAME => 6,
            TdnsRecordType::NS => 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdnsRecord {
    pub name: String,
    pub record_type: TdnsRecordType,
    pub value: String,
    pub ttl: u32,
    pub created_tick: u64,
    pub priority: u16,
}

impl TdnsRecord {
    pub fn new(name: &str, record_type: TdnsRecordType, value: &str, ttl: u32) -> Self {
        Self {
            name: String::from(name),
            record_type,
            value: String::from(value),
            ttl,
            created_tick: 0,
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: u16) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_created_tick(mut self, tick: u64) -> Self {
        self.created_tick = tick;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdnsQuery {
    pub name: String,
    pub record_type: TdnsRecordType,
    pub query_id: u32,
    pub recursive: bool,
}

impl TdnsQuery {
    pub fn new(name: &str, record_type: TdnsRecordType, query_id: u32) -> Self {
        Self {
            name: String::from(name),
            record_type,
            query_id,
            recursive: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TdnsResponseStatus {
    NoError,
    NameError,
    ServerFailure,
    NotImplemented,
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdnsResponse {
    pub query_id: u32,
    pub status: TdnsResponseStatus,
    pub records: Vec<TdnsRecord>,
    pub authoritative: bool,
}

impl TdnsResponse {
    pub fn new(query_id: u32, status: TdnsResponseStatus) -> Self {
        Self {
            query_id,
            status,
            records: Vec::new(),
            authoritative: false,
        }
    }
}

#[derive(Debug)]
pub struct TdnsCache {
    entries: BTreeMap<(String, u8), Vec<TdnsRecord>>,
    max_entries: usize,
}

impl TdnsCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_entries,
        }
    }

    pub fn lookup(&self, name: &str, record_type: &TdnsRecordType) -> Option<&[TdnsRecord]> {
        let key = (String::from(name), record_type.ordinal());
        self.entries.get(&key).map(|v| v.as_slice())
    }

    pub fn insert(&mut self, record: TdnsRecord) {
        let key = (record.name.clone(), record.record_type.ordinal());
        let entry = self.entries.entry(key).or_insert_with(Vec::new);
        entry.push(record);

        if self.total_records() > self.max_entries {
            if let Some(first_key) = self.entries.keys().next().cloned() {
                self.entries.remove(&first_key);
            }
        }
    }

    pub fn remove(&mut self, name: &str, record_type: &TdnsRecordType) {
        let key = (String::from(name), record_type.ordinal());
        self.entries.remove(&key);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    fn total_records(&self) -> usize {
        self.entries.values().map(|v| v.len()).sum()
    }

    pub fn is_expired(&self, record: &TdnsRecord, current_tick: u64) -> bool {
        if record.ttl == 0 {
            return false;
        }
        current_tick > record.created_tick + record.ttl as u64
    }

    pub fn purge_expired(&mut self, current_tick: u64) {
        let keys: Vec<(String, u8)> = self.entries.keys().cloned().collect();
        for key in keys {
            if let Some(records) = self.entries.get_mut(&key) {
                records.retain(|r| {
                    if r.ttl == 0 {
                        return true;
                    }
                    current_tick <= r.created_tick + r.ttl as u64
                });
                if records.is_empty() {
                    self.entries.remove(&key);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct TdnsResolver {
    cache: TdnsCache,
    zones: BTreeMap<String, Vec<TdnsRecord>>,
    next_query_id: u32,
    queries_processed: u64,
}

impl TdnsResolver {
    pub fn new() -> Self {
        Self {
            cache: TdnsCache::new(10000),
            zones: BTreeMap::new(),
            next_query_id: 1,
            queries_processed: 0,
        }
    }

    pub fn add_zone(&mut self, zone: &str, records: Vec<TdnsRecord>) {
        self.zones.insert(String::from(zone), records);
    }

    pub fn resolve(&mut self, name: &str, record_type: TdnsRecordType) -> TdnsResponse {
        self.queries_processed += 1;
        let query_id = self.next_query_id;
        self.next_query_id += 1;

        if let Some(cached) = self.cache.lookup(name, &record_type) {
            let mut response = TdnsResponse::new(query_id, TdnsResponseStatus::NoError);
            response.records = cached.to_vec();
            response.authoritative = false;
            return response;
        }

        for (_zone_name, zone_records) in &self.zones {
            let matching: Vec<TdnsRecord> = zone_records.iter()
                .filter(|r| r.name == name && r.record_type == record_type)
                .cloned()
                .collect();
            if !matching.is_empty() {
                for record in &matching {
                    self.cache.insert(record.clone());
                }
                let mut response = TdnsResponse::new(query_id, TdnsResponseStatus::NoError);
                response.records = matching;
                response.authoritative = true;
                return response;
            }
        }

        TdnsResponse::new(query_id, TdnsResponseStatus::NameError)
    }

    pub fn query(&mut self, query: TdnsQuery) -> TdnsResponse {
        self.queries_processed += 1;

        if let Some(cached) = self.cache.lookup(&query.name, &query.record_type) {
            let mut response = TdnsResponse::new(query.query_id, TdnsResponseStatus::NoError);
            response.records = cached.to_vec();
            response.authoritative = false;
            return response;
        }

        for (_zone_name, zone_records) in &self.zones {
            let matching: Vec<TdnsRecord> = zone_records.iter()
                .filter(|r| r.name == query.name && r.record_type == query.record_type)
                .cloned()
                .collect();
            if !matching.is_empty() {
                for record in &matching {
                    self.cache.insert(record.clone());
                }
                let mut response = TdnsResponse::new(query.query_id, TdnsResponseStatus::NoError);
                response.records = matching;
                response.authoritative = true;
                return response;
            }
        }

        TdnsResponse::new(query.query_id, TdnsResponseStatus::NameError)
    }

    pub fn add_record(&mut self, zone: &str, record: TdnsRecord) {
        let zone_records = self.zones.entry(String::from(zone)).or_insert_with(Vec::new);
        zone_records.push(record);
    }

    pub fn remove_record(&mut self, zone: &str, name: &str, record_type: &TdnsRecordType) {
        if let Some(zone_records) = self.zones.get_mut(zone) {
            zone_records.retain(|r| !(r.name == name && r.record_type == *record_type));
        }
    }

    pub fn cache_stats(&self) -> (usize, u64) {
        (self.cache.size(), self.queries_processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_record_creation() {
        let record = TdnsRecord::new("node1.ternary.net", TdnsRecordType::A, "0.1.2.0.1.2", 3600);
        assert_eq!(record.name, "node1.ternary.net");
        assert_eq!(record.record_type, TdnsRecordType::A);
        assert_eq!(record.value, "0.1.2.0.1.2");
        assert_eq!(record.ttl, 3600);
        assert_eq!(record.priority, 0);
    }

    #[test]
    fn test_query_creation() {
        let query = TdnsQuery::new("node1.ternary.net", TdnsRecordType::A, 1);
        assert_eq!(query.name, "node1.ternary.net");
        assert_eq!(query.record_type, TdnsRecordType::A);
        assert_eq!(query.query_id, 1);
        assert!(query.recursive);
    }

    #[test]
    fn test_cache_insert_lookup() {
        let mut cache = TdnsCache::new(100);
        let record = TdnsRecord::new("test.net", TdnsRecordType::A, "1.2.0", 300);
        cache.insert(record);
        let result = cache.lookup("test.net", &TdnsRecordType::A);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
        assert_eq!(result.unwrap()[0].value, "1.2.0");
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = TdnsCache::new(2);
        cache.insert(TdnsRecord::new("a.net", TdnsRecordType::A, "1", 300));
        cache.insert(TdnsRecord::new("b.net", TdnsRecordType::A, "2", 300));
        cache.insert(TdnsRecord::new("c.net", TdnsRecordType::A, "3", 300));
        assert!(cache.size() <= 2);
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = TdnsCache::new(100);
        cache.insert(TdnsRecord::new("test.net", TdnsRecordType::A, "1.2.0", 300));
        assert!(cache.lookup("test.net", &TdnsRecordType::A).is_some());
        cache.remove("test.net", &TdnsRecordType::A);
        assert!(cache.lookup("test.net", &TdnsRecordType::A).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = TdnsCache::new(100);
        cache.insert(TdnsRecord::new("a.net", TdnsRecordType::A, "1", 300));
        cache.insert(TdnsRecord::new("b.net", TdnsRecordType::A, "2", 300));
        assert!(cache.size() > 0);
        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_expired() {
        let cache = TdnsCache::new(100);
        let record = TdnsRecord::new("test.net", TdnsRecordType::A, "1", 100)
            .with_created_tick(1000);
        assert!(!cache.is_expired(&record, 1050));
        assert!(cache.is_expired(&record, 1101));
    }

    #[test]
    fn test_cache_purge() {
        let mut cache = TdnsCache::new(100);
        cache.insert(TdnsRecord::new("old.net", TdnsRecordType::A, "1", 100)
            .with_created_tick(100));
        cache.insert(TdnsRecord::new("new.net", TdnsRecordType::A, "2", 1000)
            .with_created_tick(100));
        cache.purge_expired(300);
        assert!(cache.lookup("old.net", &TdnsRecordType::A).is_none());
        assert!(cache.lookup("new.net", &TdnsRecordType::A).is_some());
    }

    #[test]
    fn test_resolver_creation() {
        let resolver = TdnsResolver::new();
        let (cache_size, queries) = resolver.cache_stats();
        assert_eq!(cache_size, 0);
        assert_eq!(queries, 0);
    }

    #[test]
    fn test_resolver_add_zone() {
        let mut resolver = TdnsResolver::new();
        let records = vec![
            TdnsRecord::new("node1.ternary.net", TdnsRecordType::A, "0.1.2", 3600),
        ];
        resolver.add_zone("ternary.net", records);
    }

    #[test]
    fn test_resolver_resolve_from_zone() {
        let mut resolver = TdnsResolver::new();
        let records = vec![
            TdnsRecord::new("node1.ternary.net", TdnsRecordType::A, "0.1.2.0.1.2", 3600),
        ];
        resolver.add_zone("ternary.net", records);
        let response = resolver.resolve("node1.ternary.net", TdnsRecordType::A);
        assert_eq!(response.status, TdnsResponseStatus::NoError);
        assert_eq!(response.records.len(), 1);
        assert!(response.authoritative);
    }

    #[test]
    fn test_resolver_resolve_cache() {
        let mut resolver = TdnsResolver::new();
        let records = vec![
            TdnsRecord::new("node1.ternary.net", TdnsRecordType::A, "0.1.2", 3600),
        ];
        resolver.add_zone("ternary.net", records);

        resolver.resolve("node1.ternary.net", TdnsRecordType::A);
        let response2 = resolver.resolve("node1.ternary.net", TdnsRecordType::A);
        assert_eq!(response2.status, TdnsResponseStatus::NoError);
        assert!(!response2.authoritative);
    }

    #[test]
    fn test_resolver_name_not_found() {
        let mut resolver = TdnsResolver::new();
        let response = resolver.resolve("nonexistent.net", TdnsRecordType::A);
        assert_eq!(response.status, TdnsResponseStatus::NameError);
        assert!(response.records.is_empty());
    }

    #[test]
    fn test_resolver_query() {
        let mut resolver = TdnsResolver::new();
        let records = vec![
            TdnsRecord::new("node1.ternary.net", TdnsRecordType::A, "0.1.2", 3600),
        ];
        resolver.add_zone("ternary.net", records);

        let query = TdnsQuery::new("node1.ternary.net", TdnsRecordType::A, 42);
        let response = resolver.query(query);
        assert_eq!(response.query_id, 42);
        assert_eq!(response.status, TdnsResponseStatus::NoError);
        assert_eq!(response.records.len(), 1);
    }

    #[test]
    fn test_resolver_authoritative() {
        let mut resolver = TdnsResolver::new();
        let records = vec![
            TdnsRecord::new("node1.ternary.net", TdnsRecordType::A, "0.1.2", 3600),
        ];
        resolver.add_zone("ternary.net", records);

        let response = resolver.resolve("node1.ternary.net", TdnsRecordType::A);
        assert!(response.authoritative);
    }

    #[test]
    fn test_record_types() {
        let types = [
            TdnsRecordType::A,
            TdnsRecordType::AAAA,
            TdnsRecordType::TRN,
            TdnsRecordType::PTR,
            TdnsRecordType::SRV,
            TdnsRecordType::TXT,
            TdnsRecordType::CNAME,
            TdnsRecordType::NS,
        ];
        assert_eq!(types.len(), 8);
        for (i, t) in types.iter().enumerate() {
            assert_eq!(t.ordinal(), i as u8);
        }
    }

    #[test]
    fn test_srv_priority() {
        let record = TdnsRecord::new("_ttp._tcp.ternary.net", TdnsRecordType::SRV, "node1.ternary.net:8080", 3600)
            .with_priority(10);
        assert_eq!(record.priority, 10);
        assert_eq!(record.record_type, TdnsRecordType::SRV);
    }

    #[test]
    fn test_resolver_multiple_records() {
        let mut resolver = TdnsResolver::new();
        let records = vec![
            TdnsRecord::new("cluster.ternary.net", TdnsRecordType::A, "0.1.2", 3600),
            TdnsRecord::new("cluster.ternary.net", TdnsRecordType::A, "1.0.2", 3600),
            TdnsRecord::new("cluster.ternary.net", TdnsRecordType::A, "2.1.0", 3600),
        ];
        resolver.add_zone("ternary.net", records);

        let response = resolver.resolve("cluster.ternary.net", TdnsRecordType::A);
        assert_eq!(response.status, TdnsResponseStatus::NoError);
        assert_eq!(response.records.len(), 3);
    }

    #[test]
    fn test_resolver_cname() {
        let mut resolver = TdnsResolver::new();
        let records = vec![
            TdnsRecord::new("alias.ternary.net", TdnsRecordType::CNAME, "node1.ternary.net", 3600),
        ];
        resolver.add_zone("ternary.net", records);

        let response = resolver.resolve("alias.ternary.net", TdnsRecordType::CNAME);
        assert_eq!(response.status, TdnsResponseStatus::NoError);
        assert_eq!(response.records.len(), 1);
        assert_eq!(response.records[0].record_type, TdnsRecordType::CNAME);
    }

    #[test]
    fn test_cache_stats() {
        let mut resolver = TdnsResolver::new();
        let records = vec![
            TdnsRecord::new("node1.ternary.net", TdnsRecordType::A, "0.1.2", 3600),
        ];
        resolver.add_zone("ternary.net", records);

        resolver.resolve("node1.ternary.net", TdnsRecordType::A);
        resolver.resolve("node1.ternary.net", TdnsRecordType::A);
        resolver.resolve("missing.net", TdnsRecordType::A);

        let (cache_size, queries) = resolver.cache_stats();
        assert!(cache_size > 0);
        assert_eq!(queries, 3);
    }
}
