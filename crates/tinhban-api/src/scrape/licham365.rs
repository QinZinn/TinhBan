//! Scrape phần diễn giải chi tiết từ **licham365.vn**.
//!
//! Đây là **nguồn phụ**. Nguồn chính là `tinhban_core::trach_nhat` (tự tính,
//! offline, không bao giờ hỏng). Module này chỉ thêm phần văn bản dân gian mà
//! thuật toán không sinh ra được — "ngày này nên/không nên làm gì" cho từng loại
//! việc, sao tốt/sao xấu theo Ngọc Hạp Thông Thư, hướng xuất hành…
//!
//! # Chiến lược chống vỡ khi site đổi HTML
//!
//! Không bám vào selector riêng cho từng mục (kiểu `.sao-tot > ul > li:nth(2)`).
//! Thay vào đó bóc **tổng quát**: trang licham365 chia nội dung thành các khối
//! `div.c-de`, mỗi khối có một tiêu đề `h3`. Ta duyệt mọi khối, lấy `h3` làm
//! tên mục và phần text còn lại làm nội dung.
//!
//! Hệ quả:
//!  - site **thêm/đổi tên/sắp xếp lại** mục → ta vẫn lấy được, chỉ là tên mục đổi;
//!  - chỉ khi site đổi hẳn khung `div.c-de` + `h3` thì mới hỏng, và lúc đó
//!    [`parse_sections`] trả về 0 mục → [`ScrapeError::NoSections`] nêu đích danh
//!    điều cần sửa.
//!
//! **Dấu hiệu cần cập nhật selector**: log xuất hiện `ScrapeError::NoSections`
//! hoặc `TooFewSections`. Hai hằng [`SECTION_BLOCK_SELECTOR`] và
//! [`SECTION_TITLE_SELECTOR`] là hai thứ duy nhất cần sửa.
//!
//! # Lễ độ với server nguồn
//!
//! Đây là app cá nhân, dùng thưa. Chỉ scrape **theo yêu cầu thật** của người
//! dùng cho đúng ngày họ xem, không cào hàng loạt, không pre-cache cả năm. Kết
//! quả cache vĩnh viễn nên mỗi ngày chỉ tải đúng một lần trong suốt vòng đời app.
//! User-Agent khai báo trung thực là bot của dự án kèm link repo — không giả mạo
//! trình duyệt để né phát hiện.

use std::time::Duration;

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// Khối nội dung trên trang licham365. Đổi hằng này nếu site đổi khung HTML.
pub const SECTION_BLOCK_SELECTOR: &str = "div.c-de";

/// Tiêu đề của một khối nội dung.
pub const SECTION_TITLE_SELECTOR: &str = "h3";

/// Số mục tối thiểu coi là "trang còn nguyên vẹn". Trang bình thường có ~11–12
/// mục; dưới ngưỡng này gần như chắc chắn cấu trúc đã đổi hoặc ta bị chặn.
pub const MIN_SECTIONS: usize = 4;

/// User-Agent khai báo trung thực (bot của dự án + link repo), KHÔNG giả trình
/// duyệt.
pub const USER_AGENT: &str = concat!(
    "TinhBan/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/QinZinn/TinhBan; self-hosted personal almanac)"
);

/// Timeout cho một request. Ngắn có chủ đích: nếu nguồn phụ chậm, thà rơi về
/// kết quả tự tính ngay còn hơn bắt người dùng chờ.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Một mục diễn giải bóc từ trang.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    /// Tiêu đề mục, ví dụ `"Theo Ngọc Hạp Thông Thư"`, `"Trực: Bế"`.
    pub title: String,
    /// Nội dung dạng text, mỗi ý một dòng.
    pub body: String,
}

/// Toàn bộ phần diễn giải bóc được cho một ngày.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Licham365Detail {
    /// URL nguồn.
    pub source_url: String,
    /// Các mục diễn giải, giữ nguyên thứ tự xuất hiện trên trang.
    pub sections: Vec<Section>,
    /// Đoạn tóm tắt "… là ngày Hoàng đạo/Hắc đạo" nếu bóc được.
    pub tom_tat: Option<String>,
}

/// Lỗi khi scrape. Mỗi biến thể nêu rõ *cần làm gì* khi gặp.
#[derive(Debug)]
pub enum ScrapeError {
    /// Không gọi được (mạng, DNS, timeout, TLS…). Thường là tạm thời.
    Request(reqwest::Error),
    /// Server trả mã lỗi HTTP.
    Status(u16),
    /// Selector khối nội dung không khớp gì cả → **site đã đổi cấu trúc HTML**,
    /// cần cập nhật [`SECTION_BLOCK_SELECTOR`].
    NoSections,
    /// Bóc được nhưng quá ít mục so với bình thường → cấu trúc đổi một phần,
    /// hoặc bị chặn/trả trang rút gọn.
    TooFewSections(usize),
}

impl std::fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(e) if e.is_timeout() => {
                write!(f, "quá thời gian chờ khi gọi licham365.vn ({e})")
            }
            Self::Request(e) if e.is_connect() => {
                write!(f, "không kết nối được tới licham365.vn ({e})")
            }
            Self::Request(e) => write!(f, "lỗi mạng khi gọi licham365.vn: {e}"),
            Self::Status(c) => write!(f, "licham365.vn trả HTTP {c}"),
            Self::NoSections => write!(
                f,
                "không tìm thấy khối nội dung nào khớp selector {SECTION_BLOCK_SELECTOR:?} \
                 — nhiều khả năng licham365.vn đã đổi cấu trúc HTML, cần cập nhật \
                 SECTION_BLOCK_SELECTOR trong scrape/licham365.rs"
            ),
            Self::TooFewSections(n) => write!(
                f,
                "chỉ bóc được {n} mục (kỳ vọng ít nhất {MIN_SECTIONS}) — cấu trúc HTML \
                 của licham365.vn có thể đã đổi một phần, hoặc trang trả về bản rút gọn"
            ),
        }
    }
}

impl std::error::Error for ScrapeError {}

/// Gốc URL mặc định của nguồn.
pub const DEFAULT_BASE_URL: &str = "https://licham365.vn";

/// Gốc URL đang dùng — đọc từ biến môi trường `LICHAM365_BASE_URL`, mặc định
/// [`DEFAULT_BASE_URL`].
///
/// Có biến này để **kiểm thử đường fallback mà không phải sửa code**: trỏ sang
/// một host không tồn tại rồi gọi endpoint, hệ thống phải rơi êm về kết quả tự
/// tính. Xem mục "Kiểm thử fallback" trong README.
pub fn base_url() -> String {
    std::env::var("LICHAM365_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

/// Dựng URL trang chi tiết của một ngày.
///
/// Pattern: `/lich-am-ngay-{D}-thang-{M}-nam-{YYYY}` — ngày và tháng **không**
/// đệm số 0 (`.../lich-am-ngay-5-thang-3-nam-2024`, không phải `05`/`03`).
pub fn build_url(date: chrono::NaiveDate) -> String {
    use chrono::Datelike;
    format!(
        "{}/lich-am-ngay-{}-thang-{}-nam-{}",
        base_url().trim_end_matches('/'),
        date.day(),
        date.month(),
        date.year()
    )
}

/// Gom text của một phần tử thành chuỗi gọn: bỏ khoảng trắng thừa, mỗi ý một
/// dòng, bỏ dòng rỗng.
fn text_of(el: scraper::ElementRef<'_>) -> String {
    let raw: String = el.text().collect::<Vec<_>>().join("\n");
    raw.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Bóc các mục diễn giải từ HTML thô.
///
/// Tách riêng khỏi phần gọi mạng để test được bằng HTML lưu sẵn, không cần
/// internet — xem `tests/` cuối file.
pub fn parse_sections(html: &str, source_url: &str) -> Result<Licham365Detail, ScrapeError> {
    let doc = Html::parse_document(html);

    let block_sel = Selector::parse(SECTION_BLOCK_SELECTOR).expect("selector hằng hợp lệ");
    let title_sel = Selector::parse(SECTION_TITLE_SELECTOR).expect("selector hằng hợp lệ");

    let mut sections = Vec::new();
    for block in doc.select(&block_sel) {
        let title = block
            .select(&title_sel)
            .next()
            .map(text_of)
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        // Body = toàn bộ text của khối trừ dòng tiêu đề ở đầu.
        let full = text_of(block);
        let body = full
            .strip_prefix(&title)
            .unwrap_or(&full)
            .trim_start()
            .to_string();
        if body.is_empty() {
            continue;
        }
        sections.push(Section { title, body });
    }

    if sections.is_empty() {
        return Err(ScrapeError::NoSections);
    }
    if sections.len() < MIN_SECTIONS {
        return Err(ScrapeError::TooFewSections(sections.len()));
    }

    // Đoạn tóm tắt nằm trong text trang: "… âm lịch, là ngày Hoàng đạo (tốt)."
    let tom_tat = doc
        .root_element()
        .text()
        .collect::<String>()
        .split_once("Chi tiết ngày tốt xấu,")
        .and_then(|(_, rest)| rest.split_once('.').map(|(s, _)| s.trim().to_string()))
        .filter(|s| !s.is_empty() && s.len() < 400)
        .map(|s| format!("Chi tiết ngày tốt xấu, {s}."));

    Ok(Licham365Detail {
        source_url: source_url.to_string(),
        sections,
        tom_tat,
    })
}

/// Tải và bóc trang chi tiết của một ngày.
///
/// Hàm này **chỉ** gọi mạng + parse; nó không biết gì về cache. Chính sách cache
/// nằm ở [`super::super::ngay_tot_xau`] để logic mạng và logic lưu trữ tách bạch.
pub async fn fetch_detail(
    client: &reqwest::Client,
    date: chrono::NaiveDate,
) -> Result<Licham365Detail, ScrapeError> {
    let url = build_url(date);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(ScrapeError::Request)?;

    let status = resp.status();
    if !status.is_success() {
        return Err(ScrapeError::Status(status.as_u16()));
    }

    let body = resp.text().await.map_err(ScrapeError::Request)?;
    parse_sections(&body, &url)
}

/// Client dùng chung: bật timeout và User-Agent trung thực.
pub fn build_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(REQUEST_TIMEOUT)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// HTML mẫu tối giản, mô phỏng đúng khung `div.c-de` + `h3` của licham365.
    const HTML_OK: &str = r#"
        <html><body>
          <table class="c-da"><tr><td>Tháng 3 Năm 2024</td></tr></table>
          <div class="c-de"><div class="c-he-titles"><h3>Giờ hoàng đạo &amp; Giờ hắc đạo</h3></div>
            <table><tr><td>Giờ Hoàng Đạo</td><td>Tý (23:00-0:59), Sửu (1:00-2:59)</td></tr></table></div>
          <div class="c-de"><div class="c-he-titles"><h3>Theo Ngọc Hạp Thông Thư</h3></div>
            <p>Sao tốt: Thiên Phúc</p><p>Sao xấu: Ly sào</p></div>
          <div class="c-de"><div class="c-he-titles"><h3>Trực: Bế</h3></div>
            <p>Nên làm: Xây đắp tường</p><p>Không nên làm: Nhập học</p></div>
          <div class="c-de"><div class="c-he-titles"><h3>Xuất hành</h3></div>
            <p>Hỷ thần: Đông Nam</p></div>
          <p>Chi tiết ngày tốt xấu, 15 tháng 3 năm 2024 , nhằm ngày 6-2-2024 âm lịch,
             là ngày Hoàng đạo (tốt). Còn lại không quan trọng.</p>
        </body></html>
    "#;

    #[test]
    fn bocduoc_cac_muc_va_giu_thu_tu() {
        let d = parse_sections(HTML_OK, "https://example/x").expect("phải bóc được");
        let titles: Vec<&str> = d.sections.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(
            titles,
            vec![
                "Giờ hoàng đạo & Giờ hắc đạo",
                "Theo Ngọc Hạp Thông Thư",
                "Trực: Bế",
                "Xuất hành",
            ]
        );
        assert!(d.sections[2].body.contains("Nên làm"));
        // Tiêu đề không được lặp lại trong body.
        assert!(!d.sections[2].body.starts_with("Trực: Bế"));
    }

    #[test]
    fn boc_duoc_doan_tom_tat() {
        let d = parse_sections(HTML_OK, "https://example/x").unwrap();
        let t = d.tom_tat.expect("phải có tóm tắt");
        assert!(t.contains("Hoàng đạo (tốt)"), "tóm tắt sai: {t}");
        assert!(!t.contains("Còn lại không quan trọng"), "cắt sai câu: {t}");
    }

    /// Site đổi khung HTML → phải báo `NoSections` với thông điệp chỉ đúng chỗ
    /// cần sửa, KHÔNG panic và KHÔNG trả về rỗng im lặng.
    #[test]
    fn doi_cau_truc_html_bao_loi_ro_rang() {
        let html = "<html><body><div class='doi-roi'><h3>Gì đó</h3><p>abc</p></div></body></html>";
        let e = parse_sections(html, "u").expect_err("phải lỗi");
        assert!(matches!(e, ScrapeError::NoSections));
        let msg = e.to_string();
        assert!(msg.contains("div.c-de"), "thông điệp phải nêu selector: {msg}");
        assert!(
            msg.contains("SECTION_BLOCK_SELECTOR"),
            "thông điệp phải chỉ ra hằng cần sửa: {msg}"
        );
    }

    /// Trang rút gọn / bị chặn → `TooFewSections`, không nhận nhầm là thành công.
    #[test]
    fn trang_qua_it_muc_bi_tu_choi() {
        let html = "<html><body>\
            <div class='c-de'><h3>A</h3><p>1</p></div>\
            <div class='c-de'><h3>B</h3><p>2</p></div></body></html>";
        let e = parse_sections(html, "u").expect_err("phải lỗi");
        assert!(matches!(e, ScrapeError::TooFewSections(2)), "{e:?}");
    }

    /// Khối không có `h3` hoặc không có nội dung thì bỏ qua, không sinh mục rỗng.
    #[test]
    fn bo_qua_khoi_rong() {
        let html = format!(
            "<html><body>{}\
             <div class='c-de'><p>không có tiêu đề</p></div>\
             <div class='c-de'><h3>Rỗng</h3></div></body></html>",
            HTML_OK
        );
        let d = parse_sections(&html, "u").unwrap();
        assert!(d.sections.iter().all(|s| !s.title.is_empty() && !s.body.is_empty()));
        assert!(!d.sections.iter().any(|s| s.title == "Rỗng"));
    }

    #[test]
    fn url_khong_dem_so_0() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 3, 5).unwrap();
        // Không đặt LICHAM365_BASE_URL trong test này → dùng gốc mặc định.
        assert_eq!(
            build_url(d),
            format!("{DEFAULT_BASE_URL}/lich-am-ngay-5-thang-3-nam-2024")
        );
    }

    #[test]
    fn user_agent_khong_gia_mao_trinh_duyet() {
        assert!(USER_AGENT.starts_with("TinhBan/"));
        for xau in ["Mozilla", "Chrome", "Safari", "AppleWebKit"] {
            assert!(!USER_AGENT.contains(xau), "UA không được giả {xau}");
        }
    }
}
