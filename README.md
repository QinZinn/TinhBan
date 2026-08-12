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

## Roadmap (giai đoạn sau)
- `tinhban-core`: lịch âm, an sao Tử Vi Đẩu Số, Bát Tự, từ điển, xem ngày.
- `tinhban-db`: bảng hồ sơ người được xem + lá số, cache ngày tốt/xấu.
- `tinhban-api`: routes/endpoint thật, có thể bật client hydration.