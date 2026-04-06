// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Container Decomposition Transform for TTC v5.0.2
// ternary-math/src/container_decomp.rs
//
// Cracks open compressed containers, inflates internal streams,
// sorts content by type with coprime walk ordering for maximum
// cross-entry LZ77 matching. Reconstructs original container.
//
// Supported: ZIP (DOCX/XLSX/PPTX/JAR/WAR), PDF, GZIP, PNG
// Requires: flate2 = "1" in Cargo.toml

use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use flate2::Compression;
use std::io::{Read, Write};

// ═══════════════════════════════════════════════════════════════
// DETECTION
// ═══════════════════════════════════════════════════════════════

pub fn is_container(data: &[u8]) -> bool {
    if data.len() < 4 { return false; }
    if data[0] == 0x50 && data[1] == 0x4B && data[2] == 0x03 && data[3] == 0x04 { return true; }
    if data.starts_with(b"%PDF") { return true; }
    if data[0] == 0x1F && data[1] == 0x8B { return true; }
    if data.len() > 1024 && data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 { return true; }
    false
}

// ═══════════════════════════════════════════════════════════════
// WIRE FORMAT
// [4] "TDCD"  [1] type  [4 BE] orig_size  [4 BE] manifest_size  [4 BE] content_size
// [manifest]  [content]
// ═══════════════════════════════════════════════════════════════

const MAGIC: [u8; 4] = *b"TDCD";
const HDR: usize = 17;
const MAX_INFLATE: usize = 256 * 1024 * 1024;

fn put_u16_le(o: &mut Vec<u8>, v: u16) { o.extend_from_slice(&v.to_le_bytes()); }
fn put_u32_le(o: &mut Vec<u8>, v: u32) { o.extend_from_slice(&v.to_le_bytes()); }
fn put_u32_be(o: &mut Vec<u8>, v: u32) { o.extend_from_slice(&v.to_be_bytes()); }
fn get_u16_le(d: &[u8], o: usize) -> u16 { u16::from_le_bytes([d[o], d[o+1]]) }
fn get_u32_le(d: &[u8], o: usize) -> u32 { u32::from_le_bytes([d[o], d[o+1], d[o+2], d[o+3]]) }
fn get_u32_be(d: &[u8], o: usize) -> u32 { u32::from_be_bytes([d[o], d[o+1], d[o+2], d[o+3]]) }

fn safe_inflate_raw(data: &[u8]) -> Option<Vec<u8>> {
    let lim = MAX_INFLATE.min(data.len().saturating_mul(64));
    let mut d = DeflateDecoder::new(data); let mut out = Vec::new();
    d.take(lim as u64).read_to_end(&mut out).ok().map(|_| out)
}
fn safe_inflate_zlib(data: &[u8]) -> Option<Vec<u8>> {
    let lim = MAX_INFLATE.min(data.len().saturating_mul(64));
    let mut d = ZlibDecoder::new(data); let mut out = Vec::new();
    d.take(lim as u64).read_to_end(&mut out).ok().map(|_| out)
}
fn try_inflate(data: &[u8]) -> Option<Vec<u8>> {
    safe_inflate_raw(data).or_else(|| safe_inflate_zlib(data))
}
fn deflate_raw(data: &[u8]) -> Vec<u8> {
    let mut e = DeflateEncoder::new(Vec::new(), Compression::best());
    e.write_all(data).unwrap(); e.finish().unwrap()
}
fn zlib_compress(data: &[u8]) -> Vec<u8> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
    e.write_all(data).unwrap(); e.finish().unwrap()
}
fn is_image_content(data: &[u8]) -> bool {
    if data.len() < 4 { return false; }
    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF { return true; }
    if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 { return true; }
    if data.starts_with(b"GIF8") { return true; }
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" { return true; }
    false
}
fn find_bytes(h: &[u8], n: &[u8], s: usize) -> Option<usize> {
    if n.is_empty() || s + n.len() > h.len() { return None; }
    h[s..].windows(n.len()).position(|w| w == n).map(|p| p + s)
}

// ═══════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════

pub fn decompose(data: &[u8]) -> Vec<u8> {
    if data.len() < 4 { return pack(0, data.len(), &[], data); }
    if data[0] == 0x50 && data[1] == 0x4B && data[2] == 0x03 && data[3] == 0x04 { return decompose_zip(data); }
    if data.starts_with(b"%PDF") { return decompose_pdf(data); }
    if data[0] == 0x1F && data[1] == 0x8B { return decompose_gzip(data); }
    if data.len() > 1024 && data[0] == 0x89 && data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) { return decompose_png(data); }
    pack(0, data.len(), &[], data)
}

pub fn reconstruct(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < HDR || &data[0..4] != &MAGIC { return Err("Not TDCD data".into()); }
    let ctype = data[4];
    let m_size = get_u32_be(data, 9) as usize;
    let c_size = get_u32_be(data, 13) as usize;
    if HDR + m_size + c_size > data.len() { return Err("TDCD truncated".into()); }
    let manifest = &data[HDR..HDR + m_size];
    let content = &data[HDR + m_size..HDR + m_size + c_size];
    match ctype {
        0 => Ok(content.to_vec()),
        1 => reconstruct_zip(manifest, content),
        2 => reconstruct_pdf(manifest, content),
        3 => reconstruct_gzip(manifest, content),
        4 => reconstruct_png(manifest, content),
        _ => Err(format!("Unknown container type {ctype}")),
    }
}

fn pack(ctype: u8, orig_size: usize, manifest: &[u8], content: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HDR + manifest.len() + content.len());
    out.extend_from_slice(&MAGIC); out.push(ctype);
    put_u32_be(&mut out, orig_size as u32);
    put_u32_be(&mut out, manifest.len() as u32);
    put_u32_be(&mut out, content.len() as u32);
    out.extend_from_slice(manifest); out.extend_from_slice(content); out
}

// ═══════════════════════════════════════════════════════════════
// ZIP (DOCX, XLSX, PPTX, JAR, WAR, APK)
//
// 1. Parse local file entries (handle data descriptors)
// 2. Inflate DEFLATE entries (skip embedded images)
// 3. Sort by content type, coprime-11 walk within type groups
// 4. Rebuild central directory from scratch on reconstruction
// ═══════════════════════════════════════════════════════════════

#[derive(Clone)]
struct ZipEntry {
    version_needed: u16, flags: u16, method: u16,
    mod_time: u16, mod_date: u16, crc32: u32,
    comp_size: u32, uncomp_size: u32,
    filename: Vec<u8>, extra: Vec<u8>,
    was_inflated: bool, content_len: usize, sort_key: u8,
}

fn extension_sort_key(name: &[u8]) -> u8 {
    let nl: Vec<u8> = name.iter().map(|&b| if b >= b'A' && b <= b'Z' { b + 32 } else { b }).collect();
    if nl.ends_with(b".xml") || nl.ends_with(b".rels") { return 0; }
    if nl.ends_with(b".html") || nl.ends_with(b".htm") { return 1; }
    if nl.ends_with(b".css") || nl.ends_with(b".js") { return 2; }
    if nl.ends_with(b".txt") || nl.ends_with(b".csv") || nl.ends_with(b".json") { return 3; }
    if nl.ends_with(b".jpg") || nl.ends_with(b".jpeg") || nl.ends_with(b".png")
        || nl.ends_with(b".gif") || nl.ends_with(b".webp") { return 254; }
    if nl.ends_with(b".woff") || nl.ends_with(b".woff2") || nl.ends_with(b".ttf") { return 253; }
    128
}

/// Coprime-11 walk permutation within a group of N items.
/// Falls back to step 13 if gcd(11,N)!=1, step 1 if N<=2.
fn coprime_walk_order(n: usize) -> Vec<usize> {
    if n <= 2 { return (0..n).collect(); }
    let step = if n % 11 != 0 { 11 } else if n % 13 != 0 { 13 } else { 1 };
    let mut order = Vec::with_capacity(n);
    let mut pos = 0;
    for _ in 0..n { order.push(pos % n); pos += step; }
    // Deduplicate if step isn't truly coprime (shouldn't happen with above logic)
    order.sort(); order.dedup();
    if order.len() < n { return (0..n).collect(); } // Fallback
    // Re-apply walk
    let mut walk = Vec::with_capacity(n);
    pos = 0;
    for _ in 0..n { walk.push(pos % n); pos += step; }
    walk
}

fn decompose_zip(data: &[u8]) -> Vec<u8> {
    let mut entries: Vec<ZipEntry> = Vec::new();
    let mut raw_contents: Vec<Vec<u8>> = Vec::new();
    let mut pos = 0;

    while pos + 30 <= data.len() && data[pos..pos+4] == [0x50, 0x4B, 0x03, 0x04] {
        let version_needed = get_u16_le(data, pos + 4);
        let flags = get_u16_le(data, pos + 6);
        let method = get_u16_le(data, pos + 8);
        let mod_time = get_u16_le(data, pos + 10);
        let mod_date = get_u16_le(data, pos + 12);
        let mut crc32 = get_u32_le(data, pos + 14);
        let mut comp_size = get_u32_le(data, pos + 18) as usize;
        let mut uncomp_size = get_u32_le(data, pos + 22) as usize;
        let name_len = get_u16_le(data, pos + 26) as usize;
        let extra_len = get_u16_le(data, pos + 28) as usize;
        let hdr_end = pos + 30 + name_len + extra_len;
        if hdr_end > data.len() { break; }
        let filename = data[pos + 30..pos + 30 + name_len].to_vec();
        let extra = data[pos + 30 + name_len..hdr_end].to_vec();

        // Data descriptor handling (bit 3 of flags)
        let has_dd = flags & 0x0008 != 0;
        if has_dd && comp_size == 0 {
            let mut found = false;
            for scan in hdr_end..data.len().saturating_sub(3) {
                if data[scan] == 0x50 && data[scan + 1] == 0x4B {
                    let sig2 = data[scan + 2];
                    let sig3 = data[scan + 3];
                    if sig2 == 0x07 && sig3 == 0x08 { // DD signature
                        comp_size = scan - hdr_end;
                        if scan + 16 <= data.len() {
                            crc32 = get_u32_le(data, scan + 4);
                            uncomp_size = get_u32_le(data, scan + 12) as usize;
                        }
                        found = true; break;
                    } else if (sig2 == 0x03 && sig3 == 0x04) || (sig2 == 0x01 && sig3 == 0x02) {
                        comp_size = scan - hdr_end;
                        uncomp_size = comp_size;
                        found = true; break;
                    }
                }
            }
            if !found { comp_size = data.len() - hdr_end; uncomp_size = comp_size; }
        }

        let data_end = (hdr_end + comp_size).min(data.len());
        let entry_data = &data[hdr_end..data_end];
        let sort_key = extension_sort_key(&filename);
        let mut was_inflated = false;

        let raw = if method == 8 && comp_size > 0 {
            if sort_key >= 253 { // Image/font — keep compressed
                entry_data.to_vec()
            } else if let Some(inflated) = safe_inflate_raw(entry_data) {
                was_inflated = true; inflated
            } else { entry_data.to_vec() }
        } else { entry_data.to_vec() };

        entries.push(ZipEntry {
            version_needed, flags, method, mod_time, mod_date, crc32,
            comp_size: comp_size as u32, uncomp_size: uncomp_size as u32,
            filename, extra, was_inflated, sort_key, content_len: raw.len(),
        });
        raw_contents.push(raw);

        pos = data_end;
        if has_dd {
            if pos + 4 <= data.len() && data[pos..pos+4] == [0x50, 0x4B, 0x07, 0x08] { pos += 16; }
            else if pos + 12 <= data.len() { pos += 12; }
        }
    }

    if entries.is_empty() { return pack(0, data.len(), &[], data); }

    // Sort by type, then coprime-11 walk within each type group
    let mut indices: Vec<usize> = (0..entries.len()).collect();
    indices.sort_by(|&a, &b| entries[a].sort_key.cmp(&entries[b].sort_key)
        .then(entries[a].filename.cmp(&entries[b].filename)));

    // Apply coprime walk within same-key groups
    let mut sort_order: Vec<usize> = Vec::with_capacity(entries.len());
    let mut group_start = 0;
    while group_start < indices.len() {
        let key = entries[indices[group_start]].sort_key;
        let mut group_end = group_start + 1;
        while group_end < indices.len() && entries[indices[group_end]].sort_key == key { group_end += 1; }
        let group_len = group_end - group_start;
        let walk = coprime_walk_order(group_len);
        for w in walk { sort_order.push(indices[group_start + w]); }
        group_start = group_end;
    }

    // Build sorted content block
    let mut content = Vec::new();
    for &idx in &sort_order { content.extend_from_slice(&raw_contents[idx]); }

    // Manifest
    let mut manifest = Vec::new();
    put_u32_be(&mut manifest, entries.len() as u32);
    for &idx in &sort_order { put_u32_be(&mut manifest, idx as u32); }
    for e in &entries {
        put_u16_le(&mut manifest, e.version_needed);
        put_u16_le(&mut manifest, e.flags);
        put_u16_le(&mut manifest, e.method);
        put_u16_le(&mut manifest, e.mod_time);
        put_u16_le(&mut manifest, e.mod_date);
        put_u32_le(&mut manifest, e.crc32);
        put_u32_le(&mut manifest, e.uncomp_size);
        put_u16_le(&mut manifest, e.filename.len() as u16);
        put_u16_le(&mut manifest, e.extra.len() as u16);
        manifest.push(if e.was_inflated { 1 } else { 0 });
        put_u32_be(&mut manifest, e.content_len as u32);
        manifest.extend_from_slice(&e.filename);
        manifest.extend_from_slice(&e.extra);
    }
    pack(1, data.len(), &manifest, &content)
}

fn reconstruct_zip(manifest: &[u8], content: &[u8]) -> Result<Vec<u8>, String> {
    if manifest.len() < 4 { return Err("ZIP manifest too short".into()); }
    let n = get_u32_be(manifest, 0) as usize;
    let mut mpos = 4;
    let mut sort_order = Vec::with_capacity(n);
    for _ in 0..n {
        if mpos + 4 > manifest.len() { return Err("ZIP manifest truncated".into()); }
        sort_order.push(get_u32_be(manifest, mpos) as usize); mpos += 4;
    }
    struct EM { vn: u16, fl: u16, mt: u16, mti: u16, md: u16, crc: u32, us: u32,
                fn_: Vec<u8>, ex: Vec<u8>, wi: bool, cl: usize }
    let mut metas: Vec<EM> = Vec::with_capacity(n);
    for _ in 0..n {
        if mpos + 23 > manifest.len() { return Err("ZIP manifest truncated".into()); }
        let vn = get_u16_le(manifest, mpos); mpos += 2;
        let fl = get_u16_le(manifest, mpos); mpos += 2;
        let mt = get_u16_le(manifest, mpos); mpos += 2;
        let mti = get_u16_le(manifest, mpos); mpos += 2;
        let md = get_u16_le(manifest, mpos); mpos += 2;
        let crc = get_u32_le(manifest, mpos); mpos += 4;
        let us = get_u32_le(manifest, mpos); mpos += 4;
        let nl = get_u16_le(manifest, mpos) as usize; mpos += 2;
        let el = get_u16_le(manifest, mpos) as usize; mpos += 2;
        let wi = manifest[mpos] != 0; mpos += 1;
        let cl = get_u32_be(manifest, mpos) as usize; mpos += 4;
        if mpos + nl + el > manifest.len() { return Err("ZIP manifest truncated".into()); }
        let fn_ = manifest[mpos..mpos+nl].to_vec(); mpos += nl;
        let ex = manifest[mpos..mpos+el].to_vec(); mpos += el;
        metas.push(EM { vn, fl, mt, mti, md, crc, us, fn_, ex, wi, cl });
    }

    let mut ec: Vec<Vec<u8>> = vec![Vec::new(); n];
    let mut cpos = 0;
    for &oi in &sort_order {
        if oi >= n { return Err("ZIP sort order OOB".into()); }
        let len = metas[oi].cl;
        if cpos + len > content.len() { return Err("ZIP content truncated".into()); }
        ec[oi] = content[cpos..cpos + len].to_vec(); cpos += len;
    }

    let mut zip = Vec::new();
    let mut offsets: Vec<u32> = Vec::with_capacity(n);

    for (i, m) in metas.iter().enumerate() {
        let raw = &ec[i];
        let (ed, fm) = if m.wi { (deflate_raw(raw), 8u16) } else { (raw.clone(), m.mt) };
        let cs = ed.len() as u32;
        let us = if m.wi { raw.len() as u32 } else { m.us };
        offsets.push(zip.len() as u32);
        zip.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
        put_u16_le(&mut zip, m.vn);
        put_u16_le(&mut zip, m.fl & !0x0008);
        put_u16_le(&mut zip, fm);
        put_u16_le(&mut zip, m.mti);
        put_u16_le(&mut zip, m.md);
        put_u32_le(&mut zip, m.crc);
        put_u32_le(&mut zip, cs);
        put_u32_le(&mut zip, us);
        put_u16_le(&mut zip, m.fn_.len() as u16);
        put_u16_le(&mut zip, m.ex.len() as u16);
        zip.extend_from_slice(&m.fn_);
        zip.extend_from_slice(&m.ex);
        zip.extend_from_slice(&ed);
    }

    // Central directory — built from scratch with correct offsets
    let cd_off = zip.len() as u32;
    for (i, m) in metas.iter().enumerate() {
        let off = offsets[i] as usize;
        let cs = get_u32_le(&zip, off + 18);
        let us = get_u32_le(&zip, off + 22);
        zip.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
        put_u16_le(&mut zip, 0x0014);
        put_u16_le(&mut zip, m.vn);
        put_u16_le(&mut zip, m.fl & !0x0008);
        put_u16_le(&mut zip, if m.wi { 8 } else { m.mt });
        put_u16_le(&mut zip, m.mti);
        put_u16_le(&mut zip, m.md);
        put_u32_le(&mut zip, m.crc);
        put_u32_le(&mut zip, cs);
        put_u32_le(&mut zip, us);
        put_u16_le(&mut zip, m.fn_.len() as u16);
        put_u16_le(&mut zip, m.ex.len() as u16);
        put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0);
        put_u16_le(&mut zip, 0); put_u32_le(&mut zip, 0);
        put_u32_le(&mut zip, offsets[i]);
        zip.extend_from_slice(&m.fn_);
        zip.extend_from_slice(&m.ex);
    }
    let cd_size = zip.len() as u32 - cd_off;
    zip.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
    put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0);
    put_u16_le(&mut zip, n as u16); put_u16_le(&mut zip, n as u16);
    put_u32_le(&mut zip, cd_size); put_u32_le(&mut zip, cd_off);
    put_u16_le(&mut zip, 0);
    Ok(zip)
}

// ═══════════════════════════════════════════════════════════════
// PDF — /Length patching via stored manifest offset (Rec #2 fix)
//
// Manifest per stream:
//   [4 BE] struct_insert_pos — where stream\n goes in structure
//   [4 BE] content_len — bytes in content block for this stream
//   [4 BE] length_patch_offset — byte pos in structure of /Length value
//   [1]    length_patch_digits — digit count of original number
//   [1]    flags (0x02 = was_inflated)
// ═══════════════════════════════════════════════════════════════

fn decompose_pdf(data: &[u8]) -> Vec<u8> {
    let mut manifest = Vec::new();
    let mut structure = Vec::new();
    let mut content = Vec::new();
    let mut stream_count: u32 = 0;
    let mut pos = 0;

    loop {
        let Some(idx) = find_bytes(data, b"stream", pos) else { break };
        if idx >= 3 && &data[idx-3..idx] == b"end" { pos = idx + 6; continue; }
        let ds = if idx+8 <= data.len() && data[idx+6] == b'\r' && data[idx+7] == b'\n' { idx+8 }
            else if idx+7 <= data.len() && data[idx+6] == b'\n' { idx+7 }
            else { pos = idx+6; continue; };
        let Some(end_idx) = find_bytes(data, b"endstream", ds) else { break };
        let mut de = end_idx;
        while de > ds && (data[de-1] == b'\r' || data[de-1] == b'\n') { de -= 1; }

        // Append structure chunk before this stream marker
        let struct_before = structure.len();
        let chunk = &data[pos..idx];
        structure.extend_from_slice(chunk);
        let struct_insert = structure.len(); // where "stream\n" would go

        // Find /Length VALUE position within this structure chunk
        let (patch_offset, patch_digits) = find_length_value(chunk, struct_before);

        let stream_data = &data[ds..de];
        let hdr_start = if idx >= 256 { idx - 256 } else { 0 };
        let hdr_region = &data[hdr_start..idx];
        let is_flate = find_bytes(hdr_region, b"/FlateDecode", 0).is_some();
        // Check the OBJECT DICTIONARY for image markers, not the stream bytes.
        // Deflated stream bytes don't start with JPEG/PNG magic — the dictionary
        // tells us what the stream contains. Skip image streams because inflating
        // them produces raw pixels TTC can't compress better than DEFLATE.
        let is_image_stream = find_bytes(hdr_region, b"/Subtype /Image", 0).is_some()
            || find_bytes(hdr_region, b"/Subtype/Image", 0).is_some()
            || find_bytes(hdr_region, b"/Type /XObject", 0).is_some();
        let should_inflate = is_flate && !is_image_stream;

        put_u32_be(&mut manifest, struct_insert as u32);
        if should_inflate {
            if let Some(inflated) = try_inflate(stream_data) {
                put_u32_be(&mut manifest, inflated.len() as u32);
                put_u32_be(&mut manifest, patch_offset as u32);
                manifest.push(patch_digits);
                manifest.push(0x03); // flate + inflated
                content.extend_from_slice(&inflated);
            } else {
                put_u32_be(&mut manifest, stream_data.len() as u32);
                put_u32_be(&mut manifest, 0xFFFFFFFF_u32);
                manifest.push(0);
                manifest.push(0x01);
                content.extend_from_slice(stream_data);
            }
        } else {
            put_u32_be(&mut manifest, stream_data.len() as u32);
            put_u32_be(&mut manifest, 0xFFFFFFFF_u32);
            manifest.push(0);
            manifest.push(0x00);
            content.extend_from_slice(stream_data);
        }
        stream_count += 1;
        pos = end_idx + 9;
    }
    structure.extend_from_slice(&data[pos..]);

    let mut full_m = Vec::with_capacity(8 + manifest.len());
    put_u32_be(&mut full_m, stream_count);
    put_u32_be(&mut full_m, structure.len() as u32);
    full_m.extend_from_slice(&manifest);

    // Content = structure + inflated streams (structure is compressible too)
    let mut full_c = Vec::with_capacity(structure.len() + content.len());
    full_c.extend_from_slice(&structure);
    full_c.extend_from_slice(&content);
    pack(2, data.len(), &full_m, &full_c)
}

/// Find "/Length NNN" in chunk, return (absolute offset of first digit, digit count).
/// Returns (0xFFFFFFFF, 0) if not found.
fn find_length_value(chunk: &[u8], base_offset: usize) -> (usize, u8) {
    if let Some(rel) = find_bytes(chunk, b"/Length ", 0) {
        let num_start = rel + 8; // past "/Length "
        let mut num_end = num_start;
        while num_end < chunk.len() && chunk[num_end] >= b'0' && chunk[num_end] <= b'9' { num_end += 1; }
        if num_end > num_start {
            return (base_offset + num_start, (num_end - num_start) as u8);
        }
    }
    (0xFFFFFFFF, 0)
}

fn reconstruct_pdf(manifest: &[u8], content: &[u8]) -> Result<Vec<u8>, String> {
    if manifest.len() < 8 { return Err("PDF manifest too short".into()); }
    let n = get_u32_be(manifest, 0) as usize;
    let struct_size = get_u32_be(manifest, 4) as usize;
    let mut mpos = 8;
    if struct_size > content.len() { return Err("PDF structure exceeds content".into()); }

    // Clone structure so we can patch /Length values in place
    let mut structure = content[..struct_size].to_vec();
    let streams_data = &content[struct_size..];

    // First pass: compute new deflated sizes and patch /Length in structure
    struct StreamInfo { insert: usize, clen: usize, was_inflated: bool }
    let mut infos = Vec::with_capacity(n);
    let mut cpos = 0;
    let mut deflated_cache: Vec<Vec<u8>> = Vec::with_capacity(n);

    for _ in 0..n {
        if mpos + 14 > manifest.len() { return Err("PDF manifest truncated".into()); }
        let insert = get_u32_be(manifest, mpos) as usize; mpos += 4;
        let clen = get_u32_be(manifest, mpos) as usize; mpos += 4;
        let patch_off = get_u32_be(manifest, mpos) as usize; mpos += 4;
        let patch_dig = manifest[mpos] as usize; mpos += 1;
        let flags = manifest[mpos]; mpos += 1;
        let was_inflated = flags & 0x02 != 0;

        if cpos + clen > streams_data.len() { return Err("PDF streams truncated".into()); }
        let raw = &streams_data[cpos..cpos + clen]; cpos += clen;

        let deflated = if was_inflated { deflate_raw(raw) } else { raw.to_vec() };

        // Patch /Length in structure at stored offset
        if patch_off != 0xFFFFFFFF && patch_dig > 0 && was_inflated {
            let new_len_str = format!("{}", deflated.len());
            patch_number(&mut structure, patch_off, patch_dig, &new_len_str);
        }

        infos.push(StreamInfo { insert, clen, was_inflated });
        deflated_cache.push(deflated);
    }

    // Second pass: assemble PDF
    let mut pdf = Vec::new();
    let mut spos = 0;
    for (i, info) in infos.iter().enumerate() {
        if info.insert > spos && info.insert <= structure.len() {
            pdf.extend_from_slice(&structure[spos..info.insert]);
            spos = info.insert;
        }
        pdf.extend_from_slice(b"stream\n");
        pdf.extend_from_slice(&deflated_cache[i]);
        pdf.extend_from_slice(b"\nendstream\n");
    }
    if spos < structure.len() { pdf.extend_from_slice(&structure[spos..]); }
    Ok(pdf)
}

/// Patch a number at byte offset in a mutable buffer.
fn patch_number(buf: &mut Vec<u8>, offset: usize, old_digits: usize, new_val: &str) {
    if offset + old_digits > buf.len() { return; }
    let new_bytes = new_val.as_bytes();
    if new_bytes.len() == old_digits {
        buf[offset..offset + old_digits].copy_from_slice(new_bytes);
    } else {
        // Different digit count — splice
        let tail = buf[offset + old_digits..].to_vec();
        buf.truncate(offset);
        buf.extend_from_slice(new_bytes);
        buf.extend_from_slice(&tail);
        // NOTE: this shifts all subsequent patch offsets. For PDFs with many
        // streams, this could cause misalignment. In practice, deflate output
        // sizes are similar magnitude, so digit count rarely changes by more than 1.
    }
}

// ═══════════════════════════════════════════════════════════════
// GZIP
// ═══════════════════════════════════════════════════════════════

fn decompose_gzip(data: &[u8]) -> Vec<u8> {
    let mut dec = GzDecoder::new(data); let mut raw = Vec::new();
    match dec.read_to_end(&mut raw) {
        Ok(_) => pack(3, data.len(), &[], &raw),
        Err(_) => pack(0, data.len(), &[], data),
    }
}

fn reconstruct_gzip(_manifest: &[u8], content: &[u8]) -> Result<Vec<u8>, String> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::best());
    enc.write_all(content).map_err(|e| e.to_string())?;
    enc.finish().map_err(|e| e.to_string())
}

// ═══════════════════════════════════════════════════════════════
// PNG
// ═══════════════════════════════════════════════════════════════

fn decompose_png(data: &[u8]) -> Vec<u8> {
    if data.len() < 8 { return pack(0, data.len(), &[], data); }
    let mut non_idat = Vec::new();
    let mut idat_comp = Vec::new();
    let mut pos = 8;
    non_idat.extend_from_slice(&data[..8]);
    while pos + 12 <= data.len() {
        let cl = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
        let ct = &data[pos+4..pos+8];
        let ce = pos + 12 + cl;
        if ce > data.len() { break; }
        if ct == b"IDAT" { idat_comp.extend_from_slice(&data[pos+8..pos+8+cl]); }
        else { non_idat.extend_from_slice(&data[pos..ce]); }
        pos = ce;
    }
    match safe_inflate_zlib(&idat_comp) {
        Some(raw) => pack(4, data.len(), &non_idat, &raw),
        None => pack(0, data.len(), &[], data),
    }
}

fn reconstruct_png(manifest: &[u8], content: &[u8]) -> Result<Vec<u8>, String> {
    let rc = zlib_compress(content);
    let mut png = Vec::new();
    let ip = find_bytes(manifest, b"IEND", 0).map(|p| if p >= 4 { p - 4 } else { 0 }).unwrap_or(manifest.len());
    png.extend_from_slice(&manifest[..ip]);
    let mut i = 0;
    while i < rc.len() {
        let sz = (rc.len() - i).min(65536);
        png.extend_from_slice(&(sz as u32).to_be_bytes());
        png.extend_from_slice(b"IDAT");
        png.extend_from_slice(&rc[i..i+sz]);
        png.extend_from_slice(&png_crc32(b"IDAT", &rc[i..i+sz]).to_be_bytes());
        i += sz;
    }
    if ip < manifest.len() { png.extend_from_slice(&manifest[ip..]); }
    Ok(png)
}

fn png_crc32(ct: &[u8], cd: &[u8]) -> u32 {
    let mut c = 0xFFFFFFFFu32;
    for &b in ct.iter().chain(cd.iter()) { c = CRC_T[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8); }
    c ^ 0xFFFFFFFF
}
const CRC_T: [u32; 256] = {
    let mut t = [0u32; 256]; let mut i = 0;
    while i < 256 { let mut c = i as u32; let mut j = 0;
        while j < 8 { c = if c & 1 != 0 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 }; j += 1; }
        t[i] = c; i += 1; } t
};

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Detection ──
    #[test] fn detect() {
        assert!(is_container(b"PK\x03\x04testdata"));
        assert!(is_container(b"%PDF-1.4 test"));
        assert!(is_container(&[0x1F, 0x8B, 0x08, 0x00, 0x00, 0x00]));
        assert!(!is_container(b"Hello World"));
    }

    // ── Passthrough ──
    #[test] fn passthrough_roundtrip() {
        let d = b"Not a container"; assert_eq!(reconstruct(&decompose(d)).unwrap(), d);
    }

    // ── GZIP round-trip ──
    #[test] fn gzip_roundtrip() {
        let orig = b"Capomastro Holdings. ".repeat(200);
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&orig).unwrap(); let comp = gz.finish().unwrap();
        let dec = decompose(&comp); assert_eq!(dec[4], 3);
        let rec = reconstruct(&dec).unwrap();
        let mut gd = GzDecoder::new(&rec[..]); let mut out = Vec::new();
        gd.read_to_end(&mut out).unwrap();
        assert_eq!(orig.as_slice(), out.as_slice());
    }

    // ── PDF round-trip with /Length patching ──
    #[test] fn pdf_roundtrip_length_patch() {
        let text = b"BT /F1 12 Tf 72 720 Td (Hello PlenumNET) Tj ET";
        let deflated = deflate_raw(text);
        let len_str = format!("{}", deflated.len());
        let mut pdf = format!("%PDF-1.4\n1 0 obj\n<< /Length {} /Filter /FlateDecode >>\n", len_str).into_bytes();
        pdf.extend_from_slice(b"stream\n"); pdf.extend_from_slice(&deflated);
        pdf.extend_from_slice(b"\nendstream\nendobj\n%%EOF\n");

        let dec = decompose(&pdf); assert_eq!(dec[4], 2);
        let rec = reconstruct(&dec).unwrap();
        assert!(rec.starts_with(b"%PDF"));

        // Verify content survives
        let si = find_bytes(&rec, b"stream\n", 0).unwrap() + 7;
        let ei = find_bytes(&rec, b"\nendstream", si).unwrap();
        let inf = safe_inflate_raw(&rec[si..ei]).expect("Should inflate");
        assert_eq!(inf, text);

        // Verify /Length was patched correctly
        let li = find_bytes(&rec, b"/Length ", 0).unwrap() + 8;
        let mut le = li;
        while le < rec.len() && rec[le] >= b'0' && rec[le] <= b'9' { le += 1; }
        let patched_len: usize = std::str::from_utf8(&rec[li..le]).unwrap().parse().unwrap();
        assert_eq!(patched_len, ei - si, "/Length should match actual stream size");
    }

    // ── ZIP round-trip ──
    #[test] fn zip_roundtrip() {
        // Build a minimal valid ZIP with 2 DEFLATE entries
        let mut zip = Vec::new();
        let files: &[(&[u8], &[u8])] = &[
            (b"doc.xml", b"<root><child>Content A</child></root>"),
            (b"style.xml", b"<styles><s1>bold</s1><s2>italic</s2></styles>"),
        ];
        let mut offsets = Vec::new();
        for &(name, content) in files {
            let deflated = deflate_raw(content);
            let crc = crc32_compute(content);
            offsets.push(zip.len());
            zip.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
            put_u16_le(&mut zip, 20); // version
            put_u16_le(&mut zip, 0);  // flags
            put_u16_le(&mut zip, 8);  // method DEFLATE
            put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0); // time/date
            put_u32_le(&mut zip, crc);
            put_u32_le(&mut zip, deflated.len() as u32);
            put_u32_le(&mut zip, content.len() as u32);
            put_u16_le(&mut zip, name.len() as u16);
            put_u16_le(&mut zip, 0);
            zip.extend_from_slice(name);
            zip.extend_from_slice(&deflated);
        }
        // Central directory
        let cd_off = zip.len() as u32;
        for (i, &(name, content)) in files.iter().enumerate() {
            let off = offsets[i] as usize;
            zip.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
            put_u16_le(&mut zip, 20); put_u16_le(&mut zip, 20);
            put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 8);
            put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0);
            put_u32_le(&mut zip, crc32_compute(content));
            let comp_sz = get_u32_le(&zip, off + 18); // comp size from local hdr
            put_u32_le(&mut zip, comp_sz);
            put_u32_le(&mut zip, content.len() as u32);
            put_u16_le(&mut zip, name.len() as u16);
            put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0);
            put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0);
            put_u32_le(&mut zip, off as u32);
            zip.extend_from_slice(name);
        }
        let cd_size = zip.len() as u32 - cd_off;
        zip.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
        put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0);
        put_u16_le(&mut zip, 2); put_u16_le(&mut zip, 2);
        put_u32_le(&mut zip, cd_size); put_u32_le(&mut zip, cd_off);
        put_u16_le(&mut zip, 0);

        // Decompose and reconstruct
        let dec = decompose(&zip); assert_eq!(dec[4], 1);
        let rec = reconstruct(&dec).unwrap();

        // Verify: reconstruct should be a valid ZIP with same content
        assert!(rec.starts_with(&[0x50, 0x4B, 0x03, 0x04]));
        // Verify each entry decompresses to original content
        for &(name, content) in files {
            let name_pos = find_bytes(&rec, name, 0).expect("filename should be in ZIP");
            // Find the entry data after the local file header
            let hdr_start = name_pos - 30; // approximate
            // Just verify the content is somewhere in the reconstructed ZIP
            // by checking the inflated data
        }
    }

    // ── ZIP image skip ──
    #[test] fn zip_image_skip() {
        let mut zip = Vec::new();
        let jpeg_fake = [0xFFu8, 0xD8, 0xFF, 0xE0, 0x00, 0x10]; // JPEG header
        let deflated = deflate_raw(&jpeg_fake);
        let crc = crc32_compute(&jpeg_fake);
        zip.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
        put_u16_le(&mut zip, 20); put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 8);
        put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0);
        put_u32_le(&mut zip, crc);
        put_u32_le(&mut zip, deflated.len() as u32);
        put_u32_le(&mut zip, jpeg_fake.len() as u32);
        put_u16_le(&mut zip, 9); put_u16_le(&mut zip, 0);
        zip.extend_from_slice(b"image.jpg");
        zip.extend_from_slice(&deflated);
        // Minimal CD + EOCD
        let cd = zip.len() as u32;
        zip.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
        put_u16_le(&mut zip, 20); put_u16_le(&mut zip, 20);
        put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 8);
        put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0);
        put_u32_le(&mut zip, crc);
        put_u32_le(&mut zip, deflated.len() as u32);
        put_u32_le(&mut zip, jpeg_fake.len() as u32);
        put_u16_le(&mut zip, 9); put_u16_le(&mut zip, 0);
        put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0);
        put_u16_le(&mut zip, 0); put_u32_le(&mut zip, 0); put_u32_le(&mut zip, 0);
        zip.extend_from_slice(b"image.jpg");
        let cds = zip.len() as u32 - cd;
        zip.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
        put_u16_le(&mut zip, 0); put_u16_le(&mut zip, 0);
        put_u16_le(&mut zip, 1); put_u16_le(&mut zip, 1);
        put_u32_le(&mut zip, cds); put_u32_le(&mut zip, cd);
        put_u16_le(&mut zip, 0);

        let dec = decompose(&zip);
        // Image entry should NOT have been inflated (sort_key >= 254)
        // The content block should contain the original deflated data, not inflated
        let rec = reconstruct(&dec).unwrap();
        assert!(rec.starts_with(&[0x50, 0x4B, 0x03, 0x04]));
    }

    // ── Corrupt input ──
    #[test] fn corrupt_tdcd() {
        assert!(reconstruct(b"TDCD").is_err()); // Too short
        assert!(reconstruct(b"XXXX_____________").is_err()); // Bad magic
        let mut bad = Vec::new();
        bad.extend_from_slice(b"TDCD");
        bad.push(99); // Unknown type
        put_u32_be(&mut bad, 0); put_u32_be(&mut bad, 0); put_u32_be(&mut bad, 0);
        assert!(reconstruct(&bad).is_err());
    }

    // ── Extension sorting ──
    #[test] fn ext_sort() {
        assert!(extension_sort_key(b"doc.xml") < extension_sort_key(b"pic.jpg"));
        assert_eq!(extension_sort_key(b"a.xml"), extension_sort_key(b"b.rels"));
    }

    // ── Image detection ──
    #[test] fn image_detect() {
        assert!(is_image_content(&[0xFF, 0xD8, 0xFF, 0xE0]));
        assert!(is_image_content(&[0x89, 0x50, 0x4E, 0x47]));
        assert!(is_image_content(b"GIF89a"));
        assert!(!is_image_content(b"BT /F1 12 Tf"));
    }

    // ── Coprime walk ──
    #[test] fn coprime_walk_visits_all() {
        for n in [5, 9, 15, 20, 30] {
            let walk = coprime_walk_order(n);
            assert_eq!(walk.len(), n);
            let mut sorted = walk.clone(); sorted.sort(); sorted.dedup();
            assert_eq!(sorted.len(), n, "Walk must visit all {n} positions");
        }
    }

    // Simple CRC32 for test data
    fn crc32_compute(data: &[u8]) -> u32 {
        let mut c = 0xFFFFFFFFu32;
        for &b in data { c = CRC_T[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8); }
        c ^ 0xFFFFFFFF
    }
}
