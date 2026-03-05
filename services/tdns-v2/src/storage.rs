// TDNS v2.3.2 — SQLite Storage
// Capomastro Holdings Ltd. — Applied Physics Division
//
// Three tables. One file. TRN records survive restarts.
//
//   trn_records — name → address + CRD + scan_hash + metadata
//   drift_log   — append-only audit trail of address changes
//   redirects   — temporary old→new mappings during grace period

use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

use crate::trn::Trn;
use crate::addr::CubeAddr;
use crate::scan::ScanHash;

/// SQLite-backed TRN storage.
pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    /// Open (or create) the database at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let db = Self { conn: Mutex::new(conn) };
        db.create_tables()?;
        Ok(db)
    }

    /// In-memory database (for tests).
    pub fn memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        let db = Self { conn: Mutex::new(conn) };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS trn_records (
                name            TEXT PRIMARY KEY,
                address         TEXT NOT NULL,
                crd             INTEGER NOT NULL,
                public_key      BLOB NOT NULL,
                ttl             INTEGER NOT NULL,
                registered_at   INTEGER NOT NULL,
                zone            TEXT NOT NULL,
                scan_hash       TEXT NOT NULL,
                confidence      TEXT,
                hptp_sync       TEXT,
                hptp_offset_ns  INTEGER,
                last_rescan     INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_trn_address ON trn_records(address);

            CREATE TABLE IF NOT EXISTS drift_log (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                name            TEXT NOT NULL,
                old_address     TEXT NOT NULL,
                new_address     TEXT NOT NULL,
                changed_dims    TEXT NOT NULL,
                detected_at     INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS redirects (
                old_address     TEXT PRIMARY KEY,
                new_address     TEXT NOT NULL,
                expires_ns      INTEGER NOT NULL
            );
        ").map_err(|e| e.to_string())
    }

    // ── TRN Records ─────────────────────────────────────────────────

    pub fn store(&self, trn: &Trn) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO trn_records (name, address, crd, public_key, ttl, registered_at, zone, scan_hash, confidence, hptp_sync, hptp_offset_ns, last_rescan)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                trn.name,
                trn.address.to_category_string(),
                trn.crd,
                trn.public_key,
                trn.ttl,
                trn.registered_at,
                trn.zone,
                trn.scan_hash.to_hex(),
                trn.confidence.as_ref().map(|c| serde_json::to_string(c).unwrap()),
                trn.hptp_sync_status.as_ref().map(|s| format!("{:?}", s)),
                trn.hptp_offset_ns,
                trn.last_rescan,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(&self, name: &str) -> Result<Option<Trn>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, address, crd, public_key, ttl, registered_at, zone, scan_hash, confidence, last_rescan
             FROM trn_records WHERE name = ?1"
        ).map_err(|e| e.to_string())?;

        let result = stmt.query_row(params![name], |row| {
            let addr_str: String = row.get(1)?;
            let hash_hex: String = row.get(7)?;
            let conf_json: Option<String> = row.get(8)?;

            Ok(Trn {
                name: row.get(0)?,
                address: CubeAddr::from_category_string(&addr_str).unwrap(),
                crd: row.get::<_, u8>(2)?,
                public_key: row.get(3)?,
                ttl: row.get(4)?,
                registered_at: row.get(5)?,
                zone: row.get(6)?,
                scan_hash: ScanHash::from_hex(&hash_hex),
                confidence: conf_json.and_then(|j| serde_json::from_str(&j).ok()),
                valid_from: None,
                valid_until: None,
                hptp_sync_status: None,
                hptp_offset_ns: None,
                attributes: None,
                last_rescan: row.get(9)?,
            })
        });

        match result {
            Ok(trn) => Ok(Some(trn)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn update(&self, trn: &Trn) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE trn_records SET address=?2, crd=?3, scan_hash=?4, confidence=?5, last_rescan=?6
             WHERE name=?1",
            params![
                trn.name,
                trn.address.to_category_string(),
                trn.crd,
                trn.scan_hash.to_hex(),
                trn.confidence.as_ref().map(|c| serde_json::to_string(c).unwrap()),
                trn.last_rescan,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete(&self, name: &str) -> Result<bool, String> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM trn_records WHERE name=?1", params![name])
            .map_err(|e| e.to_string())?;
        Ok(rows > 0)
    }

    pub fn count(&self) -> Result<u64, String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM trn_records", [], |row| row.get(0))
            .map_err(|e| e.to_string())
    }

    /// All names at a given address (for CRD management).
    pub fn at_address(&self, addr: &CubeAddr) -> Result<Vec<(String, u8)>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, crd FROM trn_records WHERE address=?1"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(params![addr.to_category_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u8>(1)?))
        }).map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    /// Smallest unused CRD (1-9) at an address. None if full.
    pub fn next_crd(&self, addr: &CubeAddr) -> Result<Option<u8>, String> {
        let occupied: Vec<u8> = self.at_address(addr)?
            .into_iter().map(|(_, crd)| crd).collect();
        Ok((1..=9u8).find(|d| !occupied.contains(d)))
    }

    /// Load all TRN records (for boot restoration).
    pub fn load_all(&self) -> Result<Vec<Trn>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, address, crd, public_key, ttl, registered_at, zone, scan_hash, confidence, last_rescan
             FROM trn_records"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([], |row| {
            let addr_str: String = row.get(1)?;
            let hash_hex: String = row.get(7)?;
            let conf_json: Option<String> = row.get(8)?;

            Ok(Trn {
                name: row.get(0)?,
                address: CubeAddr::from_category_string(&addr_str).unwrap(),
                crd: row.get::<_, u8>(2)?,
                public_key: row.get(3)?,
                ttl: row.get(4)?,
                registered_at: row.get(5)?,
                zone: row.get(6)?,
                scan_hash: ScanHash::from_hex(&hash_hex),
                confidence: conf_json.and_then(|j| serde_json::from_str(&j).ok()),
                valid_from: None,
                valid_until: None,
                hptp_sync_status: None,
                hptp_offset_ns: None,
                attributes: None,
                last_rescan: row.get(9)?,
            })
        }).map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    // ── Redirects ───────────────────────────────────────────────────

    pub fn store_redirect(&self, old: &CubeAddr, new: &CubeAddr, expires_ns: u64) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO redirects (old_address, new_address, expires_ns) VALUES (?1, ?2, ?3)",
            params![old.to_category_string(), new.to_category_string(), expires_ns],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn check_redirect(&self, addr: &CubeAddr, now_ns: u64) -> Result<Option<CubeAddr>, String> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT new_address FROM redirects WHERE old_address=?1 AND expires_ns > ?2",
            params![addr.to_category_string(), now_ns],
            |row| {
                let s: String = row.get(0)?;
                Ok(CubeAddr::from_category_string(&s).unwrap())
            },
        );
        match result {
            Ok(addr) => Ok(Some(addr)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn purge_expired_redirects(&self, now_ns: u64) -> Result<u64, String> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM redirects WHERE expires_ns <= ?1", params![now_ns])
            .map_err(|e| e.to_string())?;
        Ok(rows as u64)
    }

    // ── Drift Log ───────────────────────────────────────────────────

    pub fn log_drift(&self, name: &str, old: &CubeAddr, new: &CubeAddr, changed: &[usize], at: u64) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let dims_str = changed.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(",");
        conn.execute(
            "INSERT INTO drift_log (name, old_address, new_address, changed_dims, detected_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, old.to_category_string(), new.to_category_string(), dims_str, at],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ─── ScanHash helper ────────────────────────────────────────────────────────

impl ScanHash {
    pub fn from_hex(hex: &str) -> Self {
        let mut bytes = [0u8; 32];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            if i >= 32 { break; }
            let s = std::str::from_utf8(chunk).unwrap_or("00");
            bytes[i] = u8::from_str_radix(s, 16).unwrap_or(0);
        }
        ScanHash(bytes)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trn(name: &str, addr_str: &str, crd: u8) -> Trn {
        let addr = CubeAddr::from_category_string(addr_str).unwrap();
        Trn {
            name: name.into(),
            address: addr,
            crd,
            public_key: vec![0xDE, 0xAD],
            ttl: 3600,
            registered_at: 1_000_000_000,
            zone: "plm".into(),
            scan_hash: ScanHash::compute(b"test"),
            valid_from: None,
            valid_until: None,
            hptp_sync_status: None,
            hptp_offset_ns: None,
            attributes: None,
            last_rescan: None,
            confidence: Some(vec![9; 27]),
        }
    }

    #[test]
    fn store_load_delete() {
        let db = Db::memory().unwrap();
        let addr = "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313";

        db.store(&make_trn("google.plm", addr, 1)).unwrap();
        assert_eq!(db.count().unwrap(), 1);

        let trn = db.load("google.plm").unwrap().unwrap();
        assert_eq!(trn.name, "google.plm");
        assert_eq!(trn.crd, 1);
        assert_eq!(trn.confidence.unwrap().len(), 27);

        db.delete("google.plm").unwrap();
        assert_eq!(db.count().unwrap(), 0);
        assert!(db.load("google.plm").unwrap().is_none());
    }

    #[test]
    fn crd_assignment() {
        let db = Db::memory().unwrap();
        let addr = "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313";
        let cube = CubeAddr::from_category_string(addr).unwrap();

        for i in 1..=9u8 {
            db.store(&make_trn(&format!("e{}.plm", i), addr, i)).unwrap();
        }
        assert_eq!(db.next_crd(&cube).unwrap(), None);

        db.delete("e5.plm").unwrap();
        assert_eq!(db.next_crd(&cube).unwrap(), Some(5));
    }

    #[test]
    fn redirects() {
        let db = Db::memory().unwrap();
        let old = CubeAddr::from_category_string("WO:1111 WA:1111 WR:1111 WN:1111 WY:1111 HO:1111 PE:111").unwrap();
        let new = CubeAddr::from_category_string("WO:2222 WA:2222 WR:2222 WN:2222 WY:2222 HO:2222 PE:222").unwrap();

        db.store_redirect(&old, &new, 1000).unwrap();
        assert!(db.check_redirect(&old, 500).unwrap().is_some());
        assert!(db.check_redirect(&old, 2000).unwrap().is_none());
    }

    #[test]
    fn drift_log() {
        let db = Db::memory().unwrap();
        let old = CubeAddr::from_category_string("WO:1111 WA:1111 WR:1111 WN:1111 WY:1111 HO:1111 PE:111").unwrap();
        let new = CubeAddr::from_category_string("WO:2222 WA:2222 WR:2222 WN:2222 WY:2222 HO:2222 PE:222").unwrap();
        db.log_drift("test.plm", &old, &new, &[3, 8, 25], 999).unwrap();
        // No crash = success. Append-only.
    }

    #[test]
    fn load_all_for_boot() {
        let db = Db::memory().unwrap();
        let addr = "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313";
        db.store(&make_trn("a.plm", addr, 1)).unwrap();
        db.store(&make_trn("b.plm", addr, 2)).unwrap();

        let all = db.load_all().unwrap();
        assert_eq!(all.len(), 2);
    }
}
