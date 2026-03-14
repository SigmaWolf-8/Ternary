// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TL-Sponge-385 N-API Native Addon — T-AE-MAC (Ternary Authenticated Encryption MAC)
//
// T-AE-MAC is a dual-phase authenticated encryption MAC over keyed TLSponge-385:
//   - Key enters via domain absorb (key material ‖ nonce ‖ phase angle)
//   - Keystream via bulk-rate squeeze (486 trits/perm) on cloned state
//   - GF(3) trit-wise OTP encryption (information-theoretic per nonce)
//   - MAC: ciphertext TRITS absorbed directly into base sponge (no binary)
//   - Tag squeezed from 486-trit capacity (385-bit PQ security)
//
// Permutation: χ(x)=x¹⁷ over GF(27) → 7-neighbor theta (±1,±7,±13) → π(i)=(376i+1) mod 729
// Forgery bound: q_f / 2^c where c=486 trits ≈ 385 bits
// Security proofs: TM-2026-008 (Representation Universality), TM-2026-011 (Phase Encryption)
//
// v1: no chi, sequential, standard rate (backward compat)
// v2: chi, sequential, standard rate
// v3: T-AE-MAC — chi, bulk rate keystream, dual-phase MAC on trits,
//     mac_trit_count=0 skips MAC (bulk mode, verify via TL-DSA),
//     auto-parallel when cores>1 AND data large enough

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;
use ternary_math::sponge::{Sponge385Pub, hash_hex, hash_hex_v1,
                            sponge_permutation, sponge_permutation_v1};
use ternary_math::tl_dsa;

const MAX_TRIT_COUNT: u32 = 1_000_000;
const MAX_PLAIN_BYTES: usize = 1_048_576;
const TRITS_PER_BYTE: usize = 6;
const RATE: usize = 243;
const PARALLEL_THRESHOLD: usize = RATE * 8;

#[inline(always)] fn trit_add(a:i8,b:i8)->i8{let s=a+b;if s>1{s-3}else if s< -1{s+3}else{s}}
#[inline(always)] fn trit_sub(a:i8,b:i8)->i8{let s=a-b;if s>1{s-3}else if s< -1{s+3}else{s}}

fn bytes_to_balanced_trits6(input:&[u8])->Vec<i8>{
    let mut t=Vec::with_capacity(input.len()*TRITS_PER_BYTE);
    for &b in input{let mut v=b as i32;for _ in 0..TRITS_PER_BYTE{t.push((v%3)as i8-1);v/=3;}}t}
fn balanced_trits6_to_bytes(trits:&[i8],byte_len:usize)->Vec<u8>{
    let mut out=vec![0u8;byte_len];let mut ti=0;
    for b in 0..byte_len{let mut v:i32=0;let mut m:i32=1;
        for _ in 0..TRITS_PER_BYTE{if ti<trits.len(){v+=(trits[ti]as i32+1)*m;m*=3;ti+=1;}}out[b]=v as u8;}out}
fn cipher_trits_to_bytes(trits:&[i8])->Vec<u8>{
    let pk=5;let bl=(trits.len()+pk-1)/pk;let mut out=vec![0u8;bl];let mut ti=0;
    for b in 0..bl{let mut v:i32=0;let mut m:i32=1;
        for _ in 0..pk{if ti<trits.len(){v+=(trits[ti]as i32+1)*m;m*=3;ti+=1;}}out[b]=(v&0xFF)as u8;}out}
fn cipher_bytes_to_trits(input:&[u8],tc:usize)->Vec<i8>{
    let mut t=vec![0i8;tc];let mut idx=0;
    for &b in input{if idx>=tc{break;}let mut v=if b<243{b}else{0}as i32;
        for _ in 0..5{if idx>=tc{break;}t[idx]=(v%3)as i8-1;v/=3;idx+=1;}}t}
fn trits_to_hex(trits:&[i8])->String{
    let pk=5;let bl=(trits.len()+pk-1)/pk;let mut bytes=Vec::with_capacity(bl);let mut i=0;
    while i<trits.len(){let mut v:u32=0;let mut m:u32=1;
        for _ in 0..pk{if i<trits.len(){v+=(trits[i]as u32+1)*m;m*=3;i+=1;}}bytes.push(v as u8);}
    hex::encode(&bytes)}

// ---- Bulk keystream generation ----

fn generate_keystream_v3(base:&Sponge385Pub,trit_count:usize,tag:&[u8])->Vec<i8>{
    let num_threads=rayon::current_num_threads();
    if trit_count<PARALLEL_THRESHOLD||num_threads<=1{
        let mut s=base.clone();s.absorb_bytes(tag);return s.squeeze_bulk(trit_count);}
    let max_useful=trit_count/RATE;
    let segments=num_threads.min(max_useful).max(1);
    let base_seg=trit_count/segments;let extra=trit_count%segments;
    let results:Vec<Vec<i8>>=(0..segments).into_par_iter().map(|i|{
        let mut fork=base.clone();
        let mut sep=Vec::with_capacity(tag.len()+2);sep.extend_from_slice(tag);sep.push(i as u8);sep.push((i>>8)as u8);
        fork.absorb_bytes(&sep);fork.squeeze_bulk(base_seg+if i<extra{1}else{0})
    }).collect();
    let mut out=Vec::with_capacity(trit_count);for seg in results{out.extend_from_slice(&seg);}out
}

fn cipher_trits(data:&[i8],keystream:&[i8],encrypt:bool)->Vec<i8>{
    let len=data.len();let mut out=vec![0i8;len];
    if len>4096&&rayon::current_num_threads()>1{
        out.par_chunks_mut(1024).enumerate().for_each(|(ci,chunk)|{let start=ci*1024;
            for i in 0..chunk.len(){chunk[i]=if encrypt{trit_add(data[start+i],keystream[start+i])}else{trit_sub(data[start+i],keystream[start+i])};}});
    }else{for i in 0..len{out[i]=if encrypt{trit_add(data[i],keystream[i])}else{trit_sub(data[i],keystream[i])};}}out
}

// ---- N-API exports ----

#[napi] pub fn sponge_hash(input:Buffer)->String{hash_hex(input.as_ref())}
#[napi] pub fn sponge_hash_v1(input:Buffer)->String{hash_hex_v1(input.as_ref())}
#[napi] pub fn sponge_keystream(domain_input:Buffer,trit_count:u32)->napi::Result<Buffer>{
    if trit_count>MAX_TRIT_COUNT{return Err(napi::Error::from_reason("trit_count exceeds max".to_string()));}
    let mut s=Sponge385Pub::new();s.absorb_bytes(domain_input.as_ref());
    Ok(Buffer::from(s.squeeze(trit_count as usize).iter().map(|&t|(t+1)as u8).collect::<Vec<_>>()))}
#[napi] pub fn sponge_keystream_v1(domain_input:Buffer,trit_count:u32)->napi::Result<Buffer>{
    if trit_count>MAX_TRIT_COUNT{return Err(napi::Error::from_reason("trit_count exceeds max".to_string()));}
    let mut s=Sponge385Pub::new_v1();s.absorb_bytes(domain_input.as_ref());
    Ok(Buffer::from(s.squeeze(trit_count as usize).iter().map(|&t|(t+1)as u8).collect::<Vec<_>>()))}
#[napi] pub fn sponge_derive_key(context:Buffer,material:Buffer,key_len:u32)->napi::Result<Buffer>{
    if key_len>MAX_TRIT_COUNT{return Err(napi::Error::from_reason("key_len exceeds max".to_string()));}
    Ok(Buffer::from(ternary_math::tlsponge385::derive_key(context.as_ref(),material.as_ref(),key_len as usize)))}
#[napi] pub fn sponge_permute_v2(state_buf:Buffer)->napi::Result<Buffer>{
    let src=state_buf.as_ref();if src.len()!=729{return Err(napi::Error::from_reason("state must be 729".to_string()));}
    let mut state=[0i8;729];for i in 0..729{state[i]=src[i]as i8;}sponge_permutation(&mut state);
    Ok(Buffer::from(state.iter().map(|&t|t as u8).collect::<Vec<_>>()))}
#[napi] pub fn sponge_permute_v1(state_buf:Buffer)->napi::Result<Buffer>{
    let src=state_buf.as_ref();if src.len()!=729{return Err(napi::Error::from_reason("state must be 729".to_string()));}
    let mut state=[0i8;729];for i in 0..729{state[i]=src[i]as i8;}sponge_permutation_v1(&mut state);
    Ok(Buffer::from(state.iter().map(|&t|t as u8).collect::<Vec<_>>()))}

// ---- Full phase encrypt (dual-phase) ----

#[napi]
pub fn phase_duplex_encrypt(
    domain_input:Buffer,primary_plain_bytes:Buffer,switch_marker:Buffer,
    secondary_plain_bytes:Buffer,mac_trit_count:u32,sponge_version:u32,
)->napi::Result<Buffer>{
    let p1=primary_plain_bytes.as_ref();let p2=secondary_plain_bytes.as_ref();
    if p1.len()>MAX_PLAIN_BYTES||p2.len()>MAX_PLAIN_BYTES{return Err(napi::Error::from_reason("plaintext too large".to_string()));}
    if mac_trit_count>MAX_TRIT_COUNT{return Err(napi::Error::from_reason("mac_trit_count too large".to_string()));}
    if sponge_version>=3{return phase_encrypt_v3(domain_input.as_ref(),p1,switch_marker.as_ref(),p2,mac_trit_count);}
    phase_encrypt_sequential(domain_input.as_ref(),p1,switch_marker.as_ref(),p2,mac_trit_count,sponge_version)
}

/// v1/v2: sequential, standard rate — exact existing behavior preserved
fn phase_encrypt_sequential(di:&[u8],p1:&[u8],sw:&[u8],p2:&[u8],mac_tc:u32,sv:u32)->napi::Result<Buffer>{
    let pt1=bytes_to_balanced_trits6(p1);let pt2=bytes_to_balanced_trits6(p2);
    let mut s=if sv>=2{Sponge385Pub::new()}else{Sponge385Pub::new_v1()};
    s.absorb_bytes(di);
    let ks1=s.squeeze(pt1.len());
    let ct1:Vec<i8>=pt1.iter().zip(ks1.iter()).map(|(&p,&k)|trit_add(p,k)).collect();
    let cb1=cipher_trits_to_bytes(&ct1);
    s.absorb_bytes(sw);
    let ks2=s.squeeze(pt2.len());
    let ct2:Vec<i8>=pt2.iter().zip(ks2.iter()).map(|(&p,&k)|trit_add(p,k)).collect();
    let cb2=cipher_trits_to_bytes(&ct2);
    let mut h1=[0u8;8];h1[0..4].copy_from_slice(&(p1.len()as u32).to_be_bytes());h1[4..8].copy_from_slice(&(pt1.len()as u32).to_be_bytes());
    let mut h2=[0u8;8];h2[0..4].copy_from_slice(&(p2.len()as u32).to_be_bytes());h2[4..8].copy_from_slice(&(pt2.len()as u32).to_be_bytes());
    s.absorb_bytes(&h1);s.absorb_bytes(&cb1);s.absorb_bytes(&h2);s.absorb_bytes(&cb2);
    let mac_hex=trits_to_hex(&s.squeeze(mac_tc as usize));
    let fc1=[&h1[..],&cb1[..]].concat();let fc2=[&h2[..],&cb2[..]].concat();
    let mut out=Vec::with_capacity(4+fc1.len()+4+fc2.len()+mac_hex.len());
    out.extend_from_slice(&(fc1.len()as u32).to_le_bytes());out.extend_from_slice(&fc1);
    out.extend_from_slice(&(fc2.len()as u32).to_le_bytes());out.extend_from_slice(&fc2);
    out.extend_from_slice(mac_hex.as_bytes());
    Ok(Buffer::from(out))
}

/// v3 T-AE-MAC: bulk keystream, dual-phase MAC on trits (no binary round-trip), auto-parallel
///
/// T-AE-MAC dual-phase flow:
///   1. base sponge absorbs domain (key ‖ nonce ‖ phase_angle ‖ context)
///   2. Clone base → fork per phase → bulk squeeze keystream trits
///   3. GF(3) trit cipher: cipher_trits = plain_trits + keystream_trits
///   4. MAC (mac_tc > 0): base sponge absorbs headers (bytes, tiny) + cipher TRITS
///      directly — sponge never leaves ternary domain for MAC
///   5. Squeeze T-AE-MAC tag from capacity (385-bit security)
///   6. MAC skip (mac_tc = 0): zero MAC permutations, integrity via external TL-DSA
///   7. Cipher trits → bytes ONLY for wire output (last step)
fn phase_encrypt_v3(di:&[u8],p1:&[u8],sw:&[u8],p2:&[u8],mac_tc:u32)->napi::Result<Buffer>{
    let mut base=Sponge385Pub::new();
    base.absorb_bytes(di);

    let pt1=bytes_to_balanced_trits6(p1);
    let pt2=bytes_to_balanced_trits6(p2);
    let mut sec_tag=Vec::with_capacity(1+sw.len());sec_tag.push(0x01);sec_tag.extend_from_slice(sw);

    let use_parallel=rayon::current_num_threads()>1
        &&(pt1.len()>=PARALLEL_THRESHOLD||pt2.len()>=PARALLEL_THRESHOLD);

    // Keystream: bulk rate (486 trits/perm = 2x throughput)
    let(ks1,ks2)=if use_parallel{
        rayon::join(
            ||generate_keystream_v3(&base,pt1.len(),&[0x00]),
            ||generate_keystream_v3(&base,pt2.len(),&sec_tag))
    }else{
        (generate_keystream_v3(&base,pt1.len(),&[0x00]),
         generate_keystream_v3(&base,pt2.len(),&sec_tag))
    };

    // GF(3) cipher — all ternary, no binary
    let(ct1,ct2)=if use_parallel{
        rayon::join(||cipher_trits(&pt1,&ks1,true),||cipher_trits(&pt2,&ks2,true))
    }else{
        (cipher_trits(&pt1,&ks1,true),cipher_trits(&pt2,&ks2,true))
    };

    // Headers (8 bytes each — only binary data in the whole pipeline, unavoidable for transport)
    let mut h1=[0u8;8];h1[0..4].copy_from_slice(&(p1.len()as u32).to_be_bytes());h1[4..8].copy_from_slice(&(pt1.len()as u32).to_be_bytes());
    let mut h2=[0u8;8];h2[0..4].copy_from_slice(&(p2.len()as u32).to_be_bytes());h2[4..8].copy_from_slice(&(pt2.len()as u32).to_be_bytes());

    // T-AE-MAC: dual-phase: base sponge (never squeezed keystream), absorbing TRITS directly.
    // base already has domain absorbed. Cipher trits feed in without binary conversion.
    // Headers are 8 bytes (40 trits) — absorb_bytes is fine for those.
    //
    // IMPORTANT: mac_trit_count=0 is BULK CONFIDENTIALITY MODE — encryption only,
    // NO authentication tag. Integrity MUST be provided by an outer mechanism
    // (TL-DSA signature over ciphertext, or application-layer verification).
    // Callers MUST NOT assume ciphertext integrity when mac_trit_count=0.
    let mac_hex = if mac_tc > 0 {
        base.absorb_bytes(&h1);   // 40 trits — tiny
        base.absorb(&ct1);        // cipher trits direct — NO binary round-trip
        base.absorb_bytes(&h2);   // 40 trits — tiny
        base.absorb(&ct2);        // cipher trits direct — NO binary round-trip
        trits_to_hex(&base.squeeze(mac_tc as usize))
    } else {
        // Bulk confidentiality mode: no T-AE-MAC tag. Integrity deferred to outer TL-DSA.
        String::new()
    };

    // Pack for transport — cipher trits to bytes happens here ONLY, for the wire format
    let cb1=cipher_trits_to_bytes(&ct1);
    let cb2=cipher_trits_to_bytes(&ct2);
    let fc1=[&h1[..],&cb1[..]].concat();
    let fc2=[&h2[..],&cb2[..]].concat();
    let mut out=Vec::with_capacity(4+fc1.len()+4+fc2.len()+mac_hex.len());
    out.extend_from_slice(&(fc1.len()as u32).to_le_bytes());out.extend_from_slice(&fc1);
    out.extend_from_slice(&(fc2.len()as u32).to_le_bytes());out.extend_from_slice(&fc2);
    out.extend_from_slice(mac_hex.as_bytes());
    Ok(Buffer::from(out))
}

// ---- Full phase decrypt (dual-phase) ----

#[napi]
pub fn phase_duplex_decrypt(
    domain_input:Buffer,primary_cipher_raw:Buffer,switch_marker:Buffer,
    secondary_cipher_raw:Buffer,expected_mac_hex:String,mac_trit_count:u32,sponge_version:u32,
)->napi::Result<Buffer>{
    if mac_trit_count>MAX_TRIT_COUNT{return Err(napi::Error::from_reason("mac_trit_count too large".to_string()));}
    if sponge_version>=3{return phase_decrypt_v3(domain_input.as_ref(),primary_cipher_raw.as_ref(),
        switch_marker.as_ref(),secondary_cipher_raw.as_ref(),&expected_mac_hex,mac_trit_count);}
    phase_decrypt_sequential(domain_input.as_ref(),primary_cipher_raw.as_ref(),
        switch_marker.as_ref(),secondary_cipher_raw.as_ref(),&expected_mac_hex,mac_trit_count,sponge_version)
}

fn phase_decrypt_sequential(di:&[u8],r1:&[u8],sw:&[u8],r2:&[u8],exp_mac:&str,mac_tc:u32,sv:u32)->napi::Result<Buffer>{
    if r1.len()<8||r2.len()<8{return Err(napi::Error::from_reason("cipher too short".to_string()));}
    let obl1=u32::from_be_bytes([r1[0],r1[1],r1[2],r1[3]])as usize;
    let tc1=u32::from_be_bytes([r1[4],r1[5],r1[6],r1[7]])as usize;let cb1=&r1[8..];
    let obl2=u32::from_be_bytes([r2[0],r2[1],r2[2],r2[3]])as usize;
    let tc2=u32::from_be_bytes([r2[4],r2[5],r2[6],r2[7]])as usize;let cb2=&r2[8..];
    let ct1=cipher_bytes_to_trits(cb1,tc1);let ct2=cipher_bytes_to_trits(cb2,tc2);
    let mut s=if sv>=2{Sponge385Pub::new()}else{Sponge385Pub::new_v1()};
    s.absorb_bytes(di);let ks1=s.squeeze(tc1);
    s.absorb_bytes(sw);let ks2=s.squeeze(tc2);
    s.absorb_bytes(&r1[0..8]);s.absorb_bytes(cb1);s.absorb_bytes(&r2[0..8]);s.absorb_bytes(cb2);
    let cm=trits_to_hex(&s.squeeze(mac_tc as usize));
    if cm.len()!=exp_mac.len(){return Ok(Buffer::from(vec![]));}
    let(a,b)=(cm.as_bytes(),exp_mac.as_bytes());
    let mut d:u8=0;for i in 0..a.len(){d|=a[i]^b[i];}if d!=0{return Ok(Buffer::from(vec![]));}
    let pl1:Vec<i8>=ct1.iter().zip(ks1.iter()).map(|(&c,&k)|trit_sub(c,k)).collect();
    let pl2:Vec<i8>=ct2.iter().zip(ks2.iter()).map(|(&c,&k)|trit_sub(c,k)).collect();
    let pb1=balanced_trits6_to_bytes(&pl1,obl1);let pb2=balanced_trits6_to_bytes(&pl2,obl2);
    let mut out=Vec::with_capacity(8+pb1.len()+pb2.len());
    out.extend_from_slice(&(pb1.len()as u32).to_le_bytes());out.extend_from_slice(&pb1);
    out.extend_from_slice(&(pb2.len()as u32).to_le_bytes());out.extend_from_slice(&pb2);
    Ok(Buffer::from(out))
}

/// v3 T-AE-MAC decrypt: verify tag on trits (dual-phase, base never squeezed keystream), then decrypt
///
/// T-AE-MAC verification flow:
///   1. base sponge absorbs domain
///   2. Clone base BEFORE MAC (keystream needs clean post-domain state)
///   3. T-AE-MAC verify (if mac_tc > 0): base absorbs headers + cipher TRITS
///      directly, squeezes tag, constant-time compare
///   4. Keystream from pre-MAC clone (bulk rate), GF(3) decrypt
fn phase_decrypt_v3(di:&[u8],r1:&[u8],sw:&[u8],r2:&[u8],exp_mac:&str,mac_tc:u32)->napi::Result<Buffer>{
    if r1.len()<8||r2.len()<8{return Err(napi::Error::from_reason("cipher too short".to_string()));}
    let obl1=u32::from_be_bytes([r1[0],r1[1],r1[2],r1[3]])as usize;
    let tc1=u32::from_be_bytes([r1[4],r1[5],r1[6],r1[7]])as usize;let cb1=&r1[8..];
    let obl2=u32::from_be_bytes([r2[0],r2[1],r2[2],r2[3]])as usize;
    let tc2=u32::from_be_bytes([r2[4],r2[5],r2[6],r2[7]])as usize;let cb2=&r2[8..];

    // Unpack cipher trits from transport bytes — this is the ONE binary→ternary step
    // (unavoidable: wire format is bytes)
    let ct1=cipher_bytes_to_trits(cb1,tc1);
    let ct2=cipher_bytes_to_trits(cb2,tc2);

    // Base sponge absorbs domain
    let mut base=Sponge385Pub::new();
    base.absorb_bytes(di);

    // Clone base BEFORE MAC (MAC absorb is destructive, keystream needs clean state)
    let ks_base=base.clone();

    // T-AE-MAC verification: dual-phase: base (never squeezed keystream), absorbing TRITS directly.
    // When mac_tc=0: BULK CONFIDENTIALITY MODE — no tag verification.
    // Caller asserts integrity was verified externally (TL-DSA signature).
    // Ciphertext is decrypted WITHOUT authentication. Use with caution.
    if mac_tc>0{
        base.absorb_bytes(&r1[0..8]);  // header1 — 40 trits, tiny
        base.absorb(&ct1);              // cipher trits direct — no binary round-trip
        base.absorb_bytes(&r2[0..8]);  // header2 — 40 trits, tiny
        base.absorb(&ct2);              // cipher trits direct — no binary round-trip
        let cm=trits_to_hex(&base.squeeze(mac_tc as usize));
        if cm.len()!=exp_mac.len(){return Ok(Buffer::from(vec![]));}
        let(a,b)=(cm.as_bytes(),exp_mac.as_bytes());
        let mut d:u8=0;for i in 0..a.len(){d|=a[i]^b[i];}
        if d!=0{return Ok(Buffer::from(vec![]));}
    }

    // Keystream from pre-MAC clone (bulk rate)
    let mut sec_tag=Vec::with_capacity(1+sw.len());sec_tag.push(0x01);sec_tag.extend_from_slice(sw);

    let use_parallel=rayon::current_num_threads()>1
        &&(tc1>=PARALLEL_THRESHOLD||tc2>=PARALLEL_THRESHOLD);

    let(ks1,ks2)=if use_parallel{
        rayon::join(
            ||generate_keystream_v3(&ks_base,tc1,&[0x00]),
            ||generate_keystream_v3(&ks_base,tc2,&sec_tag))
    }else{
        (generate_keystream_v3(&ks_base,tc1,&[0x00]),
         generate_keystream_v3(&ks_base,tc2,&sec_tag))
    };

    // GF(3) decrypt
    let(pl1,pl2)=if use_parallel{
        rayon::join(||cipher_trits(&ct1,&ks1,false),||cipher_trits(&ct2,&ks2,false))
    }else{
        (cipher_trits(&ct1,&ks1,false),cipher_trits(&ct2,&ks2,false))
    };

    let pb1=balanced_trits6_to_bytes(&pl1,obl1);let pb2=balanced_trits6_to_bytes(&pl2,obl2);
    let mut out=Vec::with_capacity(8+pb1.len()+pb2.len());
    out.extend_from_slice(&(pb1.len()as u32).to_le_bytes());out.extend_from_slice(&pb1);
    out.extend_from_slice(&(pb2.len()as u32).to_le_bytes());out.extend_from_slice(&pb2);
    Ok(Buffer::from(out))
}

// ---- TL-DSA Signature Operations (T-03, SPEC-2026-NEXT) ----

/// Generate a TL-DSA keypair.
///
/// Returns a buffer containing: [pk_len (4 LE) | pk | sk_len (4 LE) | sk]
///
/// @param variant - 44, 65, or 87
/// @param seed - Optional seed bytes (must have ≥256 bits entropy in production)
#[napi]
pub fn tl_dsa_keygen(variant: u32, seed: Option<Buffer>) -> napi::Result<Buffer> {
    let v = tl_dsa::TlDsaVariant::from_u32(variant).ok_or_else(|| {
        napi::Error::from_reason(format!(
            "Unknown TL-DSA variant {}. Use 44, 65, or 87.",
            variant
        ).to_string())
    })?;

    let seed_bytes = seed.as_ref().map(|b| b.as_ref());
    let kp = tl_dsa::keygen(v, seed_bytes);

    let mut out = Vec::with_capacity(4 + kp.public_key.len() + 4 + kp.secret_key.len());
    out.extend_from_slice(&(kp.public_key.len() as u32).to_le_bytes());
    out.extend_from_slice(&kp.public_key);
    out.extend_from_slice(&(kp.secret_key.len() as u32).to_le_bytes());
    out.extend_from_slice(&kp.secret_key);
    Ok(Buffer::from(out))
}

/// Sign a message with a TL-DSA secret key.
///
/// Returns the raw signature bytes.
#[napi]
pub fn tl_dsa_sign(secret_key: Buffer, message: Buffer, variant: u32) -> napi::Result<Buffer> {
    let v = tl_dsa::TlDsaVariant::from_u32(variant).ok_or_else(|| {
        napi::Error::from_reason(format!(
            "Unknown TL-DSA variant {}. Use 44, 65, or 87.",
            variant
        ).to_string())
    })?;

    let sig = tl_dsa::sign(secret_key.as_ref(), message.as_ref(), v);
    Ok(Buffer::from(sig))
}

/// Verify a TL-DSA signature using ONLY the public key.
///
/// **This function does NOT require the secret key.**
#[napi]
pub fn tl_dsa_verify(
    public_key: Buffer,
    message: Buffer,
    signature: Buffer,
    variant: u32,
) -> napi::Result<bool> {
    let v = tl_dsa::TlDsaVariant::from_u32(variant).ok_or_else(|| {
        napi::Error::from_reason(format!(
            "Unknown TL-DSA variant {}. Use 44, 65, or 87.",
            variant
        ).to_string())
    })?;

    Ok(tl_dsa::verify(
        public_key.as_ref(),
        message.as_ref(),
        signature.as_ref(),
        v,
    ))
}

/// Get the signature size for a TL-DSA variant.
#[napi]
pub fn tl_dsa_sig_len(variant: u32) -> napi::Result<u32> {
    let v = tl_dsa::TlDsaVariant::from_u32(variant).ok_or_else(|| {
        napi::Error::from_reason(format!(
            "Unknown TL-DSA variant {}. Use 44, 65, or 87.",
            variant
        ).to_string())
    })?;
    Ok(tl_dsa::sig_len(v) as u32)
}
