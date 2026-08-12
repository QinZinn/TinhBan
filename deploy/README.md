# Deploy Tinh Bàn lên homeserver

> Đây là **note/hướng dẫn thủ công** cho agent deploy (Hermes) hoặc khi tự deploy
> tay. Giai đoạn 1 KHÔNG chạy các bước này — chỉ chuẩn bị sẵn file. Thiết kế:
> KHÔNG Docker, app chạy trực tiếp dạng Rust binary, quản bằng systemd.

## Yêu cầu trên server (Ubuntu Server, ThinkPad T450s)
- Rust toolchain (stable). Cài qua `rustup`:
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- `cc` (C compiler) — vì sqlx dùng **bundled SQLite** (`sqlite` feature → build
  `libsqlite3-sys` tĩnh). Ubuntu:
  ```sh
  sudo apt-get update && sudo apt-get install -y build-essential
  ```

## Cách A — Build trực tiếp trên server (đơn giản nhất cho 1 máy)
```sh
sudo mkdir -p /opt/tinhban
sudo chown -R $USER:$USER /opt/tinhban

cd /opt/tinhban
git clone https://github.com/QinZinn/TinhBan.git repo
cd repo

cargo build --release -p tinhban-api
# Binary: target/release/tinhban-api
```

## Cách B — Build ở máy dev rồi copy binary sang server
```sh
# máy dev
cargo build --release -p tinhban-api
scp target/release/tinhban-api  <server>:/opt/tinhban/bin/tinhban-api
scp -r crates/tinhban-db/migrations <server>:/opt/tinhban/   # KHÔNG cần thiết:
# migrations đã được compile/embed vào binary nhờ `sqlx::migrate!`.
```
> Migration được embed vào binary lúc build, nên server **không cần** thư mục
> `migrations/`. Copy duy nhất 1 file binary là đủ.

## Cấu hình environment
```sh
sudo install -d -m 0750 /etc/tinhban
sudo tee /etc/tinhban/tinhban.env >/dev/null <<'EOF'
DATABASE_URL=sqlite:/var/lib/tinhban/tinhban.db?mode=rwc
PORT=8080
IP=127.0.0.1
RUST_LOG=info,tower_http=info
EOF

sudo install -d -o $USER -g $USER /var/lib/tinhban   # nơi lưu file DB
```
Sửa `DATABASE_URL` cho đúng user/thư mục nếu chạy dưới user riêng.

## Cài service systemd
```sh
sudo install -m 0644 deploy/tinhban.service /etc/systemd/system/tinhban.service
# (chỉnh lại ExecStart/EnvironmentFile cho đúng đường dẫn server trước khi install)
sudo systemctl daemon-reload
sudo systemctl enable --now tinhban.service
sudo systemctl status tinhban.service
journalctl -u tinhban -f          # xem log
```

## Healthcheck
```sh
curl 127.0.0.1:8080/health        # -> {"status":"ok"}
curl 127.0.0.1:8080/api/version
```

## Expose qua Tailscale (không mở port ra LAN/public)
App bind `127.0.0.1:8080`. Dùng `tailscale serve` để proxy từ tailnet vào:
```sh
# chỉ máy đã cài tailscale + đã `tailscale up`
sudo tailscale serve --bg --https 443 http://127.0.0.1:8080
tailscale serve status
```
Sau đó truy cập `https://<tên-máy-tailnet>` từ thiết bị khác trong tailnet. Để tắt:
```sh
sudo tailscale serve reset
```

## Khi cập nhật code
```sh
cd /opt/tinhban/repo && git pull
cargo build --release -p tinhban-api
sudo systemctl restart tinhban.service
```
> Vì `panic = "abort"` + `Restart=on-failure`, server tự dậy lại nếu sự cố.
> Database sót migration cũ sẽ được `sqlx::migrate!` chạy thêm khi boot.