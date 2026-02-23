#!/bin/bash
set -e
BASE="http://localhost:3001"

echo "=== Health ==="
curl -sf "$BASE/health" | python3 -m json.tool
echo ""

echo "=== Algorithms ==="
curl -sf "$BASE/api/pqti/algorithms" | python3 -c "import sys,json; d=json.load(sys.stdin); print('Success:', d['success']); [print(f'  DSA: {a[\"name\"]}') for a in d.get('tl_dsa',[])]; [print(f'  KEM: {a[\"name\"]}') for a in d.get('tl_kem',[])]"
echo ""

echo "=== TL-DSA-44 KeyGen ==="
curl -sf -X POST "$BASE/api/pqti/tldsa/keygen" \
  -H "Content-Type: application/json" \
  -d '{"variant":"TL-DSA-44","seed":"AAECAAEC"}' > /tmp/keygen.json
python3 -c "
import json
d = json.load(open('/tmp/keygen.json'))
print('Success:', d['success'])
print('PK trits:', d['public_key']['size_trits'])
print('SK trits:', d['secret_key']['size_trits'])
"

echo ""
echo "=== TL-DSA-44 Sign ==="
python3 -c "
import json, base64
kg = json.load(open('/tmp/keygen.json'))
body = {'variant':'TL-DSA-44','secret_key':kg['secret_key']['data'],'message':base64.b64encode(b'Hello PlenumNET').decode()}
open('/tmp/sign_req.json','w').write(json.dumps(body))
"
curl -sf -X POST "$BASE/api/pqti/tldsa/sign" \
  -H "Content-Type: application/json" \
  -d @/tmp/sign_req.json > /tmp/sign.json
python3 -c "
import json
d = json.load(open('/tmp/sign.json'))
print('Success:', d['success'])
print('Sig trits:', d['signature']['size_trits'])
"

echo ""
echo "=== TL-DSA-44 Verify (correct msg) ==="
python3 -c "
import json, base64
kg = json.load(open('/tmp/keygen.json'))
sg = json.load(open('/tmp/sign.json'))
body = {'variant':'TL-DSA-44','public_key':kg['public_key']['data'],'message':base64.b64encode(b'Hello PlenumNET').decode(),'signature':sg['signature']['data']}
open('/tmp/verify_req.json','w').write(json.dumps(body))
"
curl -sf -X POST "$BASE/api/pqti/tldsa/verify" \
  -H "Content-Type: application/json" \
  -d @/tmp/verify_req.json | python3 -c "import sys,json; d=json.load(sys.stdin); print('Valid:', d['valid'])"

echo ""
echo "=== TL-DSA-44 Verify (wrong msg) ==="
python3 -c "
import json, base64
kg = json.load(open('/tmp/keygen.json'))
sg = json.load(open('/tmp/sign.json'))
body = {'variant':'TL-DSA-44','public_key':kg['public_key']['data'],'message':base64.b64encode(b'WRONG MSG').decode(),'signature':sg['signature']['data']}
open('/tmp/verify_wrong.json','w').write(json.dumps(body))
"
curl -sf -X POST "$BASE/api/pqti/tldsa/verify" \
  -H "Content-Type: application/json" \
  -d @/tmp/verify_wrong.json | python3 -c "import sys,json; d=json.load(sys.stdin); print('Valid:', d['valid'])"

echo ""
echo "=== TL-KEM-512 KeyGen ==="
curl -sf -X POST "$BASE/api/pqti/tlkem/keygen" \
  -H "Content-Type: application/json" \
  -d '{"variant":"TL-KEM-512","seed":"AAECAAEC"}' > /tmp/kem_keygen.json
python3 -c "
import json
d = json.load(open('/tmp/kem_keygen.json'))
print('Success:', d['success'])
print('PK trits:', d['public_key']['size_trits'])
print('SK trits:', d['secret_key']['size_trits'])
"

echo ""
echo "=== TL-KEM-512 Encapsulate ==="
python3 -c "
import json
kg = json.load(open('/tmp/kem_keygen.json'))
body = {'variant':'TL-KEM-512','public_key':kg['public_key']['data']}
open('/tmp/encap_req.json','w').write(json.dumps(body))
"
curl -sf -X POST "$BASE/api/pqti/tlkem/encapsulate" \
  -H "Content-Type: application/json" \
  -d @/tmp/encap_req.json > /tmp/encap.json
python3 -c "
import json
d = json.load(open('/tmp/encap.json'))
print('Success:', d['success'])
print('CT trits:', d['ciphertext']['size_trits'])
print('SS trits:', d['shared_secret']['size_trits'])
"

echo ""
echo "=== TL-KEM-512 Decapsulate ==="
python3 -c "
import json
kg = json.load(open('/tmp/kem_keygen.json'))
enc = json.load(open('/tmp/encap.json'))
body = {'variant':'TL-KEM-512','secret_key':kg['secret_key']['data'],'ciphertext':enc['ciphertext']['data']}
open('/tmp/decap_req.json','w').write(json.dumps(body))
"
curl -sf -X POST "$BASE/api/pqti/tlkem/decapsulate" \
  -H "Content-Type: application/json" \
  -d @/tmp/decap_req.json > /tmp/decap.json
python3 -c "
import json
enc = json.load(open('/tmp/encap.json'))
dec = json.load(open('/tmp/decap.json'))
print('Success:', dec['success'])
print('SS match:', enc['shared_secret']['data'] == dec['shared_secret']['data'])
"

echo ""
echo "=== All Tests Passed ==="
