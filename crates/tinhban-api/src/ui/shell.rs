//! Khung trang dùng chung: `<head>`, CSS, thanh điều hướng.
//!
//! Toàn bộ UI là **SSR thuần**: server dựng sẵn HTML bằng `rsx!` rồi trả về
//! chuỗi. Không có JavaScript, không hydration — form gửi bằng POST thường và
//! server trả redirect. Đơn giản, và hợp với việc chỉ dùng nội bộ qua Tailscale.

use dioxus::prelude::*;

/// CSS toàn app — nhúng thẳng vào `<head>` để không phải phục vụ file tĩnh.
pub const CSS: &str = r#"
:root {
  --bg: #fbfaf7; --fg: #23201c; --muted: #6b6155; --line: #ddd6ca;
  --card: #fff; --accent: #7a1f1f; --accent-soft: #f3e9e9;
  --good: #1f6b3a; --good-soft: #e8f3ec; --bad: #9a2222; --bad-soft: #f8ebeb;
}
* { box-sizing: border-box; }
body {
  margin: 0; background: var(--bg); color: var(--fg);
  font: 15px/1.6 system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
}
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
header.top {
  background: var(--accent); color: #fff; padding: .6rem 1rem;
  display: flex; gap: 1.2rem; align-items: baseline; flex-wrap: wrap;
}
header.top .brand { font-weight: 700; font-size: 1.1rem; }
header.top a { color: #fff; opacity: .9; }
header.top a:hover { opacity: 1; }
main { max-width: 1100px; margin: 1.5rem auto; padding: 0 1rem; }
h1 { font-size: 1.5rem; margin: 0 0 1rem; }
h2 { font-size: 1.15rem; margin: 1.6rem 0 .6rem; }
h3 { font-size: 1rem; margin: 1.2rem 0 .4rem; }
.card {
  background: var(--card); border: 1px solid var(--line);
  border-radius: 6px; padding: 1rem; margin-bottom: 1rem;
}
.muted { color: var(--muted); }
.small { font-size: .86rem; }
table { border-collapse: collapse; width: 100%; }
th, td { border: 1px solid var(--line); padding: .45rem .6rem; text-align: left; vertical-align: top; }
th { background: #f4efe7; font-weight: 600; }
label { display: block; margin: .7rem 0 .2rem; font-weight: 600; font-size: .9rem; }
input, select, textarea {
  width: 100%; padding: .45rem .55rem; border: 1px solid var(--line);
  border-radius: 4px; font: inherit; background: #fff; color: var(--fg);
}
textarea { min-height: 5rem; resize: vertical; }
button, .btn {
  display: inline-block; padding: .5rem .9rem; border: 1px solid var(--accent);
  background: var(--accent); color: #fff; border-radius: 4px; cursor: pointer;
  font: inherit;
}
button:hover, .btn:hover { filter: brightness(1.1); text-decoration: none; }
.btn-ghost { background: #fff; color: var(--accent); }
.row { display: flex; gap: 1rem; flex-wrap: wrap; }
.row > * { flex: 1 1 12rem; }
.pill {
  display: inline-block; padding: .1rem .5rem; border-radius: 999px;
  font-size: .78rem; border: 1px solid var(--line); background: #f4efe7;
}
.pill.good { background: var(--good-soft); border-color: #bcd9c6; color: var(--good); }
.pill.bad { background: var(--bad-soft); border-color: #e0bcbc; color: var(--bad); }

/* --- Bàn 12 cung Tử Vi: lưới 4x4, 12 ô viền ngoài, giữa là 2x2 thông tin --- */
.diaban {
  display: grid; grid-template-columns: repeat(4, 1fr);
  grid-template-rows: repeat(4, minmax(8.5rem, auto));
  gap: 3px; background: var(--line); border: 3px solid var(--line);
  border-radius: 4px; margin: 1rem 0;
}
.cung { background: var(--card); padding: .4rem .5rem; overflow: hidden; }
.cung .ten { font-weight: 700; font-size: .92rem; }
.cung .chi { float: right; color: var(--muted); font-size: .82rem; }
.cung.menh { background: #fdf6ec; box-shadow: inset 0 0 0 2px var(--accent); }
.cung .sao { margin-top: .3rem; font-size: .84rem; line-height: 1.45; }
.cung .sao .ct { font-weight: 700; color: var(--accent); }
.cung .sao .pt { color: #3a3a6b; }
.cung .ts { margin-top: .3rem; font-size: .76rem; color: var(--muted); }
.giua {
  grid-column: 2 / 4; grid-row: 2 / 4; background: #fdfbf6;
  padding: .8rem; font-size: .86rem;
}
.giua dl { display: grid; grid-template-columns: auto 1fr; gap: .2rem .6rem; margin: 0; }
.giua dt { color: var(--muted); }
.giua dd { margin: 0; font-weight: 600; }

/* --- Nội dung markdown của từ điển --- */
.md h2 { border-bottom: 1px solid var(--line); padding-bottom: .2rem; }
.md table { margin: .6rem 0; }
.md code { background: #f0ece4; padding: .1rem .3rem; border-radius: 3px; }
.md blockquote {
  margin: .8rem 0; padding: .5rem .9rem; border-left: 3px solid var(--line);
  background: #f7f3ec; color: var(--muted);
}
.flash { padding: .6rem .9rem; border-radius: 4px; margin-bottom: 1rem; }
.flash.err { background: var(--bad-soft); border: 1px solid #e0bcbc; color: var(--bad); }
"#;

/// Bọc nội dung một trang vào khung HTML đầy đủ.
pub fn trang(tieu_de: &str, noi_dung: Element) -> String {
    let body = dioxus::ssr::render_element(rsx! {
        header { class: "top",
            span { class: "brand", "Tinh Bàn" }
            a { href: "/la-so/moi", "Lập lá số" }
            a { href: "/ho-so", "Hồ sơ" }
            a { href: "/tu-dien", "Từ điển" }
            a { href: "/ngay-tot-xau", "Ngày tốt/xấu" }
        }
        main { {noi_dung} }
    });
    format!(
        "<!doctype html>\n<html lang=\"vi\"><head>\
         <meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{} · Tinh Bàn</title>\
         <style>{}</style></head><body>{}</body></html>",
        html_escape(tieu_de),
        CSS,
        body
    )
}

/// Escape tối thiểu cho text nhúng vào HTML thô (chỉ dùng cho `<title>`;
/// mọi nội dung khác đi qua `rsx!` vốn đã tự escape).
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
