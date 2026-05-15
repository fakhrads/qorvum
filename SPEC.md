# Qorvum — Technical Specification

**Version:** 0.1.0  
**Audience:** Business analyst, solution architect, security auditor  
**Scope:** Functional specification, security model, data model, deployment constraints

---

## 1. Ringkasan Sistem

Qorvum adalah permissioned blockchain yang dibangun dengan fokus pada dua hal: **keamanan post-quantum** dan **kemudahan integrasi enterprise**. Data tersimpan secara immutable di ledger terdesentralisasi, setiap perubahan data ditandatangani secara kriptografis, dan konsistensi antar node dijamin melalui consensus BFT.

**Kasus penggunaan utama:**
- Rekam jejak data yang tidak bisa dimanipulasi (audit trail)
- Otomasi proses bisnis berbasis aturan yang dapat diverifikasi
- Sistem identitas terpusat dengan manajemen hak akses berbasis peran
- Infrastruktur data lintas departemen atau lintas organisasi yang perlu saling percaya

**Posisi dalam arsitektur enterprise:**  
Qorvum bukan database biasa dan bukan blockchain publik. Ia berada di antara keduanya: seperti database dalam hal API dan kemudahan query, seperti blockchain dalam hal immutability, auditability, dan multi-party trust.

---

## 2. Status Komponen

| Komponen | Status | Keterangan |
|---|---|---|
| REST API + JWT auth | ✅ Production-ready | Token auth, enrollment, revocation |
| PKI berbasis Dilithium3 | ✅ Production-ready | Certificate Authority built-in |
| HotStuff BFT consensus | ✅ Production-ready | Single-node dan multi-node (LAN) |
| RocksDB persistent storage | ✅ Production-ready | Data survive restart |
| P2P multi-node via mDNS | ✅ Satu mesin / satu LAN | Auto-discovery tanpa konfigurasi |
| Block Explorer (UI) | ✅ Tersedia | Real-time via WebSocket |
| Node-to-node PQ-TLS | ⏳ Belum | P2P saat ini menggunakan libp2p Noise |
| Bootstrap peer (cross-network) | ⏳ Belum | Belum bisa antar subnet berbeda |
| WASM chaincode | ⏳ Belum | Hanya native Rust saat ini |

---

## 3. Arsitektur Lapisan

Qorvum terdiri dari lima lapisan fungsional yang bekerja secara terpisah namun terintegrasi:

**Gateway Layer**  
Titik masuk semua request dari luar. Menangani autentikasi JWT, routing request ke executor, dan mengembalikan response. Gateway juga menjadi node P2P dalam jaringan multi-node.

**Execution Layer**  
Menjalankan logika bisnis (contract/chaincode) dalam konteks terisolasi. Setiap eksekusi menghasilkan Read-Write Set — daftar kunci yang dibaca dan perubahan yang akan ditulis. Eksekusi tidak langsung menulis ke ledger; hasilnya diteruskan ke consensus terlebih dahulu.

**Consensus Layer**  
Mengimplementasikan protokol HotStuff BFT (Byzantine Fault Tolerant). Menerima proposal transaksi dari gateway, mengedarkannya ke semua validator via P2P gossipsub, mengumpulkan vote, dan mengonfirmasi komitmen setelah quorum tercapai.

**Ledger Layer**  
Menerima transaksi yang sudah dikonfirmasi consensus dan menyimpannya ke blok. Memelihara World State (kondisi terkini semua data), block store (histori lengkap), dan indeks untuk query efisien.

**Crypto Layer**  
Menyediakan semua primitif kriptografi: hashing BLAKE3, tanda tangan Dilithium3, key exchange Kyber768, dan manajemen sertifikat X.509 post-quantum.

---

## 4. Model Data

### 4.1 Record

Unit data dasar di Qorvum adalah **Record**. Setiap record diidentifikasi oleh tiga komponen:

- **Collection** — kategori data, setara dengan nama tabel (contoh: `employees`, `contracts`)
- **Partition** — segmen di dalam collection, digunakan untuk pengelompokan dan akses kontrol (contoh: departemen, wilayah)
- **ID** — identifier unik record di dalam partition

Setiap record memiliki metadata sistem yang tidak bisa diubah langsung:

| Field | Keterangan |
|---|---|
| `version` | Counter integer, bertambah 1 setiap update |
| `tx_id` | ID transaksi yang terakhir mengubah record ini |
| `block_num` | Nomor blok tempat perubahan terakhir dikonfirmasi |
| `is_deleted` | Soft-delete flag — data tidak dihapus fisik |
| `created_at` | Timestamp pertama kali dibuat |
| `updated_at` | Timestamp perubahan terakhir |

### 4.2 Field Value Types

Nilai field mendukung tipe berikut: `Text`, `Number`, `Boolean`, `Timestamp`, `Bytes`, dan `Null`. Nested object disimpan sebagai JSON.

### 4.3 Immutability & Audit Trail

Record tidak pernah dihapus dari ledger secara fisik. Setiap versi tersimpan dalam histori dan dapat diakses via endpoint audit trail. Ini memungkinkan rekonstruksi penuh kondisi data pada titik waktu manapun di masa lalu.

---

## 5. Life Cycle

### Fase 1 — Simulasi

1. Client mengirim request invoke ke gateway dengan payload JSON dan token autentikasi.
2. Gateway memverifikasi token, mengekstrak identitas dan peran caller.
3. Execution engine menjalankan fungsi contract dalam konteks terisolasi (SimulationContext).
4. Selama eksekusi, semua operasi baca dicatat (Read Set) dan semua operasi tulis dibaffer (Write Set) — belum ada yang ditulis ke ledger.
5. Eksekusi selesai, menghasilkan Read-Write Set dan response data.

### Fase 2 — Consensus

1. Gateway mengirim proposal transaksi (berisi RW Set + metadata) ke ConsensusEngine.
2. Engine meneruskan proposal ke semua validator via P2P gossipsub.
3. Setiap validator memverifikasi proposal dan mengirim vote kembali.
4. Setelah quorum vote terkumpul (lihat bagian Consensus), blok baru dikonfirmasi.

### Fase 3 — Commit

1. Blok yang dikonfirmasi ditulis ke ledger secara atomik.
2. World State diperbarui sesuai Write Set.
3. Histori versi record diperbarui.
4. Event `block` dan `tx` dikirim ke semua client Explorer yang terkoneksi via WebSocket.
5. Gateway mengembalikan response ke client yang memanggil.

**Catatan dev mode:** Ketika node berjalan tanpa validator lain (single-node / dev), fase consensus dilewati dan transaksi langsung dicommit. Mode ini tidak cocok untuk production.

---

## 6. Consensus: HotStuff BFT

### Protokol

Qorvum mengimplementasikan **HotStuff BFT** — protokol consensus Byzantine Fault Tolerant yang digunakan juga oleh LibraBFT/DiemBFT. Dibandingkan PBFT klasik, HotStuff memiliki kompleksitas komunikasi linear (bukan kuadratik), sehingga lebih efisien untuk jaringan besar.

### Quorum dan Fault Tolerance

| Jumlah Validator | Quorum Dibutuhkan | Node Crash yang Dapat Ditolerir | Node Byzantine yang Dapat Ditolerir |
|---|---|---|---|
| 1 | 1 | 0 | 0 |
| 2 | 2 | 0 | 0 |
| 3 | 2 | 1 | 0 |
| 4 | 3 | 1 | 1 |
| 7 | 5 | 2 | 2 |

Rumus: quorum = ⌊(2n/3)⌋ + 1. Untuk fault tolerance Byzantine yang sesungguhnya, dibutuhkan minimal **4 validator** (n ≥ 3f+1 di mana f adalah jumlah node jahat yang dapat ditolerir).

### Validator Identity

Setiap validator diidentifikasi oleh **Dilithium3 public key** (hex string, panjang ~2500 karakter). Setiap node harus dikonfigurasi dengan public key semua validator lain agar dapat memverifikasi vote. Key ini di-generate otomatis pada run pertama dan disimpan di direktori data node.

### P2P Transport

Komunikasi antar validator menggunakan **libp2p** dengan:
- Transport: TCP
- Discovery: mDNS (LAN/loopback only, belum cross-subnet)
- Messaging: gossipsub (pub/sub overlay)
- Security: Noise protocol (bukan PQ-TLS — lihat roadmap)

---

## 7. Keamanan dan Identitas

### 7.1 Post-Quantum Cryptography

Qorvum menggunakan algoritma yang telah distandarisasi NIST untuk ketahanan terhadap serangan komputer kuantum:

| Fungsi | Algoritma | Standar |
|---|---|---|
| Tanda tangan digital | ML-DSA / Dilithium3 | NIST FIPS 204 |
| Key encapsulation | ML-KEM / Kyber768 | NIST FIPS 203 |
| Hashing | BLAKE3 | — |
| TLS (planned) | Hybrid PQ-TLS | — |

Implikasi praktis: tanda tangan yang dibuat Qorvum hari ini tahan terhadap serangan *harvest now, decrypt later* oleh komputer kuantum di masa depan.

### 7.2 Certificate Authority (CA) Internal

Qorvum memiliki CA built-in berbasis Dilithium3. Tidak memerlukan CA eksternal (tidak bergantung pada PKI berbasis RSA/ECDSA seperti Let's Encrypt).

**Hierarki kepercayaan:**
- CA menerbitkan sertifikat X.509 untuk setiap user
- Setiap sertifikat mencantumkan **subject** (format: `username@OrgName`) dan **roles** (daftar peran)
- Node gateway memvalidasi setiap token terhadap CA cert dan CRL (Certificate Revocation List)

**Distribusi CA:**
- `ca.cert` dan `crl.json` didistribusikan ke semua node (termasuk validator)
- `ca.key` (kunci privat CA) dan passphrase hanya disimpan di node gateway / mesin admin
- Revokasi sertifikat berlaku langsung; token yang sudah diterbitkan dari cert yang direvoke menjadi invalid

### 7.3 Autentikasi API

Semua request API menggunakan **JWT Bearer Token**. Token diperoleh via:

1. **CLI certificate flow** — `qv identity token` menggunakan cert Dilithium3 untuk menghasilkan token
2. **Password login** — `POST /api/v1/auth/login` dengan username/password yang terdaftar di user store

Token mengandung: subject, org, roles, dan expiry timestamp. Diverifikasi oleh gateway pada setiap request.

### 7.4 Otorisasi Berbasis Peran (RBAC)

Kontrol akses dilakukan di level contract. Setiap fungsi contract dapat memeriksa peran caller via `ctx.has_role("ROLE_NAME")`. Peran ditentukan saat enrollment user dan tertanam dalam sertifikat.

Contoh peran yang umum digunakan dalam HR Service: `ADMIN`, `HR_MANAGER`, `FINANCE`, `HR_ADMIN`.

---

## 8. API Endpoints

Semua endpoint membutuhkan `Authorization: Bearer <token>` kecuali yang ditandai.

### Autentikasi

| Method | Endpoint | Keterangan |
|---|---|---|
| `POST` | `/api/v1/auth/bootstrap` | Buat akun pertama (auto-disable setelah ada user) — tanpa auth |
| `POST` | `/api/v1/auth/login` | Login dengan username/password, dapat token JWT |
| `POST` | `/api/v1/auth/refresh` | Perpanjang token yang hampir kadaluarsa |

### Contract Execution

| Method | Endpoint | Keterangan |
|---|---|---|
| `POST` | `/api/v1/invoke/:contract/:function` | Eksekusi state-changing logic, melalui consensus |
| `GET` | `/api/v1/query/:contract/:function` | Query read-only, tidak melalui consensus |

### Ledger & Records

| Method | Endpoint | Keterangan |
|---|---|---|
| `GET` | `/api/v1/records/:collection/:partition/:id` | Baca satu record |
| `GET` | `/api/v1/records/:collection` | Query dengan filter/sort/pagination |
| `GET` | `/api/v1/history/:collection/:id` | Histori lengkap semua versi record |
| `GET` | `/api/v1/blocks` | List blok terbaru |
| `GET` | `/api/v1/blocks/:num` | Detail satu blok |
| `GET` | `/api/v1/health` | Status node (tanpa auth) |

### Manajemen User (perlu role ADMIN)

| Method | Endpoint | Keterangan |
|---|---|---|
| `POST` | `/api/v1/admin/users/enroll` | Daftarkan user baru dengan peran |
| `GET` | `/api/v1/admin/users` | List semua user |
| `POST` | `/api/v1/admin/users/:name/revoke` | Cabut akses user (berlaku langsung) |

### Real-time

| Protocol | Endpoint | Keterangan |
|---|---|---|
| WebSocket | `/api/v1/ws` | Stream event block, tx, node_status |
| SSE | `/api/v1/events/stream` | Alternatif SSE untuk browser |

---

## 9. Query Engine

Query records mendukung operasi berikut:

**Filter operators:** `Eq`, `Neq`, `Gt`, `Lt`, `Gte`, `Lte`, `In`, `IsNull`, `IsNotNull`  
**Logical combinators:** `And`, `Or`, `Not`  
**Pagination:** `limit` dan `offset`  
**Soft-delete:** Record yang dihapus tidak muncul secara default; gunakan `include_deleted=true` untuk menampilkan

Query dikirim sebagai JSON body pada `GET /api/v1/records/:collection`.

---

## 10. Topologi Deployment

### Dev Mode (Single Node)

Node berjalan tanpa PKI dan tanpa consensus. Semua token diterima tanpa verifikasi. Cocok untuk development dan testing lokal. Transaction langsung dicommit tanpa melewati BFT.

**Keterbatasan:** Tidak ada fault tolerance. Tidak ada verifikasi identitas kriptografis.

### Production: Single Node

Node berjalan dengan PKI aktif (Dilithium3 CA). Semua token diverifikasi terhadap CA cert. Transaction melalui consensus (single-node consensus — commit langsung karena quorum 1/1 selalu terpenuhi).

**Keterbatasan:** Zero fault tolerance. Satu node mati = sistem mati.

### Production: Multi-Node (LAN)

Beberapa node di satu jaringan lokal (atau mesin yang sama). Peer discovery otomatis via mDNS. Validator set dikonfigurasi secara eksplisit via `--validator-keys`.

**Syarat mDNS:** Semua node harus di subnet yang sama. Tidak bisa digunakan antar data center atau cloud region berbeda.

**Fault tolerance:**
- 2 node: tidak ada (butuh quorum 2/2)
- 3 node: tolerir 1 crash (quorum 2/3)
- 4 node: tolerir 1 Byzantine fault (quorum 3/4)

### Topologi Node

| Role | Fungsi | Port Tipikal |
|---|---|---|
| `all` | Gateway + Validator + P2P | REST :8080, P2P :7051 |
| `validator` | HotStuff BFT + P2P | P2P :7051 |
| `gateway` | REST API, auth, contract exec | REST :8080 |
| `peer` | P2P sync saja | P2P :7051 |

Untuk produksi, setidaknya satu node harus menjalankan role `all` atau kombinasi `validator + gateway`. Node validator murni tidak melayani API.

---

## 11. Kontrak / Chaincode

Logika bisnis dikemas dalam unit yang disebut **contract** (atau chaincode). Setiap contract adalah kumpulan fungsi yang beroperasi pada ledger.

**Kontrak bawaan:**
- `hr-service` — manajemen data karyawan (hire, transfer, terminate, riwayat)

**Kemampuan fungsi contract:**
- Baca dan tulis record ke ledger (dengan RYOW — Read Your Own Writes dalam satu transaksi)
- Periksa identitas dan peran caller
- Emit event yang dapat didengar subscriber eksternal
- Validasi data sebelum commit

**Registrasi kontrak:** Kontrak dikompilasi bersama node sebagai Rust library native. Belum mendukung hot-deploy atau WASM isolation (roadmap Phase 6).

---

## 12. Keterbatasan Saat Ini

| Keterbatasan | Dampak | Status |
|---|---|---|
| mDNS discovery hanya LAN | Multi-node tidak bisa antar subnet / cloud region berbeda | Phase 5.2 |
| P2P tidak menggunakan PQ-TLS | Traffic antar validator tidak dilindungi enkripsi post-quantum | Phase 5.3 |
| Tidak ada MVCC validation | Race condition bisa terjadi pada transaksi concurrent yang mengubah record sama | Evaluasi |
| Kontrak hanya native Rust | Butuh recompile untuk deploy kontrak baru | Phase 6 |
| Metrics node belum tersedia | CPU, memory, latency tidak terekspos via API | Belum dijadwalkan |
| Belum ada HSM support | Private key CA disimpan di filesystem (encrypted) | Phase 8 |

---

## 13. Roadmap

| Phase | Status | Cakupan |
|---|---|---|
| 1–4 | ✅ Selesai | Crypto engine, MVP ledger, HotStuff BFT, PQ-PKI |
| 5 | ✅ Selesai | REST auth, user management, role-based node, Explorer UI |
| 5.2 | ⏳ Planned | Bootstrap peer dial untuk cross-network P2P |
| 5.3 | ⏳ Planned | PQ-TLS di layer P2P (autentikasi cert node-to-node) |
| 6 | ⏳ Planned | WASM chaincode isolation (hot-deploy kontrak) |
| 7 | ⏳ Planned | Docker, Helm chart, Prometheus metrics |
| 8 | ⏳ Planned | HSM integration, cross-org federation |

---

## 14. Glosarium

| Istilah | Definisi |
|---|---|
| **Ledger** | Penyimpanan data yang immutable dan append-only |
| **Block** | Unit komitmen berisi satu atau lebih transaksi yang sudah dikonfirmasi |
| **World State** | Snapshot kondisi terkini semua data (tanpa histori) |
| **Read-Write Set** | Output simulasi transaksi: daftar key yang dibaca dan perubahan yang akan ditulis |
| **BFT** | Byzantine Fault Tolerant — sistem tetap benar meski ada node yang berperilaku jahat |
| **Quorum** | Jumlah minimum validator yang harus setuju agar transaksi dikonfirmasi |
| **Validator** | Node yang berpartisipasi dalam consensus dan memverifikasi transaksi |
| **Gateway** | Node yang melayani REST API dan menjadi titik masuk client |
| **Chaincode / Contract** | Logika bisnis yang berjalan di dalam execution environment ledger |
| **RYOW** | Read Your Own Writes — dalam satu transaksi, baca setelah tulis melihat nilai yang baru ditulis |
| **Dilithium3** | Algoritma tanda tangan post-quantum (ML-DSA), NIST FIPS 204 |
| **Kyber768** | Algoritma key encapsulation post-quantum (ML-KEM), NIST FIPS 203 |
| **BLAKE3** | Algoritma hashing kriptografis berkecepatan tinggi |
| **mDNS** | Multicast DNS — protokol discovery peer otomatis di jaringan lokal |
| **gossipsub** | Protokol pub/sub P2P untuk distribusi pesan di jaringan validator |
| **CRL** | Certificate Revocation List — daftar sertifikat yang telah dicabut |
| **RBAC** | Role-Based Access Control — kontrol akses berdasarkan peran |

---

*Apache License 2.0*
