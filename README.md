# Tinh Bàn

Toolkit tử vi cá nhân, self-hosted: tạo lá số Tử Vi Đẩu Số & Bát Tự (Tứ Trụ),
từ điển tử vi, xem ngày tốt/xấu, và lưu hồ sơ những người đã được xem. Dự án cá
nhân, **không có auth multi-user theo thiết kế**, **không dùng Docker** — chạy
trực tiếp dạng Rust binary, quản bằng systemd.

> Giai đoạn 1 (repo này) chỉ là **scaffolding**: khung dự án chạy được, chưa có
> logic tử vi/lịch âm (để giai đoạn sau).

## Stack
- **Backend**: Rust + Axum (async, Tokio) — bên trong Dioxus fullstack.
- **Frontend**: Dioxus 0.7 fullstack, **chế độ SSR-only** (xem lý do bên dưới).
- **Database**: SQLite qua `sqlx` (bundled libsqlite3 → binary tự chứa).
- **Không Docker**. Không multi-user auth. Process management: systemd.

## Cấu trúc thư mục
```
.
├── Cargo.toml                 workspace root
├── crates/
│   ├── tinhban-core/          logic nghiệp vụ thuần (placeholder; sau chứa lịch âm, an sao, Bát Tự)
│   ├── tinhban-db/            SQLite: pool + migration (embed) + truy vấn app_meta
│   │   └── migrations/        sqlx migrate (compile vào binary)
│   └── tinhban-api/           server binary: Axum + Dioxus fullstack (frontend gộp vào đây)
├── deploy/
│   ├── tinhban.service        file systemd unit mẫu
│   └── README.md              note deploy thủ công + tailscale serve
├── .github/workflows/ci.yml   cargo build + test (khung sườn)
├── .env.example
└── README.md
```

### Vì sao `tinhban-web` gộp vào `tinhban-api`?
Yêu cầu cho phép gộp nếu dùng Dioxus fullstack. Lý do cụ thể:
- Dioxus fullstack chạy UI cùng server Axum → **đúng 1 binary**, đúng 1 service
  systemd (yêu cầu cốt lõi với homeserver tài nguyên hạn chế).
- Giai đoạn này UI cực kỳ đơn giản (1 trang chủ), tách crate riêng chỉ thêm phức
  tạp cross-crate (feature propagation của server function) mà chưa cần.

Nếu sau này UI phức tạp cần chia, ta tách `crates/tinhban-web` (lib UI) + để
`tinhban-api` chỉ giữ server bin — lúc đó làm theo ["Separate Frontend and
Backend Crates"](https://dioxuslabs.com/learn/0.7/essentials/fullstack/project_setup/#separate-frontend-and-backend-crates).

### Vì sao SSR-only (chưa có client hydration)?
Dioxus fullstack gồm 2 build: server (native) và client (wasm). Mặc định repo
này đặt `default = ["server"]`:
- `cargo build` / `cargo run` chỉ build server, **không cần** `wasm32-unknown-unknown`
  target hay `dx` CLI. Trang chủ được server-side render → "frontend load được,
  hiện trạng thái backend" đã thoả mãn ở giai đoạn 1.
- Giảm bề mặt toolchain/人力资源管理 cho homeserver (chỉ 1 native binary).

Khi cần interactivity thật (giai đoạn sau): thêm `wasm32` target + `dx` CLI, bật
feature `web`, và chuyển các server-only deps sang `optional` + enable trong
`server` feature (theo hướng dẫn [Project Setup](https://dioxuslabs.com/learn/0.7/essentials/fullstack/project_setup/#adding-server-only-dependencies)).
Feature `web = ["dioxus/web"]` đã khai báo sẵn (chưa dùng được ngay).

## Chạy dev local
Yêu cầu:
- Rust toolchain (stable) — `rustup`.
- `cc` (cho bundled SQLite) — Arch: `pacman -S base-devel`; Ubuntu: `apt install build-essential`.
- `sqlx-cli` (tuỳ chọn): chỉ cần nếu muốn chạy `sqlx migrate` bằng tay. Repo
  tự chạy migration khi server boot (embed qua `sqlx::migrate!`), nên build
  không cần DATABASE_URL, không cần offline cache.

Chạy:
```sh
cp .env.example .env          # tuỳ chọn, edit nếu muốn đổi PORT/DB
cargo run -p tinhban-api
# mở http://localhost:8080
```
Check nhanh:
```sh
curl localhost:8080/health        # {"status":"ok"}
curl localhost:8080/api/version
curl localhost:8080/api/health    # {status, db, version, app}
```
File SQLite tạo tự ở `./data/tinhban.db` (do `DATABASE_URL` mặc định). `data/`
đã được `.gitignore`.

Build toàn workspace + test:
```sh
cargo build --workspace
cargo test --workspace
```

## Triển khai lên server thật
Xem [`deploy/README.md`](deploy/README.md). Tóm gọn: `cargo build --release -p
tinhban-api` → copy binary → đặt `/etc/tinhban/tinhban.env` → install
`deploy/tinhban.service` → `systemctl enable --now tinhban` → expose qua
`tailscale serve`. Các file deploy ở giai đoạn này **chỉ là chuẩn bị**, không
tự chạy trên server.

## Thiết kế / ghi chú
- **Không auth**: 1 user, nội bộ qua Tailscale. CORS để **permissive cố ý** (chỉ
  chạy trong tailnet, không ra public internet) — xem comment trong
  `crates/tinhban-api/src/main.rs`.
- **Logging**: `tracing`/`tracing-subscriber` → stdout (journalctl thu lại), kèm
  `TraceLayer` để thấy request trong `journalctl -u tinhban`.
- **DB**: `DATABASE_URL` env, mặc định `sqlite:data/tinhban.db?mode=rwc`, không
  hardcode. Migration đầu tiên (`app_meta` key-value) chỉ để xác nhận pipeline.
- **Bundled SQLite**: build cần `cc`; binary release tự chứa SQLite (server
  không cần cài libsqlite3).

## Ghi chú hồi tố: Bug #7 (epoch sai) — ảnh hưởng ngược tới giai đoạn 4

Trong lúc làm giai đoạn 5, phát hiện hằng số epoch sai trong
`sun_longitude_deg_at_local_midnight` (`crates/tinhban-core/src/astronomy.rs`):
hàm đã trừ 0.5 ngày ở `real_jd` rồi lại dùng epoch `2451545.5` vốn **đã gộp sẵn**
nửa ngày đó → double-count → kinh độ Mặt Trời thấp hơn thực tế ~0.49°.

**Hàm này không phải code của giai đoạn 5** — nó được viết ra và dùng lần đầu ở
giai đoạn 4 (`bat_tu/tiet_khi.rs`), nơi nó quyết định ranh giới Lập Xuân (trụ
Năm) và ranh giới 12 tháng tiết khí (trụ Tháng). Báo cáo giai đoạn 4 khi đó có
ghi nhận triệu chứng nhưng quy nhầm cho "giới hạn độ chính xác của đa thức Hồ
Ngọc Đức". Đã audit lại toàn bộ trước khi commit giai đoạn 5.

| Câu hỏi | Kết luận |
|---|---|
| Chiều sai | Mốc tiết khí bị đẩy **TRỄ**, một chiều tuyệt đối. Trên 1900–2100 (2412 mốc): **50.0% trễ 1 ngày, 0 mốc sớm**. |
| Ai bị ảnh hưởng | **Bát Tự** (trụ Năm + trụ Tháng) và **12 Trực** của giai đoạn 5. |
| Ai KHÔNG bị | **Tử Vi** — chỉ dùng ngày Âm lịch, đi qua `sun_longitude_at_noon` (nhánh epoch đúng). Khoá bằng 2 test. |
| **Lịch Âm** có bị không | **Không** — `solar_to_lunar` dùng nhánh đúng. |
| Lá số Bát Tự sai bao nhiêu | Quét 1960–2030 (25 933 ngày): **35 ngày sai trụ Năm** (0.135%), **389 ngày sai trụ Tháng** (1.50%) → tổng **1.635%**. |
| Vùng rủi ro trụ Năm | Đúng ngày **3/2 hoặc 4/2** của 35 năm cụ thể (xem README `bat_tu/`) — lệch hẳn sang năm Can Chi liền trước. |
| Kiểm chứng bản sửa | Đối chiếu **Đài Thiên văn Hồng Kông**, 16 năm × 12 tiết, đã quy giờ HK→VN: **192/192 (100%)** sau khi sửa, 97/192 trước khi sửa. |
| 7 case Bát Tự cũ | **0/7 bị ảnh hưởng — nhưng nhờ may, không phải thiết kế**: case5 (2000-05-05) nằm ĐÚNG trên mốc Lập Hạ, chỉ thoát vì mốc đó tình cờ thuộc nửa không bị lệch. |
| Đã bổ sung | **11 case biên** nhắm thẳng vùng rủi ro; **8/11 case bị mã cũ tính sai**. |
| **Rủi ro thực tế** | **Bằng 0** — chưa có tầng lưu lá số (`tinhban-db` cho hồ sơ dự kiến giai đoạn 6), nên chưa lá số nào từng được tạo bằng mã lỗi. |

Chi tiết đầy đủ (bảng số liệu, danh sách 35 năm rủi ro, từng case biên):
[`crates/tinhban-core/src/bat_tu/README.md`](crates/tinhban-core/src/bat_tu/README.md#ghi-chú-hồi-tố-audit-bug-7-epoch-sai--ảnh-hưởng-ngược-tới-giai-đoạn-4).
Bằng chứng chạy được: [`crates/tinhban-core/tests/bug7_epoch_audit.rs`](crates/tinhban-core/tests/bug7_epoch_audit.rs).

> ⚠️ `sun_longitude_at_noon` dùng `2451545.5` là **đúng** — ở đó shift −0.5 được
> gộp thẳng vào hằng số. Hai hàm khác nhau ở chỗ đó, **đừng "đồng bộ" hằng số
> giữa chúng**.

## Roadmap (giai đoạn sau)
- `tinhban-core`: lịch âm, an sao Tử Vi Đẩu Số, Bát Tự, từ điển, xem ngày.
- `tinhban-db`: bảng hồ sơ người được xem + lá số, cache ngày tốt/xấu.
- `tinhban-api`: routes/endpoint thật, có thể bật client hydration.