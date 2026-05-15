# Qorvum

Permissioned post-quantum blockchain. Data tersimpan secara immutable di ledger terdesentralisasi, dilindungi kriptografi Dilithium3 + BLAKE3, consensus HotStuff BFT.

---

## Status Komponen

| Komponen | Status | Keterangan |
|---|---|---|
| REST API + PKI auth (Dilithium3) | ✅ Production-ready | Token auth, enrollment, revocation |
| HotStuff BFT consensus | ✅ Production-ready | Single-node dan multi-node (LAN) |
| RocksDB persistent storage | ✅ Production-ready | Data survive restart |
| P2P multi-node (mDNS) | ✅ Sama mesin / satu LAN | Auto-discovery tanpa konfigurasi |
| Node-to-node PQ-TLS | ✅ Production-ready | Kyber-1024 KEM + X25519 + AES-256-GCM, protocol `/qorvum/pq-tls/1.0.0` |
| Bootstrap peer (cross-network) | ✅ Production-ready | `--bootstrap-peers` dial via multiaddr + PeerId |

---

## Setup

```bash
chmod +x setup.sh && ./setup.sh
```

`setup.sh` melakukan: cek + install Rust 1.85+, install system deps, build workspace, install `qv` CLI ke `~/.cargo/bin`, jalankan tests. Setelah selesai, `qv` langsung tersedia di terminal.

---

## Quick Start (Dev Mode)

Untuk testing cepat tanpa PKI. Identitas diterima tanpa verifikasi kriptografis.

```bash
cargo run -p qorvum-node

# Setelah node jalan:
curl -X POST http://localhost:8080/api/v1/auth/bootstrap \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"adminpassword","roles":["ADMIN"]}'

curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"adminpassword"}'
# → {"data":{"token":"..."}}
```

> Dev mode: node mencetak `[gateway] No CA configured — running in DEVELOPMENT mode`. Semua token diterima tanpa verifikasi cert.

---

## Production: Single Node

### 1. Build binary node

```bash
# setup.sh sudah install qv — tinggal build binary node untuk production
cargo build --release -p qorvum-node
# Binary: ./target/release/qorvum-node
```

### 2. Setup CA (satu kali, di mesin admin)

```bash
qv ca init \
  --org Org1 \
  --out ./org1-ca \
  --passphrase <CA_PASSPHRASE>
```

Output:
```
✓ CA keypair generated (Dilithium3)
✓ CA self-signed certificate → ./org1-ca/ca.cert
✓ CA private key (encrypted)  → ./org1-ca/ca.key
```

Direktori CA yang terbentuk:
```
org1-ca/
├── ca.cert     ← Didistribusikan ke semua node (PUBLIC)
├── ca.key      ← Kunci privat CA — JANGAN DIBAGIKAN
├── ca.json     ← Metadata CA
├── crl.json    ← Certificate Revocation List
├── certs/      ← Sertifikat yang pernah diterbitkan
└── users/      ← Keypair user terenkripsi
```

> **Penting — `--out` wajib diperhatikan:**
>
> Jika `--out` tidak diberikan, CA disimpan ke lokasi default: `~/.qorvum/ca/<org_lowercase>/`
>
> Contoh: `qv ca init --org NadamaOrg` → CA tersimpan di `~/.qorvum/ca/nadamaorg/` (nama org dilowercased otomatis).
>
> `--ca-dir` pada node harus menunjuk **tepat ke folder yang berisi `ca.cert`**, bukan ke parent-nya:
> ```
> # ✓ Benar
> --ca-dir ~/.qorvum/ca/nadamaorg
>
> # ✗ Salah — CA tidak ditemukan, node jalan tanpa PKI
> --ca-dir ~/.qorvum
> --ca-dir ~/.qorvum/ca
> ```
> Gunakan `--out ./org1-ca` agar lokasinya eksplisit dan mudah dirujuk.

### 3. Terbitkan sertifikat admin

```bash
qv ca issue \
  --ca ./org1-ca \
  --name admin \
  --roles "ADMIN" \
  --days 3650
# Hasil: admin.cert + admin.key (di direktori saat ini)

# Set sebagai identitas aktif di CLI
qv identity use admin.cert admin.key
qv identity show
```

Output `identity show`:
```
Subject  : admin@Org1
Roles    : [ADMIN]
Type     : User
Expires  : 2036-05-14 (VALID)
```

### 4. Jalankan node

```bash
./target/release/qorvum-node --role all --data-dir ./data/node1 --listen 0.0.0.0:8080 --p2p-listen /ip4/0.0.0.0/tcp/7051 --ca-dir ./org1-ca --ca-passphrase <CA_PASSPHRASE>
```

Log startup yang diharapkan:
```
Local peer id: 12D3KooW...
Validator pubkey: a1b2c3d4...
PKI loaded from "./org1-ca" — token verification enabled
CA enrollment enabled — admin endpoints active
[gateway] REST API ready at http://0.0.0.0:8080
```

### 5. Bootstrap akun admin (satu kali)

Ada dua cara:

**Cara A — via CLI (direkomendasikan):**
```bash
# Generate token dari cert admin
TOKEN=$(qv identity token --ttl 86400)

# Daftarkan ke user store
curl -X POST http://localhost:8080/api/v1/admin/users/enroll \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"adminpassword","roles":["ADMIN"],"days":3650}'
```

**Cara B — via bootstrap endpoint (sebelum user pertama ada):**
```bash
curl -X POST http://localhost:8080/api/v1/auth/bootstrap \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"adminpassword","roles":["ADMIN"]}'
# Endpoint ini otomatis disabled setelah ada satu user
```

### 6. Login dan gunakan API

```bash
# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"adminpassword"}'
# → {"data":{"token":"eyJ...","expires_at":1234567890}}

TOKEN="eyJ..."

# Enroll user lain
curl -X POST http://localhost:8080/api/v1/admin/users/enroll \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"alicesecret","roles":["HR_MANAGER"],"days":365}'

# Invoke contract
curl -X POST http://localhost:8080/api/v1/invoke/hr-service/hire_employee \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"id":"EMP001","name":"Budi","department":"IT","position":"Engineer","salary":20000000,"join_date":"2025-01-01","email":"budi@co.com"}'
```

---

## Node-to-Node PQ-TLS

Setiap koneksi P2P antar node Qorvum otomatis dienkripsi menggunakan protokol custom `/qorvum/pq-tls/1.0.0` yang tahan serangan quantum computer.

### Algoritma

| Tahap | Algoritma | Fungsi |
|---|---|---|
| Key Exchange | Kyber-1024 KEM + X25519 ECDH (hybrid) | Negotiasi session key |
| Encryption | AES-256-GCM | Enkripsi payload per frame |
| Authentication | Ed25519 (libp2p identity) | Verifikasi PeerId |
| MSP Auth (opsional) | Dilithium3 | Verifikasi identitas MSP/PKI |

Hybrid KEM memastikan keamanan bahkan jika salah satu dari Kyber atau X25519 ditemukan kelemahannya.

### Cara Kerja

```
Node A                           Node B
  │                                │
  │── multistream-select ──────────►│  Negotiate /qorvum/pq-tls/1.0.0
  │                                │
  │── Ed25519 pubkey (u16-prefixed)►│  \
  │◄─ Ed25519 pubkey ──────────────│   ├─ PeerId exchange (stable identity)
  │                                │  /
  │── ClientHello (Kyber KEM) ─────►│  \
  │◄─ ServerHello (KEM ciphertext) ─│   ├─ PQ handshake
  │── ClientFinish (X25519 share) ──►│   │  derive shared_key = HKDF(kyber_ss ‖ x25519_ss)
  │◄─ ServerAck ───────────────────│  /
  │                                │
  │══ AES-256-GCM encrypted frames ══│  Session aktif
```

Setelah handshake, seluruh traffic gossipsub (transaksi + consensus) mengalir melalui sesi terenkripsi ini.

### Aktif Secara Default

PQ-TLS aktif secara otomatis — tidak perlu konfigurasi tambahan. Setiap node yang dijalankan dengan versi ini akan otomatis menggunakan `/qorvum/pq-tls/1.0.0` untuk semua koneksi P2P.

```bash
# Log yang menandakan PQ-TLS aktif:
PQ-TLS server: PeerId exchange done  peer=12D3KooW...
PQ-TLS client: PeerId exchange done  peer=12D3KooW...
```

---

## Bootstrap Peer (Cross-Network)

Bootstrap peer digunakan untuk menghubungkan node yang berada di **subnet berbeda** (antar datacenter, cloud-to-on-premise, dll.) yang tidak bisa dijangkau mDNS.

### Format Multiaddr Bootstrap

```
/ip4/<IP>/tcp/<PORT>/p2p/<LIBP2P_PEER_ID>
```

- `<IP>` — IP publik atau IP LAN node tujuan
- `<PORT>` — port P2P (default `7051`)
- `<LIBP2P_PEER_ID>` — libp2p PeerId node tujuan, format `12D3KooW...` (bukan validator hex pubkey)

### Mendapatkan libp2p PeerId

Saat node pertama kali dijalankan, PeerId dicetak ke log:

```bash
./target/release/qorvum-node --role all --data-dir ./data/node1 --p2p-listen /ip4/0.0.0.0/tcp/7051
# → INFO qorvum_network: Local peer id: 12D3KooWAbc123...
# Ctrl+C
```

> **PeerId vs Validator Pubkey**: PeerId (`12D3KooW...`) adalah identitas libp2p untuk routing P2P. Validator pubkey (hex panjang) adalah kunci kriptografi untuk BFT consensus. Keduanya berbeda — `--bootstrap-peers` menggunakan PeerId, `--validator-keys` menggunakan validator pubkey.

### Contoh: 2 Node Beda Subnet

```
Datacenter A (10.0.0.0/24)      Datacenter B (172.16.0.0/24)
    node1 (10.0.0.1)    ◄────► node2 (172.16.0.1)
    PeerId: 12D3KooWAAA         PeerId: 12D3KooWBBB
```

**Langkah 1 — Jalankan node1 dan catat PeerId:**

```bash
# Di Datacenter A
./target/release/qorvum-node --role all \
  --data-dir ./data/node1 \
  --p2p-listen /ip4/10.0.0.1/tcp/7051 \
  --listen 0.0.0.0:8080
# Catat: Local peer id: 12D3KooWAAA...
```

**Langkah 2 — Jalankan node2 dan catat PeerId:**

```bash
# Di Datacenter B
./target/release/qorvum-node --role validator \
  --data-dir ./data/node2 \
  --p2p-listen /ip4/172.16.0.1/tcp/7051
# Catat: Local peer id: 12D3KooWBBB...
```

**Langkah 3 — Restart dengan `--bootstrap-peers`:**

```bash
# node1 → dial node2
./target/release/qorvum-node --role all \
  --data-dir ./data/node1 \
  --p2p-listen /ip4/10.0.0.1/tcp/7051 \
  --listen 0.0.0.0:8080 \
  --validator-keys $NODE2_PUBKEY \
  --bootstrap-peers /ip4/172.16.0.1/tcp/7051/p2p/12D3KooWBBB...

# node2 → dial node1
./target/release/qorvum-node --role validator \
  --data-dir ./data/node2 \
  --p2p-listen /ip4/172.16.0.1/tcp/7051 \
  --validator-keys $NODE1_PUBKEY \
  --bootstrap-peers /ip4/10.0.0.1/tcp/7051/p2p/12D3KooWAAA...
```

Log koneksi berhasil:

```
INFO  Dialing bootstrap peer: /ip4/172.16.0.1/tcp/7051/p2p/12D3KooWBBB...
INFO  PQ-TLS client: PeerId exchange done  peer=12D3KooWBBB...
INFO  Connected to peer 12D3KooWBBB... at /ip4/172.16.0.1/tcp/7051
```

### Multiple Bootstrap Peers

```bash
# Koma-separated via env var
QORVUM_BOOTSTRAP_PEERS="/ip4/10.0.0.1/tcp/7051/p2p/12D3KooWAAA,/ip4/10.0.0.2/tcp/7051/p2p/12D3KooWBBB"

# Atau --bootstrap-peers berulang
./target/release/qorvum-node \
  --bootstrap-peers /ip4/10.0.0.1/tcp/7051/p2p/12D3KooWAAA \
  --bootstrap-peers /ip4/10.0.0.2/tcp/7051/p2p/12D3KooWBBB \
  ...
```

### Catatan Firewall

Port P2P (`7051` default) harus terbuka untuk TCP inbound/outbound:

```bash
# UFW
ufw allow 7051/tcp

# iptables
iptables -A INPUT -p tcp --dport 7051 -j ACCEPT
```

---

## Production: Multi-Node (2 Node, 1 LAN)

> **Scope**: Tutorial ini untuk 2 node di mesin yang sama (2 terminal) atau 2 server di satu LAN. Peer discovery menggunakan mDNS — bekerja di loopback dan LAN, belum mendukung cross-network (antar subnet berbeda).

Topologi:

```
Client ──► node1 (gateway + validator)  :8080
                │  P2P mDNS (gossipsub)
           node2 (validator)             :7052
```

node1 melayani REST API dan berpartisipasi dalam consensus. node2 hanya validator — menambah fault tolerance BFT.

---

### Langkah 1 — Setup CA (sama seperti single-node, skip jika sudah)

```bash
qv ca init --org Org1 --out ./org1-ca --passphrase <CA_PASSPHRASE>
qv ca issue --ca ./org1-ca --name admin --roles "ADMIN" --days 3650
qv identity use admin.cert admin.key
```

### Langkah 2 — Generate validator keypair (satu kali per node)

Validator keypair di-generate otomatis pada run pertama dan disimpan di `{data-dir}/validator.key`. Jalankan setiap node sebentar lalu Ctrl+C:

**Terminal 1 — node1:**
```bash
./target/release/qorvum-node --role all --data-dir ./data/node1 --p2p-listen /ip4/0.0.0.0/tcp/7051 --listen 0.0.0.0:8080
# Tunggu "[gateway] REST API ready" → Ctrl+C
```

**Terminal 2 — node2:**
```bash
./target/release/qorvum-node --role validator --data-dir ./data/node2 --p2p-listen /ip4/0.0.0.0/tcp/7052
# Tunggu "[peer] P2P network running" → Ctrl+C
```

Ambil validator pubkey (hex) lewat CLI:

```bash
NODE1_PUBKEY=$(qv node peer-id --data-dir ./data/node1 | grep -E '^[0-9a-f]{100,}$')
NODE2_PUBKEY=$(qv node peer-id --data-dir ./data/node2 | grep -E '^[0-9a-f]{100,}$')

echo "node1: $NODE1_PUBKEY"
echo "node2: $NODE2_PUBKEY"
```

### Langkah 3 — Jalankan kedua node dengan PKI dan validator set lengkap

**Terminal 1 — node1** (gateway + validator):
```bash
./target/release/qorvum-node --role all --data-dir ./data/node1 --listen 0.0.0.0:8080 --p2p-listen /ip4/0.0.0.0/tcp/7051 --ca-dir ./org1-ca --ca-passphrase <CA_PASSPHRASE> --validator-keys $NODE2_PUBKEY
```

**Terminal 2 — node2** (validator):
```bash
./target/release/qorvum-node --role validator --data-dir ./data/node2 --p2p-listen /ip4/0.0.0.0/tcp/7052 --validator-keys $NODE1_PUBKEY
```

Log yang diharapkan setelah keduanya jalan:
```
# di node1 atau node2:
mDNS discovered: 12D3KooWXxxxx...
```

Ini menandakan kedua node sudah saling terhubung via gossipsub dan membentuk validator set bersama.

### Langkah 4 — Bootstrap admin (sama seperti single-node)

```bash
TOKEN=$(qv identity token --ttl 86400)
curl -X POST http://localhost:8080/api/v1/admin/users/enroll \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"adminpassword","roles":["ADMIN"],"days":3650}'
```

### Langkah 5 — Verifikasi cluster

```bash
curl http://localhost:8080/api/v1/health
```
```json
{"status":"ok","channel":"main-channel","mode":"consensus","latest_block":null}
```

`"mode":"consensus"` menandakan node berjalan dengan HotStuff BFT aktif.

### Untuk 2 Server di LAN yang Sama

Prosesnya identik, ganti `localhost`/`0.0.0.0` dengan IP masing-masing server. mDNS bekerja di LAN, tapi disarankan tetap menggunakan `--bootstrap-peers` untuk koneksi yang lebih cepat dan deterministik.

```bash
# Di server 1 (setelah run pertama lalu Ctrl+C):
NODE1_PUBKEY=$(qv node peer-id --data-dir /var/lib/qorvum/node1 | grep -E '^[0-9a-f]{100,}$')
NODE1_PEER_ID="<lihat log: Local peer id: 12D3KooW...>"
# Di server 2 (setelah run pertama lalu Ctrl+C):
NODE2_PUBKEY=$(qv node peer-id --data-dir /var/lib/qorvum/node2 | grep -E '^[0-9a-f]{100,}$')
NODE2_PEER_ID="<lihat log: Local peer id: 12D3KooW...>"
```

**Server 1** (`10.0.0.1`) — gateway + validator:
```bash
./target/release/qorvum-node --role all --data-dir /var/lib/qorvum/node1 --listen 0.0.0.0:8080 --p2p-listen /ip4/10.0.0.1/tcp/7051 --ca-dir /etc/qorvum/ca --ca-passphrase <CA_PASSPHRASE> --validator-keys $NODE2_PUBKEY --bootstrap-peers /ip4/10.0.0.2/tcp/7051/p2p/$NODE2_PEER_ID
```

**Server 2** (`10.0.0.2`) — validator:
```bash
./target/release/qorvum-node --role validator --data-dir /var/lib/qorvum/node2 --p2p-listen /ip4/10.0.0.2/tcp/7051 --validator-keys $NODE1_PUBKEY --bootstrap-peers /ip4/10.0.0.1/tcp/7051/p2p/$NODE1_PEER_ID
```

Distribusi CA ke server 2:
```bash
scp ./org1-ca/ca.cert user@10.0.0.2:/etc/qorvum/ca/
scp ./org1-ca/crl.json user@10.0.0.2:/etc/qorvum/ca/
```

> `ca.key` + passphrase hanya di node gateway. Validator cukup `ca.cert` + `crl.json`.

---

## Production: Multi-Node (3 Node, 1 LAN)

> **Scope**: 3 validator memberikan quorum 2 dari 3 — bisa tolerir 1 crash node tapi belum bisa tolerir Byzantine fault. Untuk BFT yang benar-benar toleran terhadap node jahat, butuh minimal 4 node (quorum 3 dari 4, tolerir 1 Byzantine). Peer discovery via mDNS — bekerja di LAN / loopback.

Topologi:
```
Client ──► node1 (gateway + validator)  :8080 / P2P :7051
                │   P2P mDNS (gossipsub)
           node2 (validator)             P2P :7052
                │
           node3 (validator)             P2P :7053
```

### Langkah 1 — Setup CA (skip jika sudah)

```bash
qv ca init --org Org1 --out ./org1-ca --passphrase <CA_PASSPHRASE>
qv ca issue --ca ./org1-ca --name admin --roles "ADMIN" --days 3650
qv identity use admin.cert admin.key
```

### Langkah 2 — Generate validator keypair (satu kali per node)

Jalankan setiap node sebentar lalu Ctrl+C:

```bash
# Terminal 1
./target/release/qorvum-node --role all --data-dir ./data/node1 --p2p-listen /ip4/0.0.0.0/tcp/7051 --listen 0.0.0.0:8080
# Terminal 2
./target/release/qorvum-node --role validator --data-dir ./data/node2 --p2p-listen /ip4/0.0.0.0/tcp/7052
# Terminal 3
./target/release/qorvum-node --role validator --data-dir ./data/node3 --p2p-listen /ip4/0.0.0.0/tcp/7053
# Tunggu "[gateway|peer] ready" di masing-masing → Ctrl+C semua
```

Ambil pubkey (hex) semua node:

```bash
NODE1_PUBKEY=$(qv node peer-id --data-dir ./data/node1 | grep -E '^[0-9a-f]{100,}$')
NODE2_PUBKEY=$(qv node peer-id --data-dir ./data/node2 | grep -E '^[0-9a-f]{100,}$')
NODE3_PUBKEY=$(qv node peer-id --data-dir ./data/node3 | grep -E '^[0-9a-f]{100,}$')

echo "node1: $NODE1_PUBKEY"
echo "node2: $NODE2_PUBKEY"
echo "node3: $NODE3_PUBKEY"
```

### Langkah 3 — Jalankan semua node dengan validator set lengkap

Setiap node butuh pubkey **kedua node lainnya** via `--validator-keys`.

**Terminal 1 — node1** (gateway + validator):
```bash
./target/release/qorvum-node --role all --data-dir ./data/node1 --listen 0.0.0.0:8080 --p2p-listen /ip4/0.0.0.0/tcp/7051 --ca-dir ./org1-ca --ca-passphrase <CA_PASSPHRASE> --validator-keys $NODE2_PUBKEY,$NODE3_PUBKEY
```

**Terminal 2 — node2** (validator):
```bash
./target/release/qorvum-node --role validator --data-dir ./data/node2 --p2p-listen /ip4/0.0.0.0/tcp/7052 --validator-keys $NODE1_PUBKEY,$NODE3_PUBKEY
```

**Terminal 3 — node3** (validator):
```bash
./target/release/qorvum-node --role validator --data-dir ./data/node3 --p2p-listen /ip4/0.0.0.0/tcp/7053 --validator-keys $NODE1_PUBKEY,$NODE2_PUBKEY
```

### Langkah 4 — Bootstrap admin dan verifikasi

```bash
# Bootstrap admin
TOKEN=$(qv identity token --ttl 86400)
curl -X POST http://localhost:8080/api/v1/admin/users/enroll \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"adminpassword","roles":["ADMIN"],"days":3650}'

# Verifikasi cluster
curl http://localhost:8080/api/v1/health
# → {"status":"ok","mode":"consensus","latest_block":null}
```

Coba matikan satu validator (node2 atau node3) — consensus tetap jalan karena quorum 2/3 masih terpenuhi.

---

### Untuk 3 Server di LAN yang Sama

Prosesnya identik, ganti `localhost`/`0.0.0.0` dengan IP masing-masing server. Gunakan `--bootstrap-peers` agar koneksi antar node cepat dan deterministik.

Ambil pubkey dan PeerId masing-masing setelah run pertama (lalu Ctrl+C):
```bash
# Di server 1:
NODE1_PUBKEY=$(qv node peer-id --data-dir /var/lib/qorvum/node1 | grep -E '^[0-9a-f]{100,}$')
NODE1_PEER_ID="<log: Local peer id: 12D3KooW...>"
# Di server 2:
NODE2_PUBKEY=$(qv node peer-id --data-dir /var/lib/qorvum/node2 | grep -E '^[0-9a-f]{100,}$')
NODE2_PEER_ID="<log: Local peer id: 12D3KooW...>"
# Di server 3:
NODE3_PUBKEY=$(qv node peer-id --data-dir /var/lib/qorvum/node3 | grep -E '^[0-9a-f]{100,}$')
NODE3_PEER_ID="<log: Local peer id: 12D3KooW...>"
```

**Server 1** (`10.0.0.1`) — gateway + validator:
```bash
./target/release/qorvum-node --role all --data-dir /var/lib/qorvum/node1 --listen 0.0.0.0:8080 --p2p-listen /ip4/10.0.0.1/tcp/7051 --ca-dir /etc/qorvum/ca --ca-passphrase <CA_PASSPHRASE> --validator-keys $NODE2_PUBKEY,$NODE3_PUBKEY --bootstrap-peers /ip4/10.0.0.2/tcp/7051/p2p/$NODE2_PEER_ID,/ip4/10.0.0.3/tcp/7051/p2p/$NODE3_PEER_ID
```

**Server 2** (`10.0.0.2`) — validator:
```bash
./target/release/qorvum-node --role validator --data-dir /var/lib/qorvum/node2 --p2p-listen /ip4/10.0.0.2/tcp/7051 --validator-keys $NODE1_PUBKEY,$NODE3_PUBKEY --bootstrap-peers /ip4/10.0.0.1/tcp/7051/p2p/$NODE1_PEER_ID,/ip4/10.0.0.3/tcp/7051/p2p/$NODE3_PEER_ID
```

**Server 3** (`10.0.0.3`) — validator:
```bash
./target/release/qorvum-node --role validator --data-dir /var/lib/qorvum/node3 --p2p-listen /ip4/10.0.0.3/tcp/7051 --validator-keys $NODE1_PUBKEY,$NODE2_PUBKEY --bootstrap-peers /ip4/10.0.0.1/tcp/7051/p2p/$NODE1_PEER_ID,/ip4/10.0.0.2/tcp/7051/p2p/$NODE2_PEER_ID
```

Distribusi file CA ke server 2 dan 3:
```bash
scp ./org1-ca/ca.cert user@10.0.0.2:/etc/qorvum/ca/
scp ./org1-ca/crl.json user@10.0.0.2:/etc/qorvum/ca/
scp ./org1-ca/ca.cert user@10.0.0.3:/etc/qorvum/ca/
scp ./org1-ca/crl.json user@10.0.0.3:/etc/qorvum/ca/
```

> `ca.key` dan `ca_passphrase` hanya dibutuhkan di node yang menjalankan enrollment (biasanya node1/gateway). Node validator cukup `ca.cert` + `crl.json` untuk verifikasi.

---

## Manajemen User

```bash
# Enroll user baru (butuh ADMIN token)
curl -X POST http://localhost:8080/api/v1/admin/users/enroll \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret","roles":["HR_MANAGER"],"days":365}'

# List semua user
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  http://localhost:8080/api/v1/admin/users

# Revoke user (berlaku langsung, token yang sudah ada ikut invalid)
curl -X POST http://localhost:8080/api/v1/admin/users/alice/revoke \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

---

## Node Roles

| Role | Yang Aktif | Port Default |
|---|---|---|
| `all` | Validator + Gateway + Peer | REST :8080, P2P :7051 |
| `validator` | HotStuff BFT + P2P | P2P :7051 |
| `gateway` | REST API, PKI auth, chaincode exec | REST :8080 |
| `peer` | P2P sync | P2P :7051 |

```bash
# Semua role (dev/single-node/production combined)
qorvum-node --role all

# Validator terpisah
qorvum-node --role validator

# Gateway terpisah (TX dikirim langsung tanpa consensus — single-node mode)
qorvum-node --role gateway

# Validator + Gateway dalam satu proses
qorvum-node --role validator --role gateway
```

### Environment Variables

| Variable | Default | Keterangan |
|---|---|---|
| `QORVUM_ROLE` | `all` | Role node |
| `QORVUM_LISTEN` | `0.0.0.0:8080` | Alamat REST API |
| `QORVUM_P2P_LISTEN` | `/ip4/0.0.0.0/tcp/7051` | Alamat P2P |
| `QORVUM_DATA_DIR` | `./data/node1` | Direktori RocksDB |
| `QORVUM_CA_DIR` | `./ca` | Direktori CA |
| `QORVUM_CA_PASSPHRASE` | — | Passphrase CA key (aktifkan enrollment) |
| `QORVUM_VALIDATOR_KEYS` | — | Hex pubkeys validator lain, koma-separated |
| `QORVUM_BOOTSTRAP_PEERS` | — | Multiaddr bootstrap peers (koma-separated), format `/ip4/<IP>/tcp/<PORT>/p2p/<PEER_ID>` |
| `RUST_LOG` | `info` | Log level |

---

## Contract: HR Service

Prefix invoke: `POST /api/v1/invoke/hr-service/<function>`
Prefix query: `GET /api/v1/query/hr-service/<function>`

| Function | Role | Keterangan |
|---|---|---|
| `hire_employee` | HR_MANAGER | Rekrut karyawan |
| `get_employee` | — | Ambil data |
| `update_salary` | HR_MANAGER / FINANCE | Update gaji |
| `transfer_department` | HR_MANAGER | Pindah dept |
| `terminate_employee` | HR_MANAGER | Nonaktifkan |
| `restore_employee` | HR_ADMIN | Pulihkan |
| `list_by_department` | — | List per dept |
| `search_employees` | — | Filter by salary/posisi |
| `get_employee_history` | — | Audit trail |

---

## Ledger & Block Explorer

```bash
# Blocks
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/blocks
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/blocks/1

# Records
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/records/<collection>/<partition>/<id>
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/v1/records/<collection>?partition=<p>&limit=20"

# Audit trail
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/history/<collection>/<id>
```

---

## CLI (`qv`)

```bash
# CA
qv ca init   --org <ORG> --out <DIR> --passphrase <PASS>
qv ca issue  --ca <DIR> --name <NAME> --roles <ROLES> --days <N>
qv ca revoke --ca <DIR> --cert <FILE>
qv ca list   --ca <DIR>

# Identity
qv identity use   <CERT> <KEY>
qv identity show
qv identity token [--ttl <SECONDS>]

# API
qv invoke  <contract> <function> --args '<json>'
qv query   <contract> <function>
qv get     <collection> <partition> <id>
qv list    <collection>
qv block   <number>
qv health
```

| Env Var | Default | |
|---|---|---|
| `QORVUM_URL` | `http://localhost:8080` | URL gateway |
| `QORVUM_CA_DIR` | `./ca` | Direktori CA |
| `QORVUM_CA_PASSPHRASE` | — | Passphrase CA |
| `QORVUM_DATA_DIR` | `./data` | Direktori data |

---

## Menulis Contract Sendiri

```rust
// contracts/my-service/src/lib.rs
use chain_sdk::{ChainContext, FieldValue};
use std::collections::HashMap;

pub fn create(
    _fn_name: &str,
    args: serde_json::Value,
    ctx: &dyn ChainContext,
) -> Result<serde_json::Value, String> {
    if !ctx.has_role("WRITER") {
        return Err("Requires WRITER role".into());
    }
    let caller = ctx.caller_identity();
    if !caller.verified {
        return Err("Production requires verified PQ certificate".into());
    }
    let id    = args["id"].as_str().ok_or("missing id")?;
    let value = args["value"].as_str().ok_or("missing value")?;

    let mut fields = HashMap::new();
    fields.insert("value".into(), FieldValue::Text(value.into()));
    fields.insert("created_by".into(), FieldValue::Text(caller.subject.clone()));

    let record = ctx.insert("my_collection", "default", id, fields)
        .map_err(|e| e.to_string())?;
    Ok(record)
}

pub fn register() -> HashMap<String, chain_sdk::NativeFn> {
    let mut m = HashMap::new();
    m.insert("create".into(), |f, a, c| create(f, a, c));
    m
}
```

Daftarkan di node:
```rust
app_state.executor.write().await
    .register_native("my-service", my_service::register());
```

---

## Tests

```bash
cargo test --workspace --lib
cargo test -p qorvum-msp      # PKI, token, enrollment
cargo test -p qorvum-ledger   # Storage, query engine
cargo test -p qorvum-network  # PQ-TLS handshake
```

---

## Roadmap

| Phase | Status | |
|---|---|---|
| 1–4 | ✅ | Crypto, MVP, HotStuff BFT, PQ-PKI |
| 5 | ✅ | REST auth, user management, role-based node |
| 5.2 | ✅ | Bootstrap peer dial (cross-network P2P) |
| 5.3 | ✅ | PQ-TLS di layer P2P (Kyber-1024 KEM + X25519 + AES-256-GCM) |
| 6 | ⏳ | WASM chaincode |
| 7 | ⏳ | Docker, Helm, Prometheus |
| 8 | ⏳ | HSM, cross-org federation |

---

Apache License 2.0
