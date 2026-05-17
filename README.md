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

### 1. Build

```bash
cargo build --release -p qorvum-node
cargo install --path crates/qorvum-cli   # install / update qv CLI
```

### 2. Wizard setup (satu kali)

```bash
qv init
```

Wizard interaktif akan memandu langkah demi langkah:

```
  ╔══════════════════════════════════════════════╗
  ║   Qorvum Setup Wizard                        ║
  ╚══════════════════════════════════════════════╝

? Organization name › MyOrg
? Admin username › admin
? Node role › ❯ all (validator + gateway + peer)
? Gateway listen address › 0.0.0.0:8080
? P2P listen address › /ip4/0.0.0.0/tcp/7051
? Data directory › ./data
? CA passphrase › ****
? Confirm passphrase › ****

  Summary
  ───────────────────────────────────────────
  Org      : MyOrg
  Admin    : admin [ADMIN]
  Role     : all
  ...

? Proceed? › Yes

  Initializing CA... done
  Issuing admin certificate... done
  Setting active identity... done
  Writing config/node.yaml... done

  Start your node:
    cargo run -p qorvum-node
```

Setelah wizard selesai:
- CA tersimpan di `~/.qorvum/ca/<org>/`
- Identitas aktif di-set ke `admin`
- `config/node.yaml` ditulis otomatis

### 3. Jalankan node

```bash
# Baca config/node.yaml otomatis
./target/release/qorvum-node

# Atau dengan custom config
./target/release/qorvum-node --config config/node-prod.yaml
```

Log startup yang diharapkan:
```
Local peer id: 12D3KooW...
Validator pubkey: a1b2c3d4...
PKI loaded from "~/.qorvum/ca/myorg" — token verification enabled
CA enrollment enabled — admin endpoints active
[gateway] REST API ready at http://0.0.0.0:8080
```

### 4. Live dashboard

```bash
# Di terminal lain (node harus sudah jalan)
qv node top
```

```
┌─ Qorvum Node ────────────────────────────────────────────────────────┐
│ url: http://localhost:8080   uptime: 2h 15m 30s   [q] quit           │
├──────────────────────────────────┬───────────────────────────────────┤
│  Blockchain                      │  System                           │
│  Height    1,234                 │  CPU  12.3%                       │
│                                  │  ████░░░░░░                       │
│  Disk      512 MB (RocksDB)      │  RAM  256 / 8192 MB               │
│                                  │  ███░░░░░░░░  3%                  │
├──────────────────────────────────┴───────────────────────────────────┤
│  Recent Blocks                                                        │
│  #1234   abc12345   12 tx   10:30:01                                 │
│  #1233   def45678    8 tx   10:29:58                                 │
│  #1232   ghi89012    5 tx   10:29:55                                 │
└──────────────────────────────────────────────────────────────────────┘
● Refreshing every 2s   last update: 10:30:03
```

Tekan `q` untuk keluar.

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

## Config File (`config/node.yaml`)

Node membaca `config/node.yaml` (atau `config/node.yml`) secara otomatis saat startup. CLI flags dan environment variables selalu mengoverride nilai dari config file.

```yaml
# config/node.yaml
org: MyOrg

node:
  role: all                            # all | validator | gateway | peer
  listen: "0.0.0.0:8080"
  p2p_listen: "/ip4/0.0.0.0/tcp/7051"
  data_dir: "./data"
  channel: main-channel
  log_level: info

ca:
  dir: "~/.qorvum/ca/myorg"
  # passphrase: direkomendasikan via env var QORVUM_CA_PASSPHRASE

# validator_keys menerima tiga format:
validator_keys:
  - "./data/node2"            # path data dir → baca validator.key otomatis
  - "./data/node2/validator.key"  # path langsung ke file
  # - "abc123def456..."       # hex pubkey (cara manual)

peers: []
```

### Penggunaan

```bash
# Default: baca config/node.yaml atau config/node.yml otomatis
./target/release/qorvum-node

# Custom config
./target/release/qorvum-node --config config/node-prod.yaml

# Via env var
QORVUM_CONFIG=config/node2.yaml ./target/release/qorvum-node

# CLI flag tetap mengoverride config file
./target/release/qorvum-node --config config/node1.yaml --log-level debug
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

### Langkah 1 — Setup (satu kali)

```bash
qv init   # wizard interaktif: buat CA, admin cert, config/node.yaml
```

Atau manual:
```bash
qv ca init --org Org1 --out ./org1-ca --passphrase <CA_PASSPHRASE>
qv ca issue --ca ./org1-ca --name admin --roles "ADMIN" --days 3650
qv identity use admin.cert admin.key
```

### Langkah 2 — Generate validator keypair (satu kali per node)

Validator keypair di-generate otomatis saat node pertama kali jalan. Jalankan sebentar lalu Ctrl+C:

```bash
# Terminal 1
./target/release/qorvum-node --role all --data-dir ./data/node1 --p2p-listen /ip4/0.0.0.0/tcp/7051 --listen 0.0.0.0:8080
# Tunggu "[gateway] REST API ready" → Ctrl+C

# Terminal 2
./target/release/qorvum-node --role validator --data-dir ./data/node2 --p2p-listen /ip4/0.0.0.0/tcp/7052
# Tunggu "[peer] P2P network running" → Ctrl+C
```

### Langkah 3 — Buat config file per node

**`config/node1.yaml`:**
```yaml
node:
  role: all
  listen: "0.0.0.0:8080"
  p2p_listen: "/ip4/0.0.0.0/tcp/7051"
  data_dir: "./data/node1"
  channel: main-channel

ca:
  dir: "~/.qorvum/ca/org1"
  # passphrase via: export QORVUM_CA_PASSPHRASE=<secret>

validator_keys:
  - "./data/node2"    # baca ./data/node2/validator.key otomatis

peers: []
```

**`config/node2.yaml`:**
```yaml
node:
  role: validator
  p2p_listen: "/ip4/0.0.0.0/tcp/7052"
  data_dir: "./data/node2"
  channel: main-channel

ca:
  dir: "~/.qorvum/ca/org1"

validator_keys:
  - "./data/node1"    # baca ./data/node1/validator.key otomatis

peers: []
```

### Langkah 4 — Jalankan

```bash
# Terminal 1
QORVUM_CA_PASSPHRASE=<secret> ./target/release/qorvum-node --config config/node1.yaml

# Terminal 2
./target/release/qorvum-node --config config/node2.yaml
```

Log yang diharapkan setelah keduanya jalan:
```
mDNS discovered: 12D3KooWXxxxx...
```

### Langkah 5 — Bootstrap admin dan verifikasi

```bash
TOKEN=$(qv identity token --ttl 86400)
curl -X POST http://localhost:8080/api/v1/admin/users/enroll \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"adminpassword","roles":["ADMIN"],"days":3650}'

curl http://localhost:8080/api/v1/health
# → {"status":"ok","mode":"consensus","latest_block":null}
```

`"mode":"consensus"` menandakan HotStuff BFT aktif.

### Untuk 2 Server di LAN yang Sama

Prosesnya identik, ganti `0.0.0.0` dengan IP masing-masing server di config file. mDNS bekerja di LAN, tapi disarankan tambah `bootstrap-peers` untuk koneksi yang lebih cepat.

**`config/node1.yaml`** (server `10.0.0.1`):
```yaml
node:
  role: all
  listen: "0.0.0.0:8080"
  p2p_listen: "/ip4/10.0.0.1/tcp/7051"
  data_dir: "/var/lib/qorvum/node1"
  channel: main-channel

ca:
  dir: "/etc/qorvum/ca"

validator_keys:
  - "/var/lib/qorvum/node2"

peers:
  - "/ip4/10.0.0.2/tcp/7051/p2p/<NODE2_PEER_ID>"
```

**`config/node2.yaml`** (server `10.0.0.2`):
```yaml
node:
  role: validator
  p2p_listen: "/ip4/10.0.0.2/tcp/7051"
  data_dir: "/var/lib/qorvum/node2"
  channel: main-channel

ca:
  dir: "/etc/qorvum/ca"

validator_keys:
  - "/var/lib/qorvum/node1"

peers:
  - "/ip4/10.0.0.1/tcp/7051/p2p/<NODE1_PEER_ID>"
```

Distribusi CA ke server 2:
```bash
scp ./org1-ca/ca.cert user@10.0.0.2:/etc/qorvum/ca/
scp ./org1-ca/crl.json user@10.0.0.2:/etc/qorvum/ca/
```

> `ca.key` + passphrase hanya di node gateway. Validator cukup `ca.cert` + `crl.json`.

---

## Production: Multi-Node (3 Node, 1 LAN)

> **Scope**: 3 validator memberikan quorum 2 dari 3 — bisa tolerir 1 crash node. Untuk toleransi Byzantine fault penuh butuh minimal 4 node. Peer discovery via mDNS — bekerja di LAN / loopback.

Topologi:
```
Client ──► node1 (gateway + validator)  :8080 / P2P :7051
                │   P2P mDNS (gossipsub)
           node2 (validator)             P2P :7052
                │
           node3 (validator)             P2P :7053
```

### Langkah 1 — Setup (satu kali)

```bash
qv init   # atau manual: qv ca init + qv ca issue + qv identity use
```

### Langkah 2 — Generate validator keypair (satu kali per node)

```bash
# Jalankan sebentar lalu Ctrl+C di masing-masing terminal
./target/release/qorvum-node --role all       --data-dir ./data/node1 --p2p-listen /ip4/0.0.0.0/tcp/7051 --listen 0.0.0.0:8080
./target/release/qorvum-node --role validator --data-dir ./data/node2 --p2p-listen /ip4/0.0.0.0/tcp/7052
./target/release/qorvum-node --role validator --data-dir ./data/node3 --p2p-listen /ip4/0.0.0.0/tcp/7053
```

### Langkah 3 — Buat config file per node

**`config/node1.yaml`:**
```yaml
node:
  role: all
  listen: "0.0.0.0:8080"
  p2p_listen: "/ip4/0.0.0.0/tcp/7051"
  data_dir: "./data/node1"

ca:
  dir: "~/.qorvum/ca/org1"
  # passphrase via: export QORVUM_CA_PASSPHRASE=<secret>

validator_keys:
  - "./data/node2"
  - "./data/node3"
```

**`config/node2.yaml`:**
```yaml
node:
  role: validator
  p2p_listen: "/ip4/0.0.0.0/tcp/7052"
  data_dir: "./data/node2"

ca:
  dir: "~/.qorvum/ca/org1"

validator_keys:
  - "./data/node1"
  - "./data/node3"
```

**`config/node3.yaml`:**
```yaml
node:
  role: validator
  p2p_listen: "/ip4/0.0.0.0/tcp/7053"
  data_dir: "./data/node3"

ca:
  dir: "~/.qorvum/ca/org1"

validator_keys:
  - "./data/node1"
  - "./data/node2"
```

### Langkah 4 — Jalankan

```bash
QORVUM_CA_PASSPHRASE=<secret> ./target/release/qorvum-node --config config/node1.yaml
./target/release/qorvum-node --config config/node2.yaml
./target/release/qorvum-node --config config/node3.yaml
```

### Langkah 5 — Bootstrap admin dan verifikasi

```bash
TOKEN=$(qv identity token --ttl 86400)
curl -X POST http://localhost:8080/api/v1/admin/users/enroll \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"adminpassword","roles":["ADMIN"],"days":3650}'

curl http://localhost:8080/api/v1/health
# → {"status":"ok","mode":"consensus","latest_block":null}
```

Coba matikan satu validator (node2 atau node3) — consensus tetap jalan karena quorum 2/3 masih terpenuhi.

---

### Untuk 3 Server di LAN yang Sama

Buat config file per server dengan IP yang sesuai, tambahkan `peers` untuk bootstrap koneksi:

**`config/node1.yaml`** (server `10.0.0.1`):
```yaml
node:
  role: all
  listen: "0.0.0.0:8080"
  p2p_listen: "/ip4/10.0.0.1/tcp/7051"
  data_dir: "/var/lib/qorvum/node1"

ca:
  dir: "/etc/qorvum/ca"

validator_keys:
  - "/var/lib/qorvum/node2"
  - "/var/lib/qorvum/node3"

peers:
  - "/ip4/10.0.0.2/tcp/7051/p2p/<NODE2_PEER_ID>"
  - "/ip4/10.0.0.3/tcp/7051/p2p/<NODE3_PEER_ID>"
```

> `NODE_PEER_ID` (`12D3KooW...`) tampil di log saat node pertama kali jalan. `ca.key` hanya dibutuhkan di node gateway; validator cukup `ca.cert` + `crl.json`.

Distribusi file CA ke server 2 dan 3:
```bash
scp ./org1-ca/ca.cert  user@10.0.0.2:/etc/qorvum/ca/
scp ./org1-ca/crl.json user@10.0.0.2:/etc/qorvum/ca/
scp ./org1-ca/ca.cert  user@10.0.0.3:/etc/qorvum/ca/
scp ./org1-ca/crl.json user@10.0.0.3:/etc/qorvum/ca/
```

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
| `gateway` | REST API, PKI auth, contract exec | REST :8080 |
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

## Contracts

Qorvum mendukung dua jenis contract: **native Rust** (dikompilasi langsung ke dalam binary node) dan **WASM** (AssemblyScript, di-deploy saat runtime).

### HR Service (Native Rust)

Contract built-in untuk manajemen karyawan.

Prefix invoke: `POST /api/v1/invoke/hr-service/<function>`
Prefix query:  `GET  /api/v1/query/hr-service/<function>`

| Function | Role | Keterangan |
|---|---|---|
| `hire_employee` | HR_MANAGER | Rekrut karyawan |
| `get_employee` | — | Ambil data karyawan |
| `update_salary` | HR_MANAGER / FINANCE | Update gaji |
| `transfer_department` | HR_MANAGER | Pindah departemen |
| `terminate_employee` | HR_MANAGER | Nonaktifkan karyawan |
| `restore_employee` | HR_ADMIN | Pulihkan dari terminasi |
| `list_by_department` | — | List karyawan per departemen |
| `search_employees` | — | Filter by salary range / posisi |
| `get_employee_history` | — | Audit trail perubahan |

### Todo Contract (WASM / AssemblyScript)

Contract contoh di `contracts/todo-as/` — ditulis dalam AssemblyScript, dikompilasi ke WASM, dan di-deploy ke node saat runtime. Mendemonstrasikan SDK ORM (`qorvum-contract-sdk`).

```bash
# Build
cd contracts/todo-as
npm install && npm run asbuild
# Output: build/release.wasm
```

```ts
// Deploy ke node
const wasm = fs.readFileSync("contracts/todo-as/build/release.wasm");
await executor.deploy_wasm("todo-contract", wasm);
```

Prefix invoke: `POST /api/v1/invoke/todo-contract/<function>`

| Function | Role | Keterangan |
|---|---|---|
| `create_todo` | — | Buat todo baru |
| `get_todo` | — | Ambil todo + assignee |
| `complete_todo` | — | Tandai selesai |
| `delete_todo` | — | Soft-delete |
| `list_todos` | — | Query dengan filter status |
| `assign_todo` | MANAGER | Assign todo ke user |

Struktur contract:
```
contracts/todo-as/assembly/
  index.ts          ← dispatch entry point
  schema.ts         ← TodoSchema & UserSchema (kolom + relasi)
  todo.service.ts   ← TodoService class (semua business logic)
```

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
# Setup wizard — buat CA, admin cert, dan config/node.yaml sekaligus
qv init [--org <ORG>]

# CA
qv ca init   --org <ORG> [--out <DIR>] [--passphrase <PASS>]
qv ca issue  --org <ORG> --name <NAME> --roles <ROLES> [--days <N>]
qv ca revoke --org <ORG> --cert <FILE>
qv ca list   --org <ORG>

# Identity
qv identity use   <CERT> <KEY>
qv identity show
qv identity token [--ttl <SECONDS>]
qv identity list
qv identity verify <TOKEN_OR_CERT>

# Node
qv node top              # live dashboard (tekan q untuk keluar)
qv node info             # info keypair dan ledger
qv node peer-id          # tampilkan validator pubkey

# API
qv invoke  <contract> <function> --args '<json>'
qv query   <contract> <function>
qv get     <collection> <partition> <id>
qv list    <collection>
qv block   <number>
qv health

# Contract
qv contract deploy --name <ID> --wasm <FILE>
qv contract list
```

**Environment variables `qv`:**

| Env Var | Default | |
|---|---|---|
| `QORVUM_URL` | `http://localhost:8080` | URL gateway |

**Environment variables `qorvum-node`:**

| Env Var | Default | |
|---|---|---|
| `QORVUM_CONFIG` | — | Path config file |
| `QORVUM_ROLE` | `all` | Role node |
| `QORVUM_LISTEN` | `0.0.0.0:8080` | Alamat REST API |
| `QORVUM_P2P_LISTEN` | `/ip4/0.0.0.0/tcp/7051` | Alamat P2P |
| `QORVUM_DATA_DIR` | `./data/node1` | Direktori RocksDB |
| `QORVUM_CA_DIR` | `./ca` | Direktori CA |
| `QORVUM_CA_PASSPHRASE` | — | Passphrase CA key |
| `QORVUM_VALIDATOR_KEYS` | — | Hex pubkeys atau paths, koma-separated |
| `QORVUM_BOOTSTRAP_PEERS` | — | Multiaddr peers, koma-separated |
| `RUST_LOG` | `info` | Log level |

---

## Menulis Contract Sendiri

Ada dua cara: **Native Rust** (dikompilasi ke dalam binary node) atau **WASM AssemblyScript** (di-deploy saat runtime tanpa rebuild node).

### Opsi A — Native Rust

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
    let id    = args["id"].as_str().ok_or("missing id")?;
    let value = args["value"].as_str().ok_or("missing value")?;

    let mut fields = HashMap::new();
    fields.insert("value".into(), FieldValue::Text(value.into()));

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

### Opsi B — WASM AssemblyScript

Tidak perlu rebuild node. Contract di-deploy saat runtime dari file `.wasm`.

**1. Buat project baru (contoh ikuti struktur `contracts/todo-as/`):**

```
contracts/my-contract/
  assembly/
    index.ts          ← dispatch entry point
    schema.ts         ← Schema definitions
    my.service.ts     ← Service class
  package.json
  asconfig.json
```

**2. Definisikan schema dan service:**

```ts
// schema.ts
import { Schema } from "qorvum-contract-sdk/assembly/index";

export const ItemSchema = new Schema("items")
  .text("name")
  .text("status")
  .bool("active");
```

```ts
// my.service.ts
import { QvModel, Fields, getField, qv_ok, qv_err } from "qorvum-contract-sdk/assembly/index";

export class ItemService {
  private items: QvModel;
  constructor(items: QvModel) { this.items = items; }

  create(args: string): i64 {
    const id   = getField(args, "id");
    const name = getField(args, "name");
    if (id.length == 0 || name.length == 0) return qv_err("id dan name wajib diisi");

    const record = this.items.create(id, new Fields()
      .text("name",   name)
      .text("status", "ACTIVE")
      .bool("active", true),
    );
    if (this.items.hasError()) return qv_err(this.items.lastError());
    return qv_ok(record);
  }
}
```

```ts
// index.ts
import { Context, QvModel, readString, qv_err } from "qorvum-contract-sdk/assembly/index";
export { alloc } from "qorvum-contract-sdk/assembly/index";
import { ItemSchema } from "./schema";
import { ItemService } from "./my.service";

export function dispatch(fn_ptr: i32, fn_len: i32, args_ptr: i32, args_len: i32): i64 {
  const name    = readString(fn_ptr, fn_len);
  const args    = readString(args_ptr, args_len);
  const service = new ItemService(new QvModel(new Context(), ItemSchema));

  if (name == "create") return service.create(args);
  return qv_err("Unknown function: " + name);
}
```

**3. Build dan deploy:**

```bash
cd contracts/my-contract
npm install
npm run asbuild   # → build/release.wasm

# Deploy ke node
const wasm = fs.readFileSync("build/release.wasm");
await executor.deploy_wasm("my-contract", wasm);
```

SDK tersedia di [`qorvum-contract-sdk/`](../qorvum-contract-sdk/) dan menyediakan `Schema`, `QvModel`, `Filter`, `Sort`, `getField`, dan helper lainnya.

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
| 6 | ✅ | WASM contract — SDK, todo-as, hot deploy, persisted ke disk (survive restart) |
| 7 | ⏳ | Docker, Helm, Prometheus |
| 8 | ⏳ | HSM, cross-org federation |

---

Apache License 2.0
