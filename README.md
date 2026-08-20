# Tinh Bàn

Toolkit tử vi cá nhân, self-hosted: tạo lá số Tử Vi Đẩu Số & Bát Tự (Tứ Trụ),
từ điển tử vi, xem ngày tốt/xấu, và lưu hồ sơ những người đã được xem. Dự án cá
nhân, **không có auth multi-user theo thiết kế**, **không dùng Docker** — chạy
trực tiếp dạng Rust binary, quản bằng systemd.

**Trạng thái: hoàn thành 6/6 giai đoạn — sẵn sàng deploy.**

| # | Giai đoạn | Nội dung |
|---|---|---|
| 1 | Scaffolding | workspace, Axum, SQLite migration, khung SSR |
| 2 | Lõi âm lịch | thuật toán Hồ Ngọc Đức (UTC+7), Can Chi, 1900–2100 |
| 3 | Tử Vi Đẩu Số | 12 cung, Cục, 14 chính tinh + 13 phụ tinh, vòng Trường Sinh |
| 4 | Bát Tự | Tứ Trụ theo tiết khí, Tàng Can, Thập Thần, Ngũ Hành |
| 5 | Trạch Nhật | Hoàng Đạo/Hắc Đạo, giờ tốt, 12 Trực, kiêng kỵ + scrape bổ sung |
| 6 | Từ điển · Hồ sơ · UI · Deploy | 39 mục từ điển (FTS5), CRUD hồ sơ, giao diện web, gói deploy |

Ngoài ra có một đợt **audit Bug #7** (hằng số epoch) rà lại ảnh hưởng ngược tới
giai đoạn 3 & 4 — xem cuối README.

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
│   ├── tinhban-core/          logic nghiệp vụ thuần (không web, không DB)
│   │   ├── astronomy.rs       toán thiên văn Hồ Ngọc Đức (sóc, kinh độ Mặt Trời)
│   │   ├── canchi.rs          Can Chi năm/tháng/ngày/giờ + Ngũ Hành
│   │   ├── tuvi/              engine Tử Vi Đẩu Số (giai đoạn 3)
│   │   ├── bat_tu/            engine Bát Tự + tiết khí (giai đoạn 4)
│   │   └── trach_nhat/        engine ngày tốt/xấu (giai đoạn 5) — có README riêng
│   ├── tinhban-db/            SQLite: pool + migration (embed) + truy vấn
│   │   └── migrations/        sqlx migrate (compile vào binary) — 4 migration
│   └── tinhban-api/           server binary: Axum + Dioxus SSR (frontend gộp vào đây)
│       ├── src/scrape/        scrape nguồn phụ (licham365.vn)
│       ├── src/ui/            trang SSR: khung, lá số, từ điển, ngày tốt/xấu
│       ├── src/routes.rs      toàn bộ route HTML + API JSON
│       ├── src/ho_so.rs       lập lá số + lưu hồ sơ
│       └── src/tu_dien.rs     nạp markdown từ điển vào SQLite
├── content/tu-dien/           39 mục từ điển (markdown, nhúng vào binary)
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

### Vì sao SSR thuần (không hydration, không JavaScript)?

Toàn bộ giao diện được **dựng sẵn ở server**: handler Axum gọi `rsx!` rồi render
ra chuỗi HTML bằng `dioxus::ssr::render_element`. Form gửi bằng POST thường,
server trả redirect 303 (Post/Redirect/Get). Không có wasm, không hydration,
không server function.

Ở đây Dioxus đóng vai **thư viện template** chứ không phải framework frontend;
phần phục vụ HTTP là Axum thuần.

Đánh đổi có chủ đích:

- `cargo build` là đủ — **không cần** `dx` CLI hay target `wasm32-unknown-unknown`.
  Quan trọng với homeserver T450s tài nguyên hạn chế.
- Đúng **1 binary native**, đúng 1 service systemd.
- Không có tương tác client-side (lọc tại chỗ, cập nhật không tải lại trang).
  Với app tra cứu dùng một mình thì tải lại trang là chấp nhận được.

Feature `web = ["dioxus/web"]` vẫn khai báo sẵn cho tương lai, nhưng chưa dùng.
Nếu sau này cần interactivity thật thì thêm target wasm + `dx` CLI và chuyển các
dependency server-only sang `optional` theo hướng dẫn [Project Setup](https://dioxuslabs.com/learn/0.7/essentials/fullstack/project_setup/#adding-server-only-dependencies).

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

Không cần `dx` CLI, không cần target wasm — app là SSR thuần nên `cargo run` là đủ.
Check nhanh:
```sh
curl localhost:8080/health        # {"status":"ok"}
curl localhost:8080/api/version
curl localhost:8080/api/health    # {status, db, tu_dien_muc, version, app}

# Ngày tốt/xấu. Thiếu `date` → hôm nay theo giờ VN.
curl 'localhost:8080/api/ngay-tot-xau?date=2024-03-15'

# Từ điển (gõ không dấu vẫn ra).
curl 'localhost:8080/api/tu-dien?q=dao+hoa'
curl  localhost:8080/api/tu-dien/sao-tu-vi

# Hồ sơ.
curl -X POST localhost:8080/api/ho-so -H 'content-type: application/json' \
  -d '{"ten":"A","ngay_sinh":"1991-10-24","gio":7,"gioi_tinh":"nam"}'
curl localhost:8080/api/ho-so
```
File SQLite tạo tự ở `./data/tinhban.db` (do `DATABASE_URL` mặc định). `data/`
đã được `.gitignore`.

Build toàn workspace + test:
```sh
cargo build --workspace
cargo test --workspace
```

## Triển khai lên server thật

Xem [`deploy/README.md`](deploy/README.md) và mục
[Bàn giao cho bước deploy](#bàn-giao-cho-bước-deploy) ở cuối README này.

Repo **chỉ chuẩn bị sẵn file**, không tự chạy bước nào lên server — việc deploy
thật do agent deploy (Hermes) thực hiện ở bước riêng.

## Thiết kế / ghi chú
- **Không auth**: 1 user, nội bộ qua Tailscale. CORS để **permissive cố ý** (chỉ
  chạy trong tailnet, không ra public internet) — xem comment trong
  `crates/tinhban-api/src/main.rs`.
- **Logging**: `tracing`/`tracing-subscriber` → stdout (journalctl thu lại), kèm
  `TraceLayer` để thấy request trong `journalctl -u tinhban`.
- **DB**: `DATABASE_URL` env, mặc định `sqlite:data/tinhban.db?mode=rwc`, không
  hardcode. 4 migration: `app_meta` → `licham365_cache` → `tu_dien` (+ FTS5) →
  `ho_so`. Chạy tự động lúc khởi động, đã kiểm tra sạch từ DB rỗng.
- **Bundled SQLite**: build cần `cc`; binary release tự chứa SQLite (server
  không cần cài libsqlite3).
- **Binary tự chứa mọi thứ**: migration SQL, 39 mục từ điển, SQLite và CSS đều
  nhúng lúc build → deploy chỉ copy đúng một file.
- **Không JavaScript**: xem mục "Vì sao SSR thuần" ở trên.

## Tính năng

Truy cập qua trình duyệt (SSR thuần, **không JavaScript**):

| Trang | Đường dẫn | Làm gì |
|---|---|---|
| Trang chủ | `/` | điểm vào, thống kê nhanh |
| Lập lá số | `/la-so/moi` | nhập tên + ngày giờ sinh + giới tính → tính **Tử Vi và Bát Tự cùng lúc**, lưu thành hồ sơ |
| Chi tiết hồ sơ | `/ho-so/:id` | bàn 12 cung Tử Vi + bảng Tứ Trụ Bát Tự, sửa ghi chú, xoá |
| Danh sách hồ sơ | `/ho-so` | các người đã xem |
| Từ điển | `/tu-dien?q=` | tra 39 mục; **gõ không dấu vẫn tìm được** |
| Ngày tốt/xấu | `/ngay-tot-xau?date=` | Hoàng Đạo/Hắc Đạo, giờ tốt, Trực, kiêng kỵ + diễn giải |

Tên sao trên lá số **bấm được** → nhảy thẳng sang mục từ điển tương ứng.

### API JSON

```
GET    /health                      GET    /api/health      GET /api/version
GET    /api/tu-dien?q=              GET    /api/tu-dien/:slug
POST   /api/ho-so                   GET    /api/ho-so        GET /api/ho-so/:id
PATCH  /api/ho-so/:id               DELETE /api/ho-so/:id
GET    /api/ngay-tot-xau?date=YYYY-MM-DD
```

## Từ điển tử vi — giai đoạn 6

39 mục markdown trong [`content/tu-dien/`](content/tu-dien/): **14 chính tinh +
13 phụ tinh + 12 cung**. Nội dung đối chiếu với nguồn công khai, và **ghi rõ chỗ
các nguồn mâu thuẫn nhau** thay vì chọn im lặng — xem
[`content/tu-dien/README.md`](content/tu-dien/README.md).

Ba điểm đáng chú ý:

- **Phạm vi khoá vào code.** Test `tu_dien_phu_het_sao_va_cung_da_implement` đối
  chiếu từng slug với `Sao::ALL` và `PalaceName::all_in_order()`. Thêm sao vào
  engine mà quên viết mục từ điển → đỏ; viết mục cho sao chưa implement → cũng đỏ.
- **Tìm kiếm không dấu.** FTS5 với text đã tự chuẩn hoá ở tầng Rust, vì tokenizer
  `remove_diacritics` của SQLite bỏ được dấu thanh nhưng **không** map `đ` → `d`
  (đ là chữ cái riêng). Không xử lý thì gõ "dao hoa" sẽ trượt "Đào Hoa".
- **Nhúng vào binary.** `include_dir!` gói cả thư mục vào file thực thi, nên
  deploy chỉ cần copy một file. Đặt `TINHBAN_CONTENT_DIR` để đọc từ đĩa khi cần
  sửa nội dung mà không build lại.

## Hồ sơ người đã xem — giai đoạn 6

1 người / 1 lần xem = 1 bản ghi. Lá số Tử Vi và Bát Tự được **lưu sẵn dạng JSON**
để hiển thị lại không cần tính lại.

Bảng `ho_so` có cột `engine_ver` ghi phiên bản engine đã sinh ra lá số. Lý do rất
cụ thể: Bug #7 từng làm sai trụ Năm/Tháng Bát Tự cho ngày sinh rơi đúng mốc tiết
khí. Nếu sau này lại có lỗi tương tự, cột này cho biết hồ sơ nào cần tính lại
thay vì phải đoán. Khi mở hồ sơ được tính bằng bản cũ hơn, UI hiện cảnh báo —
**không tự động tính lại**, vì lá số cũ là dữ liệu người dùng đã thấy.

Không sửa được ngày giờ sinh của hồ sơ đã lưu (chỉ sửa tên và ghi chú): đổi ngày
sinh nghĩa là một lá số khác, nên phải lập hồ sơ mới thay vì để lại bản ghi mâu
thuẫn giữa thông tin và lá số.

## Xem ngày tốt/xấu (Trạch Nhật) — giai đoạn 5

Tính năng ghép **hai nguồn**, và ranh giới giữa chúng là điểm thiết kế quan trọng
nhất:

| | Nguồn chính | Nguồn phụ |
|---|---|---|
| Là gì | `tinhban-core/trach_nhat` — tự tính | scrape licham365.vn |
| Cho ra | Hoàng Đạo/Hắc Đạo, giờ tốt, 12 Trực, kiêng kỵ | văn bản diễn giải dân gian chi tiết |
| Cần mạng | Không | Có (lần đầu mỗi ngày) |
| Hỏng thì sao | Không hỏng được | `dien_giai: null` + `ghi_chu` giải thích |

**Nguyên tắc: nguồn phụ không bao giờ được làm hỏng nguồn chính.** Mạng chết,
site sập, hay licham365 đổi HTML đều chỉ làm `dien_giai` thành `null` kèm ghi
chú — không bao giờ thành 5xx hay JSON rỗng khó hiểu.

Chi tiết thuật toán + kết quả đối chiếu: [`crates/tinhban-core/src/trach_nhat/README.md`](crates/tinhban-core/src/trach_nhat/README.md).

### Endpoint

```sh
curl 'localhost:8080/api/ngay-tot-xau?date=2024-03-15'
```

- `date` không bắt buộc → mặc định **hôm nay theo giờ VN (UTC+7)**, không theo
  timezone của máy chủ.
- `200` kể cả khi scrape hỏng; `400` khi `date` sai định dạng hoặc ngoài 1900–2100.
- Trường `nguon_dien_giai` cho biết diễn giải đến từ `"cache"` hay `"scrape"`.

### Nguồn scrape

- **Nguồn**: licham365.vn
- **URL pattern**: `/lich-am-ngay-{D}-thang-{M}-nam-{YYYY}` — ngày/tháng **không
  đệm số 0** (`.../lich-am-ngay-5-thang-3-nam-2024`).
- **Cách bóc**: **không** bám selector riêng cho từng mục. Trang chia nội dung
  thành các khối `div.c-de`, mỗi khối có tiêu đề `h3`; ta duyệt mọi khối, lấy
  `h3` làm tên mục và phần text còn lại làm nội dung. Nhờ vậy site thêm/đổi
  tên/sắp xếp lại mục thì vẫn lấy được.
- **Lễ độ**: chỉ scrape **theo yêu cầu thật** cho đúng ngày người dùng xem —
  không cào hàng loạt, không pre-cache cả năm. Cache vĩnh viễn nên mỗi ngày chỉ
  tải đúng một lần trong suốt vòng đời app. User-Agent khai báo trung thực là bot
  của dự án kèm link repo, **không giả mạo trình duyệt**. Timeout 10s.

### Cấu trúc cache

Bảng `licham365_cache` (migration `0002`), khoá chính là ngày Dương lịch:

| Cột | Ý nghĩa |
|---|---|
| `solar_date` | `'YYYY-MM-DD'`, 1 bản ghi / ngày |
| `status` | `'ok'` hoặc `'error'` |
| `payload` | JSON các mục diễn giải (NULL khi lỗi) |
| `error` | mô tả lỗi (NULL khi ok) |
| `source_url` | URL đã gọi |
| `fetched_at` | ISO-8601 UTC |

Chính sách:

| Trạng thái cache | Hành vi |
|---|---|
| có, `ok` | Dùng luôn, **không gọi mạng** |
| có, `error`, < 1 giờ | Không thử lại, trả fallback kèm lỗi đã lưu |
| có, `error`, ≥ 1 giờ | Thử scrape lại |
| chưa có | Scrape, rồi lưu kết quả (kể cả lỗi) |

Bản ghi `ok` **không có TTL** vì nội dung của một ngày cụ thể là tĩnh. Chỉ bản
ghi `error` mới có TTL — một lần site sập không được đóng băng ngày đó vĩnh
viễn, nhưng cũng không nên thử lại mỗi lần bấm F5. Bản ghi lỗi **không bao giờ
ghi đè** lên bản ghi tốt đã có.

Đo thực tế: lần đầu ~476 ms (có gọi mạng), lần hai ~2 ms (đọc cache).

### Khi licham365 đổi cấu trúc HTML

**Dấu hiệu nhận biết** — trong `journalctl -u tinhban` xuất hiện:

```
WARN scrape licham365 thất bại — rơi về kết quả tự tính date=... error=...
```

với `error` là một trong hai:

- `không tìm thấy khối nội dung nào khớp selector "div.c-de"` → site đã đổi hẳn
  khung HTML;
- `chỉ bóc được N mục (kỳ vọng ít nhất 4)` → đổi một phần, hoặc trang trả bản
  rút gọn.

**Chỗ cần sửa**: đúng hai hằng ở đầu `crates/tinhban-api/src/scrape/licham365.rs`:

```rust
pub const SECTION_BLOCK_SELECTOR: &str = "div.c-de";
pub const SECTION_TITLE_SELECTOR: &str = "h3";
```

Sau khi sửa, chạy `cargo test -p tinhban-api` — có test đối chiếu trên một trang
thật đã lưu (`tests/fixtures/licham365-2024-03-15.html`), chạy offline.

Trong lúc chưa sửa, tính năng **vẫn dùng được**: phần tự tính không phụ thuộc
licham365 chút nào.

### Kiểm thử fallback

Không cần sửa code — trỏ biến môi trường sang host không tồn tại:

```sh
LICHAM365_BASE_URL=https://khong-ton-tai.invalid cargo run -p tinhban-api
```

rồi gọi một ngày **chưa** có trong cache. Kỳ vọng: HTTP `200`, `tu_tinh` đầy đủ,
`dien_giai: null`, `ghi_chu` nêu lý do, và log có dòng `WARN`. Ngày **đã** cache
vẫn trả diễn giải bình thường.

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

## Bàn giao cho bước deploy

Dự án **đã sẵn sàng** để agent deploy (Hermes) clone về homeserver và setup.
Hướng dẫn đầy đủ: [`deploy/README.md`](deploy/README.md).

Tóm tắt việc agent deploy cần làm:

1. **Chuẩn bị máy** — cài `rustup` và `build-essential` (cần `cc` vì SQLite build
   tĩnh vào binary). Không cần Docker/Node/wasm.
2. **Build** — `cargo build --release -p tinhban-api`.
3. **Đặt binary** — copy `target/release/tinhban-api` sang
   `/opt/tinhban/bin/`. **Chỉ một file này** — migration SQL, 39 mục từ điển,
   SQLite và CSS đều đã nhúng sẵn bên trong; **không cần copy `content/`**.
4. **Tạo user + file env** — user hệ thống `tinhban`, và
   `/etc/tinhban/tinhban.env` chứa `DATABASE_URL` (trỏ vào
   `/var/lib/tinhban/tinhban.db`), `PORT`, `IP=127.0.0.1`, `RUST_LOG`.
5. **Cài systemd** — copy [`deploy/tinhban.service`](deploy/tinhban.service),
   `daemon-reload`, `enable --now`. File service đã có `StateDirectory=tinhban`
   để `/var/lib/tinhban` ghi được dưới `ProtectSystem=strict`.
6. **Expose qua Tailscale** — `tailscale serve --bg --https 443
   http://127.0.0.1:8080`. Giữ app bind localhost, **đừng** dùng `tailscale
   funnel` (app không có auth theo thiết kế).
7. **Nghiệm thu** — `curl 127.0.0.1:8080/api/health` phải trả `"db":"ok"` và
   `"tu_dien_muc":39`.

Migration chạy tự động lúc khởi động; đã kiểm tra chạy sạch từ DB rỗng qua đủ
4 migration (`app_meta`, `licham365_cache`, `tu_dien` + FTS5, `ho_so`).

## Ngoài phạm vi (cố ý)

Các mục dưới đây bị loại trừ có chủ đích qua các giai đoạn, không phải bỏ sót:

- **Luận giải sâu**: Tứ Hoá, Đại Vận / Tiểu Hạn (Tử Vi); Vượng/Suy, Đại Vận
  (Bát Tự). Engine chỉ *an sao* và *lập trụ*, không phán đoán.
- **Sao chưa implement**: Địa Không, Địa Kiếp (2/6 Lục Sát còn lại).
- **Từ điển thuật ngữ Bát Tự** và 12 sao vòng Trường Sinh.
- **Authentication** — tailnet là lớp kiểm soát truy cập.
- **Responsive/mobile** — ưu tiên desktop, dùng qua Tailscale từ máy tính.
- `tinhban-api`: routes/endpoint thật, có thể bật client hydration.