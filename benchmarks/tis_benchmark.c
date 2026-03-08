/*
 * TIS-27 Extended Theta V3 — SIMD 7-neighbor, 4 rounds
 * The state is 54 bytes. That's less than one cache line.
 * Building shifted copies is two memcpy calls of <54 bytes each.
 * Then SIMD adds and mod3 — same as the standard path but fewer rounds.
 *
 * gcc -O2 -march=native -msse2 -o tis_ext3 tis_ext3.c -lcrypto
 */
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <time.h>
#include <immintrin.h>
#ifndef NO_OPENSSL
#include <openssl/sha.h>
#endif

static inline double now_ns(void){struct timespec ts;clock_gettime(CLOCK_MONOTONIC,&ts);return ts.tv_sec*1e9+ts.tv_nsec;}
volatile uint64_t sink=0;

#define SW 54
#define SR 27
#define PAD 64

static const uint8_t PI[54]={
    0,13,26,39,52,11,24,37,50,9,22,35,48,7,20,33,46,5,
    18,31,44,3,16,29,42,1,14,27,40,53,12,25,38,51,10,23,
    36,49,8,21,34,47,6,19,32,45,4,17,30,43,2,15,28,41
};
static const uint8_t RC[27]={0,0,1,1,2,1,1,1,0,2,0,2,1,0,0,1,1,2,1,1,1,0,2,0,2,1,0};
static uint8_t RCS4[4][32] __attribute__((aligned(16)));
static uint8_t RCS13[13][32] __attribute__((aligned(16)));

static void init(void){
    memset(RCS4,0,sizeof(RCS4)); memset(RCS13,0,sizeof(RCS13));
    for(int r=0;r<4;r++) for(int i=0;i<SR;i++){int x=i+r; RCS4[r][i]=RC[x>=27?x-27:x];}
    for(int r=0;r<13;r++) for(int i=0;i<SR;i++){int x=i+r; RCS13[r][i]=RC[x>=27?x-27:x];}
}

static inline __m128i smod3(__m128i v){
    __m128i three=_mm_set1_epi8(3),two=_mm_set1_epi8(2);
    __m128i m=_mm_cmpgt_epi8(v,two); v=_mm_sub_epi8(v,_mm_and_si128(m,three));
    m=_mm_cmpgt_epi8(v,two); v=_mm_sub_epi8(v,_mm_and_si128(m,three));
    return v;
}

/*
 * Build a circular left-rotation of the state by `dist` positions.
 * rotate_left(s, out, d): out[i] = s[(i + d) % 54]
 * Two memcpy calls, no loops, no modular arithmetic.
 */
static inline void rotate_left(const uint8_t *s, uint8_t *out, int d) {
    memcpy(out, s + d, SW - d);
    memcpy(out + SW - d, s, d);
    memset(out + SW, 0, PAD - SW);
}

/*
 * SIMD extended theta: 7 neighbors at distances ±1, ±7, ±13
 * Add in two groups of 3 (range [0,6] each), mod3, then combine.
 */
static void theta_ext_simd(const uint8_t *s, uint8_t *out) {
    uint8_t __attribute__((aligned(16))) L13[PAD], L7[PAD], L1[PAD];
    uint8_t __attribute__((aligned(16))) R1[PAD], R7[PAD], R13[PAD];

    rotate_left(s, L13, 13);  /* s[(i+13) % 54] = left neighbor at distance 13 */
    rotate_left(s, L7, 7);
    rotate_left(s, L1, 1);
    rotate_left(s, R1, 53);   /* s[(i+53) % 54] = s[(i-1) % 54] */
    rotate_left(s, R7, 47);   /* s[(i-7) % 54] */
    rotate_left(s, R13, 41);  /* s[(i-13) % 54] */

    for (int i = 0; i < PAD; i += 16) {
        /* Left group: (L13 + L7 + L1) mod 3 */
        __m128i lg = _mm_add_epi8(
            _mm_add_epi8(
                _mm_load_si128((__m128i*)(R13+i)),   /* i-13 */
                _mm_load_si128((__m128i*)(R7+i))),   /* i-7 */
            _mm_load_si128((__m128i*)(R1+i)));       /* i-1 */
        lg = smod3(lg);

        /* Right group: (R1 + R7 + R13) mod 3 */
        __m128i rg = _mm_add_epi8(
            _mm_add_epi8(
                _mm_load_si128((__m128i*)(L1+i)),    /* i+1 */
                _mm_load_si128((__m128i*)(L7+i))),   /* i+7 */
            _mm_load_si128((__m128i*)(L13+i)));      /* i+13 */
        rg = smod3(rg);

        /* Combine: (left_group + center + right_group) mod 3 */
        __m128i center = _mm_load_si128((__m128i*)(s+i));
        __m128i total = _mm_add_epi8(_mm_add_epi8(lg, center), rg);
        _mm_store_si128((__m128i*)(out+i), smod3(total));
    }
}

/* Standard 3-neighbor SIMD theta for comparison */
static void theta_std_simd(const uint8_t *s, uint8_t *out) {
    uint8_t __attribute__((aligned(16))) left[PAD], right[PAD];
    rotate_left(s, right, 1);   /* s[i+1] */
    rotate_left(s, left, 53);   /* s[i-1] */

    for (int i = 0; i < PAD; i += 16) {
        __m128i vl = _mm_load_si128((__m128i*)(left+i));
        __m128i vc = _mm_load_si128((__m128i*)(s+i));
        __m128i vr = _mm_load_si128((__m128i*)(right+i));
        _mm_store_si128((__m128i*)(out+i),
            smod3(_mm_add_epi8(_mm_add_epi8(vl,vc),vr)));
    }
}

/* Pi + RC as before */
static void pi_rc(const uint8_t *theta, uint8_t *s, const uint8_t *rc) {
    uint8_t __attribute__((aligned(16))) pi_out[PAD];
    memset(pi_out, 0, PAD);
    for (int i = 0; i < SW; i++) pi_out[i] = theta[PI[i]];

    __m128i s0=_mm_load_si128((__m128i*)pi_out);
    __m128i r0=_mm_load_si128((__m128i*)rc);
    _mm_store_si128((__m128i*)pi_out, smod3(_mm_add_epi8(s0,r0)));
    __m128i s1=_mm_load_si128((__m128i*)(pi_out+16));
    __m128i r1=_mm_load_si128((__m128i*)(rc+16));
    _mm_store_si128((__m128i*)(pi_out+16), smod3(_mm_add_epi8(s1,r1)));

    memcpy(s, pi_out, SW);
    memset(s+SW, 0, PAD-SW);
}

/* Extended: 4 rounds, 7-neighbor SIMD theta */
static void tis27_ext(const uint8_t in[27], uint8_t out[27]) {
    uint8_t __attribute__((aligned(16))) s[PAD];
    uint8_t __attribute__((aligned(16))) t[PAD];
    memset(s,0,PAD); memcpy(s,in,SR);
    for (int r = 0; r < 4; r++) {
        theta_ext_simd(s, t);
        pi_rc(t, s, RCS4[r]);
    }
    memcpy(out, s, SR);
}

/* Standard: 13 rounds, 3-neighbor SIMD theta */
static void tis27_std(const uint8_t in[27], uint8_t out[27]) {
    uint8_t __attribute__((aligned(16))) s[PAD];
    uint8_t __attribute__((aligned(16))) t[PAD];
    memset(s,0,PAD); memcpy(s,in,SR);
    for (int r = 0; r < 13; r++) {
        theta_std_simd(s, t);
        pi_rc(t, s, RCS13[r]);
    }
    memcpy(out, s, SR);
}

/* Avalanche */
static void avalanche(void (*fn)(const uint8_t[27],uint8_t[27]), const char *name){
    int total=0,tests=0;
    for(int p=0;p<27;p++) for(int v=1;v<=2;v++){
        uint8_t a[27]={0},b[27]={0},ha[27],hb[27]; b[p]=v;
        fn(a,ha); fn(b,hb);
        int d=0; for(int i=0;i<27;i++) if(ha[i]!=hb[i])d++;
        total+=d; tests++;
    }
    double avg=(double)total/tests;
    printf("  %s: avg %.1f/27 (%.0f%%) %s\n",name,avg,100*avg/27,avg>=12?"✓":"✗ WEAK");
}

#define ITERS 2000000

int main(void){
    init();

    uint8_t in27[27]; for(int i=0;i<27;i++) in27[i]=i%3;
    uint8_t os[27],oe[27];
    tis27_std(in27,os); tis27_ext(in27,oe);

    int ok=1; for(int i=0;i<27;i++) if(oe[i]>2)ok=0;
    printf("GF(3) range: %s\n\n",ok?"✓":"✗");

    printf("Full avalanche (54 single-trit flips):\n");
    avalanche(tis27_std,"Standard (13r, 3-nbr SIMD)");
    avalanche(tis27_ext,"Extended (4r, 7-nbr SIMD) ");
    printf("\n");

    for(int w=0;w<ITERS/5;w++){tis27_std(in27,os);tis27_ext(in27,oe);sink+=os[0]+oe[0];}

    double s,e,ns;
    printf("╔═══════════════════════════════════════════════════════════════════╗\n");
    printf("║  TIS-27 EXTENDED THETA V3 — SIMD BOTH PATHS — %d iters    ║\n",ITERS);
    printf("╠═══════════════════════════════════════════════════════════════════╣\n");

    s=now_ns();for(int i=0;i<ITERS;i++){tis27_std(in27,os);sink+=os[0];}e=now_ns();ns=(e-s)/ITERS;
    printf("║  Standard (13r, 3-nbr SIMD):     %6.0f ns  %7.0fK/s            ║\n",ns,1e6/ns);
    double std_ns=ns;

    s=now_ns();for(int i=0;i<ITERS;i++){tis27_ext(in27,oe);sink+=oe[0];}e=now_ns();ns=(e-s)/ITERS;
    printf("║  Extended (4r, 7-nbr SIMD):      %6.0f ns  %7.0fK/s            ║\n",ns,1e6/ns);
    double ext_ns=ns;

#ifndef NO_OPENSSL
    uint8_t sha_in[27],sha_out[32]; memcpy(sha_in,in27,27);
    s=now_ns();for(int i=0;i<ITERS;i++){SHA256(sha_in,27,sha_out);sink+=sha_out[0];}e=now_ns();ns=(e-s)/ITERS;
    printf("║  SHA-256 (OpenSSL asm):          %6.0f ns  %7.0fK/s            ║\n",ns,1e6/ns);
    double sha_ns=ns;

    printf("╠═══════════════════════════════════════════════════════════════════╣\n");
    printf("║  Std→Ext speedup:       %.1fx                                    ║\n",std_ns/ext_ns);
    if(ext_ns<sha_ns)
        printf("║  Extended vs SHA-256:   %.2fx FASTER ✓                           ║\n",sha_ns/ext_ns);
    else
        printf("║  Extended vs SHA-256:   %.2fx slower                              ║\n",ext_ns/sha_ns);
    printf("║  Ext: %.1f MB/s | Std: %.1f MB/s | SHA: %.1f MB/s                ║\n",
        27e3/ext_ns, 27e3/std_ns, 27e3/sha_ns);
#endif
    printf("╚═══════════════════════════════════════════════════════════════════╝\n");
    return 0;
}
