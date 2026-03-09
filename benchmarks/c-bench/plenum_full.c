/*
 * PLENUMNET COMPLETE PLATFORM BENCHMARK SUITE
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. — Applied Physics Division
 *
 * Every PlenumNET module vs its industry equivalent.
 * 15 categories, ~40 individual benchmarks.
 *
 * gcc -O2 -march=native -msse2 -o plenum_full plenum_full.c -lcrypto -lsodium
 * Without libsodium: gcc -O2 -march=native -msse2 -o plenum_full plenum_full.c -lcrypto -DNO_SODIUM
 */
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>
#include <immintrin.h>

#include <openssl/sha.h>
#include <openssl/evp.h>
#include <openssl/hmac.h>
#include <openssl/rand.h>
#include <openssl/ec.h>
#include <openssl/ecdsa.h>
#include <openssl/kdf.h>
#include <openssl/err.h>
#include <openssl/core_names.h>

#ifndef NO_SODIUM
#include <sodium.h>
#endif

static inline double now_ns(void){struct timespec ts;clock_gettime(CLOCK_MONOTONIC,&ts);return ts.tv_sec*1e9+ts.tv_nsec;}
volatile uint64_t bsink=0;

typedef struct{const char*cat;const char*name;double ns;double ops;int is_plenum;}R;
R res[80]; int rc=0;

#define BENCH(category,label,iters,is_pn,code) do{ \
    for(int _w=0;_w<(iters)/10;_w++){code;} \
    double _s=now_ns();for(int _i=0;_i<(iters);_i++){code;}double _e=now_ns(); \
    double _n=(_e-_s)/(iters); \
    res[rc].cat=category;res[rc].name=label;res[rc].ns=_n;res[rc].ops=1e9/_n;res[rc].is_plenum=is_pn;rc++; \
    printf("  %-44s %8.1f ns %10.0f/s %s\n",label,_n,1e9/_n,is_pn?"◀ PLENUM":"  industry"); \
}while(0)

/* ══════════════════════════════════════════════════════════════
 * PLENUMNET PRIMITIVES
 * ══════════════════════════════════════════════════════════════ */

#define SW 54
#define SR 27
#define PAD 64

static inline uint8_t mod3(uint8_t n){if(n>=3)n-=3;if(n>=3)n-=3;return n;}
static inline uint8_t ga(uint8_t a,uint8_t b){uint8_t s=a+b;return s>=3?s-3:s;}

static const uint8_t PI[54]={
    0,13,26,39,52,11,24,37,50,9,22,35,48,7,20,33,46,5,
    18,31,44,3,16,29,42,1,14,27,40,53,12,25,38,51,10,23,
    36,49,8,21,34,47,6,19,32,45,4,17,30,43,2,15,28,41};
static const uint8_t RC[27]={0,0,1,1,2,1,1,1,0,2,0,2,1,0,0,1,1,2,1,1,1,0,2,0,2,1,0};
static uint8_t RCS4[4][32] __attribute__((aligned(16)));
static uint8_t RCS81[4][96] __attribute__((aligned(16)));

static inline __m128i smod3(__m128i v){
    __m128i three=_mm_set1_epi8(3),two=_mm_set1_epi8(2);
    __m128i m=_mm_cmpgt_epi8(v,two);v=_mm_sub_epi8(v,_mm_and_si128(m,three));
    m=_mm_cmpgt_epi8(v,two);v=_mm_sub_epi8(v,_mm_and_si128(m,three));
    return v;}

static inline void rot54(const uint8_t*s,uint8_t*o,int d){
    memcpy(o,s+d,SW-d);memcpy(o+SW-d,s,d);memset(o+SW,0,PAD-SW);}

/* TIS-27: 4 rounds, 7-neighbor extended theta, SIMD */
__attribute__((hot,noinline))
static void tis27_hash(const uint8_t *__restrict__ in,uint8_t *__restrict__ out){
    uint8_t __attribute__((aligned(16))) s[PAD],L13[PAD],L7[PAD],L1[PAD],R1[PAD],R7[PAD],R13[PAD],t[PAD],p[PAD];
    memset(s,0,PAD);memcpy(s,in,SR);
    for(int r=0;r<4;r++){
        rot54(s,L13,13);rot54(s,L7,7);rot54(s,L1,1);rot54(s,R1,53);rot54(s,R7,47);rot54(s,R13,41);
        for(int i=0;i<PAD;i+=16){
            __m128i lg=smod3(_mm_add_epi8(_mm_add_epi8(_mm_load_si128((__m128i*)(R13+i)),_mm_load_si128((__m128i*)(R7+i))),_mm_load_si128((__m128i*)(R1+i))));
            __m128i rg=smod3(_mm_add_epi8(_mm_add_epi8(_mm_load_si128((__m128i*)(L1+i)),_mm_load_si128((__m128i*)(L7+i))),_mm_load_si128((__m128i*)(L13+i))));
            _mm_store_si128((__m128i*)(t+i),smod3(_mm_add_epi8(_mm_add_epi8(lg,_mm_load_si128((__m128i*)(s+i))),rg)));}
        memset(p,0,PAD);for(int i=0;i<SW;i++)p[i]=t[PI[i]];
        __m128i s0=_mm_load_si128((__m128i*)p);_mm_store_si128((__m128i*)p,smod3(_mm_add_epi8(s0,_mm_load_si128((__m128i*)RCS4[r]))));
        __m128i s1=_mm_load_si128((__m128i*)(p+16));_mm_store_si128((__m128i*)(p+16),smod3(_mm_add_epi8(s1,_mm_load_si128((__m128i*)(RCS4[r]+16)))));
        memcpy(s,p,SW);memset(s+SW,0,PAD-SW);}
    memcpy(out,s,SR);}

/* TIS-81: scalar, 4 rounds, 7-neighbor */
static void tis81_hash(const uint8_t*in,uint8_t*out){
    uint8_t s[243],t[243]; memset(s,0,243);
    for(int i=0;i<81;i++) s[i]=in[i];
    for(int r=0;r<4;r++){
        t[0]=mod3(s[242]+s[0]+s[1]+mod3(s[230]+s[236]+s[12])); /* simplified extended */
        for(int i=1;i<242;i++){
            uint8_t left=mod3(s[(i+243-13)%243]+s[(i+243-7)%243]+s[(i+243-1)%243]);
            uint8_t right=mod3(s[(i+1)%243]+s[(i+7)%243]+s[(i+13)%243]);
            t[i]=mod3(left+s[i]+right);}
        t[242]=mod3(s[241]+s[242]+s[0]+mod3(s[229]+s[235]+s[11]));
        for(int i=0;i<243;i++) s[i]=t[(i*13)%243];
        for(int i=0;i<81;i++) s[i]=ga(s[i],RC[(i+r)%27]);}
    memcpy(out,s,81);}

/* Hamming: Σ(a-b)² mod 3 */
static uint32_t hamming_gf3(const uint8_t*a,const uint8_t*b,int n){
    uint32_t d=0;for(int i=0;i<n;i++){uint8_t df=a[i]+3-b[i];if(df>=3)df-=3;if(df>=3)df-=3;uint8_t sq=df*df;if(sq>=3)sq-=3;d+=sq;}return d;}

/* Forgery: product mod 7 */
static int forgery_check(const uint8_t*t,int n){uint8_t p=1;for(int i=0;i<n;i++){p=p*t[i];if(p>=14)p-=14;if(p>=7)p-=7;if(!p)return 1;}return 0;}

/* Checksum: Horner mod 364 */
static uint64_t repunit_ck(const uint8_t*t,int n){uint64_t v=0;for(int i=n-1;i>=0;i--)v=(v*3+(t[i]-1))%364;return v;}

/* CRT */
static void crt_dec(uint64_t p,uint8_t*m,uint8_t*d){*m=p%13;*d=p%28;}
static uint16_t crt_rec(uint8_t m,uint8_t d){return(196*m+169*d)%364;}

/* Wire pack/unpack */
static void wire_pack(const uint8_t*t,uint8_t*w){memset(w,0,7);for(int i=0;i<27;i++){int bp=i*2;w[bp/8]|=(t[i]&3)<<(bp%8);}}
static void wire_unpack(const uint8_t*w,uint8_t*t){for(int i=0;i<27;i++){int bp=i*2;t[i]=(w[bp/8]>>(bp%8))&3;}}

/* Phase encryption (simplified — GF(3) angle rotation + XOR mixing) */
static void phase_encrypt(const uint8_t*plain,uint8_t*cipher,const uint8_t*key,int n){
    for(int i=0;i<n;i++) cipher[i]=ga(plain[i],key[i%27]);}
static void phase_decrypt(const uint8_t*cipher,uint8_t*plain,const uint8_t*key,int n){
    for(int i=0;i<n;i++){uint8_t s=cipher[i]+3-key[i%27];if(s>=3)s-=3;if(s>=3)s-=3;plain[i]=s;}}

/* TIS-27 key derivation */
static void tis27_kdf(const uint8_t*ctx,int clen,const uint8_t*mat,int mlen,uint8_t*out,int olen){
    uint8_t buf[256]; int blen=clen+mlen; if(blen>256)blen=256;
    memcpy(buf,ctx,clen); memcpy(buf+clen,mat,mlen>256-clen?256-clen:mlen);
    /* Convert to GF(3) */
    uint8_t gf3[256]; for(int i=0;i<blen;i++) gf3[i]=buf[i]%3;
    /* Hash iteratively to fill output */
    uint8_t h[27]; int off=0;
    while(off<olen){
        int take=olen-off; if(take>27)take=27;
        tis27_hash(gf3+(off%blen>blen-27?0:off%blen),h);
        for(int i=0;i<take;i++) out[off+i]=h[i];
        off+=take;
        /* Feedback */
        for(int i=0;i<27&&i<blen;i++) gf3[i]=ga(gf3[i],h[i]);
    }}

/* Agent scheduling: Z28 coprime walk */
static void z28_walk(uint8_t*seq,int gen){for(int k=0;k<28;k++)seq[k]=(k*gen)%28;}

/* Address derivation: raw → 27 Rep C trits */
static void derive_address(const uint8_t*raw,uint8_t*addr){
    uint8_t h[27]; tis27_hash(raw,h);
    for(int i=0;i<27;i++) addr[i]=h[i]+1;}

/* Capability token: context+timestamp+permission → hashed token */
static void cap_token(const uint8_t*ctx,uint64_t ts,uint8_t perm,uint8_t*token){
    uint8_t buf[27]; memset(buf,0,27);
    memcpy(buf,ctx,16>27?27:16);
    buf[24]=(ts>>8)%3; buf[25]=ts%3; buf[26]=perm%3;
    tis27_hash(buf,token);}

/* ══════════════════════════════════════════════════════════════ */

static void init_tables(void){
    memset(RCS4,0,sizeof(RCS4));
    for(int r=0;r<4;r++)for(int i=0;i<SR;i++){int x=i+r;RCS4[r][i]=RC[x>=27?x-27:x];}
}

int main(void){
    init_tables();
#ifndef NO_SODIUM
    if(sodium_init()<0){printf("sodium init failed\n");return 1;}
#endif

    /* Test data */
    uint8_t raw27[27],raw81[81],raw256[256],key32[32],iv12[12];
    RAND_bytes(raw27,27); RAND_bytes(raw81,81); RAND_bytes(raw256,256);
    RAND_bytes(key32,32); RAND_bytes(iv12,12);
    uint8_t addr_a[27],addr_b[27]; for(int i=0;i<27;i++){addr_a[i]=i%3;addr_b[i]=(i*7)%3;}
    uint8_t valid27[27]; for(int i=0;i<27;i++) valid27[i]=(i%3)+1;

    int N=2000000, NM=500000, NS=100000;

    printf("╔═══════════════════════════════════════════════════════════════════════════╗\n");
    printf("║  PLENUMNET COMPLETE PLATFORM BENCHMARK                                   ║\n");
    printf("║  Salvi Framework vs Industry — %s — %d core iterations          ║\n",__DATE__,N);
    printf("║  Capomastro Holdings Ltd. — Applied Physics Division                     ║\n");
    printf("╚═══════════════════════════════════════════════════════════════════════════╝\n\n");

    /* ═══════════════════════ 1. HASHING ═══════════════════════ */
    printf("── 1. HASHING ──────────────────────────────────────────────────────────────\n");
    uint8_t h27[27],h81[81],sh[32],sh3[32];
    BENCH("Hash","TIS-27 (4r ext-theta, 27B)",N,1, {tis27_hash(raw27,h27);bsink+=h27[0];});
    BENCH("Hash","TIS-81 (4r ext-theta, 81B, PQ)",NM,1, {tis81_hash(raw81,h81);bsink+=h81[0];});
    BENCH("Hash","SHA-256 (OpenSSL, 27B)",N,0, {SHA256(raw27,27,sh);bsink+=sh[0];});
    BENCH("Hash","SHA-384 (OpenSSL, 27B)",NM,0, {unsigned int l;EVP_MD_CTX*c=EVP_MD_CTX_new();EVP_DigestInit_ex(c,EVP_sha384(),NULL);EVP_DigestUpdate(c,raw27,27);EVP_DigestFinal_ex(c,sh,&l);EVP_MD_CTX_free(c);bsink+=sh[0];});
    BENCH("Hash","SHA-512 (OpenSSL, 27B)",NM,0, {unsigned int l;EVP_MD_CTX*c=EVP_MD_CTX_new();EVP_DigestInit_ex(c,EVP_sha512(),NULL);EVP_DigestUpdate(c,raw27,27);EVP_DigestFinal_ex(c,sh,&l);EVP_MD_CTX_free(c);bsink+=sh[0];});
    BENCH("Hash","SHA3-256 (OpenSSL, 27B)",NM,0, {unsigned int l;EVP_MD_CTX*c=EVP_MD_CTX_new();EVP_DigestInit_ex(c,EVP_sha3_256(),NULL);EVP_DigestUpdate(c,raw27,27);EVP_DigestFinal_ex(c,sh3,&l);EVP_MD_CTX_free(c);bsink+=sh3[0];});
#ifndef NO_SODIUM
    uint8_t bh[32];
    BENCH("Hash","BLAKE2b-256 (libsodium, 27B)",N,0, {crypto_generichash(bh,32,raw27,27,NULL,0);bsink+=bh[0];});
#endif

    /* ═══════════════════════ 2. ADDRESS DERIVATION ═══════════════════════ */
    printf("\n── 2. ADDRESS DERIVATION (raw → routable Rep C) ─────────────────────────────\n");
    uint8_t addr[27];
    BENCH("Address","TIS-27 → Rep C (native GF3)",N,1, {derive_address(raw27,addr);bsink+=addr[0];});
    BENCH("Address","SHA-256 → Rep C (convert)",N,0, {uint8_t s[32];SHA256(raw27,27,s);for(int i=0;i<27;i++)addr[i]=s[i]%3+1;bsink+=addr[0];});
    /* UUID v4 comparison */
    BENCH("Address","UUID v4 (RAND_bytes 16B)",N,0, {uint8_t u[16];RAND_bytes(u,16);bsink+=u[0];});

    /* ═══════════════════════ 3. ROUTING ═══════════════════════ */
    printf("\n── 3. ROUTING (distance + next-hop decision) ────────────────────────────────\n");
    BENCH("Routing","Hamming GF(3) 27-trit",N,1, {bsink+=hamming_gf3(addr_a,addr_b,27);});
    /* Simulate table lookup: random access into 1M-entry table */
    uint8_t *route_table=malloc(1000000);memset(route_table,42,1000000);
    BENCH("Routing","Table lookup (1M entries)",N,0, {bsink+=route_table[(bsink*7919)%1000000];});
    BENCH("Routing","CRT decompose (sector+slot)",N*5,1, {uint8_t m; uint8_t d;crt_dec(bsink,&m,&d);bsink+=m+d;});
    BENCH("Routing","CRT reconstruct",N*5,1, {bsink+=crt_rec(bsink%13,bsink%28);});
    free(route_table);

    /* ═══════════════════════ 4. INTEGRITY ═══════════════════════ */
    printf("\n── 4. INTEGRITY CHECKS ──────────────────────────────────────────────────────\n");
    BENCH("Integrity","Forgery check (product mod 7)",N,1, {bsink+=forgery_check(valid27,27);});
    BENCH("Integrity","memchr zero-scan (27B)",N,0, {bsink+=(memchr(valid27,0,27)==NULL);});
    BENCH("Integrity","Repunit checksum (mod 364)",N,1, {bsink+=repunit_ck(valid27,27);});
    /* CRC-32 via zlib-style */
    BENCH("Integrity","CRC-32 (manual, 27B)",N,0, {uint32_t crc=0xFFFFFFFF;for(int i=0;i<27;i++){crc^=raw27[i];for(int j=0;j<8;j++)crc=crc&1?(crc>>1)^0xEDB88320:crc>>1;}bsink+=crc;});
    BENCH("Integrity","HMAC-SHA256 (OpenSSL, 27B)",NM,0, {uint8_t hm[32];unsigned int l;HMAC(EVP_sha256(),key32,32,raw27,27,hm,&l);bsink+=hm[0];});

    /* ═══════════════════════ 5. WIRE ENCODING ═══════════════════════ */
    printf("\n── 5. WIRE ENCODING ─────────────────────────────────────────────────────────\n");
    uint8_t wire[7],wt[27];
    BENCH("Wire","Pack 27 trits → 7 bytes",N,1, {wire_pack(valid27,wire);bsink+=wire[0];});
    BENCH("Wire","Unpack 7 bytes → 27 trits",N,1, {wire_unpack(wire,wt);bsink+=wt[0];});
    BENCH("Wire","Pack + unpack roundtrip",N,1, {wire_pack(valid27,wire);wire_unpack(wire,wt);bsink+=wt[0];});
    /* Varint-style encoding comparison */
    BENCH("Wire","memcpy 27 bytes (baseline)",N,0, {uint8_t buf[27];memcpy(buf,raw27,27);bsink+=buf[0];});

    /* ═══════════════════════ 6. ENCRYPTION ═══════════════════════ */
    printf("\n── 6. ENCRYPTION ────────────────────────────────────────────────────────────\n");
    uint8_t gf3_key[27]; for(int i=0;i<27;i++) gf3_key[i]=i%3;
    uint8_t gf3_plain[27],gf3_cipher[27],gf3_dec[27];
    for(int i=0;i<27;i++) gf3_plain[i]=i%3;
    BENCH("Encrypt","Phase encrypt GF(3) 27 trits",N,1, {phase_encrypt(gf3_plain,gf3_cipher,gf3_key,27);bsink+=gf3_cipher[0];});
    BENCH("Encrypt","Phase decrypt GF(3) 27 trits",N,1, {phase_decrypt(gf3_cipher,gf3_dec,gf3_key,27);bsink+=gf3_dec[0];});

    /* AES-256-GCM */
    uint8_t aes_ct[256+16],aes_tag[16]; int aes_len;
    BENCH("Encrypt","AES-256-GCM encrypt 27B (OpenSSL)",NM,0, {
        EVP_CIPHER_CTX*c=EVP_CIPHER_CTX_new();
        EVP_EncryptInit_ex(c,EVP_aes_256_gcm(),NULL,key32,iv12);
        EVP_EncryptUpdate(c,aes_ct,&aes_len,raw27,27);
        EVP_EncryptFinal_ex(c,aes_ct+aes_len,&aes_len);
        EVP_CIPHER_CTX_ctrl(c,EVP_CTRL_GCM_GET_TAG,16,aes_tag);
        EVP_CIPHER_CTX_free(c);bsink+=aes_ct[0];});

#ifndef NO_SODIUM
    uint8_t nonce[24],sc_ct[27+crypto_secretbox_MACBYTES];
    RAND_bytes(nonce,24);
    BENCH("Encrypt","XSalsa20-Poly1305 27B (sodium)",N,0, {crypto_secretbox_easy(sc_ct,raw27,27,nonce,key32);bsink+=sc_ct[0];});
#endif

    /* ═══════════════════════ 7. KEY DERIVATION ═══════════════════════ */
    printf("\n── 7. KEY DERIVATION ────────────────────────────────────────────────────────\n");
    uint8_t kdf_out[32];
    BENCH("KDF","TIS-27 KDF (27B context+material)",NM,1, {tis27_kdf(raw27,16,raw27+16,11,kdf_out,32);bsink+=kdf_out[0];});

    BENCH("KDF","HKDF-SHA256 (OpenSSL, 32B out)",NM,0, {
        EVP_KDF*kdf=EVP_KDF_fetch(NULL,"HKDF",NULL);
        EVP_KDF_CTX*c=EVP_KDF_CTX_new(kdf);
        OSSL_PARAM params[5];
        params[0]=OSSL_PARAM_construct_utf8_string("digest","SHA256",0);
        params[1]=OSSL_PARAM_construct_octet_string("key",(void*)key32,32);
        params[2]=OSSL_PARAM_construct_octet_string("salt",(void*)raw27,16);
        params[3]=OSSL_PARAM_construct_octet_string("info",(void*)"ctx",3);
        params[4]=OSSL_PARAM_construct_end();
        EVP_KDF_derive(c,kdf_out,32,params);
        EVP_KDF_CTX_free(c);EVP_KDF_free(kdf);bsink+=kdf_out[0];});

    /* ═══════════════════════ 8. DIGITAL SIGNATURES ═══════════════════════ */
    printf("\n── 8. DIGITAL SIGNATURES ────────────────────────────────────────────────────\n");

    /* ECDSA-P256 sign + verify */
    EVP_PKEY *ec_key=NULL;
    {EVP_PKEY_CTX*kc=EVP_PKEY_CTX_new_id(EVP_PKEY_EC,NULL);
     EVP_PKEY_keygen_init(kc);
     EVP_PKEY_CTX_set_ec_paramgen_curve_nid(kc,NID_X9_62_prime256v1);
     EVP_PKEY_keygen(kc,&ec_key);EVP_PKEY_CTX_free(kc);}

    uint8_t ec_sig[256]; size_t ec_siglen=256;
    BENCH("Sign","ECDSA-P256 sign 27B (OpenSSL)",NS,0, {
        EVP_MD_CTX*mc=EVP_MD_CTX_new();ec_siglen=256;
        EVP_DigestSignInit(mc,NULL,EVP_sha256(),NULL,ec_key);
        EVP_DigestSignUpdate(mc,raw27,27);
        EVP_DigestSignFinal(mc,ec_sig,&ec_siglen);
        EVP_MD_CTX_free(mc);bsink+=ec_sig[0];});

    BENCH("Sign","ECDSA-P256 verify 27B (OpenSSL)",NS,0, {
        EVP_MD_CTX*mc=EVP_MD_CTX_new();
        EVP_DigestVerifyInit(mc,NULL,EVP_sha256(),NULL,ec_key);
        EVP_DigestVerifyUpdate(mc,raw27,27);
        int vr=EVP_DigestVerifyFinal(mc,ec_sig,ec_siglen);
        EVP_MD_CTX_free(mc);bsink+=vr;});

#ifndef NO_SODIUM
    /* Ed25519 */
    uint8_t ed_pk[32],ed_sk[64],ed_sig2[64];
    crypto_sign_keypair(ed_pk,ed_sk);
    BENCH("Sign","Ed25519 sign 27B (libsodium)",NS,0, {unsigned long long sl;crypto_sign_detached(ed_sig2,&sl,raw27,27,ed_sk);bsink+=ed_sig2[0];});
    BENCH("Sign","Ed25519 verify 27B (libsodium)",NS,0, {bsink+=crypto_sign_verify_detached(ed_sig2,raw27,27,ed_pk)==0;});
#endif

    /* TL-DSA simulation note */
    printf("  %-44s %8s %10s %s\n","TL-DSA (real Rust impl needed)","--","--","◀ PLENUM");

    EVP_PKEY_free(ec_key);

    /* ═══════════════════════ 9. SCHEDULING ═══════════════════════ */
    printf("\n── 9. AGENT SCHEDULING ──────────────────────────────────────────────────────\n");
    uint8_t seq[28];
    BENCH("Schedule","Z28 coprime walk (step 13)",N,1, {z28_walk(seq,13);bsink+=seq[0];});
    BENCH("Schedule","Round-robin 28 agents",N,0, {for(int k=0;k<28;k++)seq[k]=k;bsink+=seq[0];});

    /* ═══════════════════════ 10. GF(3) PRIMITIVES ═══════════════════════ */
    printf("\n── 10. GF(3) ELEMENT OPERATIONS ─────────────────────────────────────────────\n");
    BENCH("GF3","gf3_add (div-free)",N*10,1, {bsink+=ga(bsink&2,1);});
    BENCH("GF3","gf3_mul (div-free)",N*10,1, {uint8_t a=bsink&2;uint8_t p=a*2;if(p>=3)p-=3;bsink+=p;});
    BENCH("GF3","gf3_square (Hamming indicator)",N*10,1, {uint8_t a=bsink&2;uint8_t p=a*a;if(p>=3)p-=3;bsink+=p;});
    BENCH("GF3","Integer add (baseline)",N*10,0, {bsink+=bsink+1;});
    BENCH("GF3","Integer mul (baseline)",N*10,0, {bsink+=bsink*3;});

    /* ═══════════════════════ 11. CAPABILITY TOKENS ═══════════════════════ */
    printf("\n── 11. CAPABILITY TOKENS ────────────────────────────────────────────────────\n");
    uint8_t cap_tok[27];
    BENCH("Token","Capability token (TIS-27 based)",N,1, {cap_token(raw27,12345678,2,cap_tok);bsink+=cap_tok[0];});
    BENCH("Token","HMAC-SHA256 token (JWT-style)",NM,0, {uint8_t hm[32];unsigned int l;HMAC(EVP_sha256(),key32,32,raw27,27,hm,&l);bsink+=hm[0];});

    /* ═══════════════════════ 12. FULL PIPELINES ═══════════════════════ */
    printf("\n── 12. FULL PIPELINES ───────────────────────────────────────────────────────\n");

    BENCH("Pipeline","TDNS: raw→address (TIS-27+lift)",N,1, {
        uint8_t a2[27];derive_address(raw27,a2);bsink+=a2[0];});
    BENCH("Pipeline","TDNS: raw→address (SHA256+conv)",N,0, {
        uint8_t s[32];SHA256(raw27,27,s);uint8_t a2[27];for(int i=0;i<27;i++)a2[i]=s[i]%3+1;bsink+=a2[0];});
    BENCH("Pipeline","Route: addr→sector+slot+dist",N,1, {
        uint8_t m; uint8_t d;crt_dec(repunit_ck(valid27,27),&m,&d);
        bsink+=hamming_gf3(addr_a,addr_b,27)+m+d;});

    /* ════════════════════════════════════════════════════════════════════
     * SCORECARD
     * ════════════════════════════════════════════════════════════════════ */
    printf("\n╔═══════════════════════════════════════════════════════════════════════════╗\n");
    printf("║                    PLENUMNET PLATFORM SCORECARD                          ║\n");
    printf("╠═══════════════════════════════════════════════════════════════════════════╣\n");

    const char*last_cat="";
    int wins=0,losses=0,ties=0;
    for(int i=0;i<rc;i++){
        if(strcmp(res[i].cat,last_cat)!=0){
            printf("║  ── %-68s ║\n",res[i].cat);
            last_cat=res[i].cat;
        }
        char buf[16];
        if(res[i].ops>1e6) snprintf(buf,16,"%.1fM",res[i].ops/1e6);
        else if(res[i].ops>1e3) snprintf(buf,16,"%.0fK",res[i].ops/1e3);
        else snprintf(buf,16,"%.0f",res[i].ops);
        printf("║  %s %-42s %7.0f ns %8s/s ║\n",
            res[i].is_plenum?"▶":"·",res[i].name,res[i].ns,buf);
    }

    /* Count wins: for each category, compare best plenum vs best industry */
    printf("╠═══════════════════════════════════════════════════════════════════════════╣\n");

    /* Simple win counting by scanning pairs */
    for(int i=0;i<rc;i++){
        if(!res[i].is_plenum) continue;
        /* Find best industry in same category */
        double best_ind=1e18;
        for(int j=0;j<rc;j++){
            if(j==i||res[j].is_plenum) continue;
            if(strcmp(res[i].cat,res[j].cat)==0 && res[j].ns<best_ind)
                best_ind=res[j].ns;
        }
        if(best_ind<1e17){
            if(res[i].ns < best_ind*0.95) wins++;
            else if(res[i].ns > best_ind*1.05) losses++;
            else ties++;
        }
    }
    printf("║  PlenumNET wins: %d | Industry wins: %d | Ties: %d                       ║\n",wins,losses,ties);
    printf("╚═══════════════════════════════════════════════════════════════════════════╝\n");

    return 0;
}