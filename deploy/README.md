# Deploy Tinh Bàn lên homeserver

> Hướng dẫn cho **agent deploy (Hermes)** hoặc khi tự deploy tay. Repo này chỉ
> **chuẩn bị sẵn** file, không tự chạy bước nào lên server.
>
> Thiết kế: **không Docker**, app chạy trực tiếp dạng Rust binary, quản bằng
> systemd, expose qua Tailscale.

## TL;DR — 6 bước

```sh
# 1. build
cargo build --release -p tinhban-api

# 2. đặt binary  (đây là file DUY NHẤT cần copy)
sudo install -D -m 0755 target/release/tinhban-api /opt/tinhban/bin/tinhban-api

# 3. tạo user chạy service
sudo useradd -r -s /usr/sbin/nologin -d /var/lib/tinhban tinhban

# 4. đặt file môi trường
sudo install -d -m 0750 /etc/tinhban
sudo tee /etc/tinhban/tinhban.env >/dev/null <<'EOF'
DATABASE_URL=sqlite:/var/lib/tinhban/tinhban.db?mode=rwc
PORT=8080
IP=127.0.0.1
RUST_LOG=info,tower_http=info
EOF

# 5. cài systemd service
sudo install -m 0644 deploy/tinhban.service /etc/systemd/system/tinhban.service
sudo systemctl daemon-reload
sudo systemctl enable --now tinhban.service

# 6. expose qua tailnet
sudo tailscale serve --bg --https 443 http://127.0.0.1:8080
```

Kiểm tra: `curl 127.0.0.1:8080/api/health` phải trả `"db":"ok"` và
`"tu_dien_muc":39`.

---

## Yêu cầu trên server (Ubuntu Server)

- **Rust toolchain** (stable), qua `rustup`:
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **`cc`** — sqlx dùng bundled SQLite (`libsqlite3-sys` build tĩnh):
  ```sh
  sudo apt-get update && sudo apt-get install -y build-essential
  ```
- **Tailscale** đã `tailscale up`.

Không cần: Docker, Node, `dx` CLI, target wasm, libsqlite3 hệ thống.

## Chỉ cần copy đúng MỘT file

Binary đã tự chứa mọi thứ:

| Thứ | Nhúng vào binary bằng | Có phải copy không |
|---|---|---|
| Migration SQL | `sqlx::migrate!` | **Không** |
| Nội dung từ điển (`content/tu-dien/`, 39 mục) | `include_dir!` | **Không** |
| SQLite | `libsqlite3-sys` bundled | **Không** |
| CSS giao diện | nhúng thẳng trong `<head>` | **Không** |

> Nếu muốn **sửa nội dung từ điển mà không build lại**: copy `content/tu-dien/`
> lên server rồi thêm `TINHBAN_CONTENT_DIR=/đường/dẫn/content/tu-dien` vào file
> env. App sẽ đọc từ đĩa thay cho bản nhúng. Nhớ thêm đường dẫn đó vào
> `ReadOnlyPaths=` của service nếu để ngoài `/var/lib/tinhban` (vì
> `ProtectSystem=strict`).

## Cách A — build trên server (đơn giản nhất)

```sh
sudo install -d -o $USER -g $USER /opt/tinhban
cd /opt/tinhban
git clone https://github.com/QinZinn/TinhBan.git repo
cd repo
cargo build --release -p tinhban-api
sudo install -D -m 0755 target/release/tinhban-api /opt/tinhban/bin/tinhban-api
```

T450s build lần đầu khá lâu (rustls + reqwest + dioxus). Lần sau nhanh hơn nhiều.

## Cách B — build ở máy dev rồi copy sang

Chỉ dùng được nếu máy dev **cùng kiến trúc** (x86_64 Linux, glibc không mới hơn
server). Nếu không chắc thì dùng Cách A.

```sh
cargo build --release -p tinhban-api
scp target/release/tinhban-api <server>:/tmp/tinhban-api
ssh <server> 'sudo install -D -m 0755 /tmp/tinhban-api /opt/tinhban/bin/tinhban-api'
```

## Biến môi trường

| Biến | Bắt buộc | Mặc định | Ghi chú |
|---|---|---|---|
| `DATABASE_URL` | nên đặt | `sqlite:data/tinhban.db?mode=rwc` | Trên server dùng đường dẫn tuyệt đối trong `/var/lib/tinhban`. `?mode=rwc` để tự tạo file. |
| `PORT` | không | `8080` | |
| `IP` | không | `127.0.0.1` | **Giữ localhost** — ra tailnet bằng `tailscale serve`, đừng bind `0.0.0.0`. |
| `RUST_LOG` | không | `info,tower_http=info` | |
| `TINHBAN_CONTENT_DIR` | không | (dùng bản nhúng) | Đọc từ điển từ đĩa thay vì bản nhúng. |
| `LICHAM365_BASE_URL` | không | `https://licham365.vn` | Chỉ để kiểm thử fallback. |

## Về file service

`deploy/tinhban.service` chạy dưới user `tinhban` với `ProtectSystem=strict`.
Thư mục ghi được **duy nhất** là `/var/lib/tinhban`, do `StateDirectory=tinhban`
tạo và cấp quyền tự động.

> ⚠️ **Đừng bỏ `StateDirectory`.** Bản service của giai đoạn 1 có
> `ProtectSystem=strict` nhưng không khai báo thư mục ghi được nào — nghĩa là
> toàn bộ filesystem read-only, SQLite không tạo nổi file DB và service chết
> ngay khi khởi động. Lỗi này đã được sửa ở giai đoạn 6.

Nếu đổi `DATABASE_URL` ra ngoài `/var/lib/tinhban`, phải thêm đường dẫn đó vào
`ReadWritePaths=`.

## Healthcheck

```sh
curl 127.0.0.1:8080/health        # {"status":"ok"}  — dùng cho systemd/monitor
curl 127.0.0.1:8080/api/health    # có kiểm tra DB + số mục từ điển
curl 127.0.0.1:8080/api/version
```

`/api/health` trả:
```json
{"app":"Tinh Bàn","db":"ok","status":"ok","tu_dien_muc":39,"version":"0.1.0"}
```
`db` phải là `"ok"` và `tu_dien_muc` phải là `39`. Nếu `tu_dien_muc` = 0 thì từ
điển không nạp được — xem `journalctl -u tinhban | grep 'từ điển'`.

## Expose qua Tailscale

App bind `127.0.0.1:8080`, không mở ra LAN.

```sh
sudo tailscale serve --bg --https 443 http://127.0.0.1:8080
tailscale serve status
```

Truy cập `https://<tên-máy-tailnet>` từ thiết bị khác trong tailnet.
Tắt: `sudo tailscale serve reset`.

> **Không có authentication** — đây là lựa chọn có chủ đích: app dùng cho đúng
> một người, và tailnet đã là lớp kiểm soát truy cập. Đừng expose ra internet
> công cộng (`tailscale funnel`) khi chưa thêm auth.

## Cập nhật code

```sh
cd /opt/tinhban/repo && git pull
cargo build --release -p tinhban-api
sudo install -D -m 0755 target/release/tinhban-api /opt/tinhban/bin/tinhban-api
sudo systemctl restart tinhban.service
```

Migration mới tự chạy khi boot. Dữ liệu cũ giữ nguyên.

## Sao lưu

Toàn bộ dữ liệu nằm trong **một file SQLite**:

```sh
sudo systemctl stop tinhban
sudo cp /var/lib/tinhban/tinhban.db ~/tinhban-backup-$(date +%F).db
sudo systemctl start tinhban
```

Hoặc nóng, không cần dừng service:
```sh
sudo -u tinhban sqlite3 /var/lib/tinhban/tinhban.db ".backup '/tmp/tinhban.db'"
```

Trong DB có: hồ sơ đã lưu (`ho_so`), cache scrape (`licham365_cache`), từ điển
(`tu_dien` — dựng lại được từ binary nên không cần backup).

## Gỡ rối

| Hiện tượng | Nguyên nhân thường gặp |
|---|---|
| Service chết ngay, log có `unable to open database file` | `DATABASE_URL` trỏ ra ngoài `/var/lib/tinhban` mà chưa thêm `ReadWritePaths` |
| `/api/health` trả `"tu_dien_muc":0` | `TINHBAN_CONTENT_DIR` trỏ sai chỗ — bỏ biến này đi để dùng bản nhúng |
| Trang ngày tốt/xấu thiếu phần "Diễn giải chi tiết" | Không ra được internet tới licham365.vn. **Không phải lỗi** — phần tự tính vẫn đủ, xem `ghi_chu` trên trang |
| Build lỗi `linker cc not found` | Thiếu `build-essential` |
