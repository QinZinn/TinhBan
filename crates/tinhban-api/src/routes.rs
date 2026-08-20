//! Toàn bộ route: trang HTML (SSR) và API JSON.
//!
//! Trang HTML dùng form POST thường + redirect 303 (Post/Redirect/Get) thay vì
//! fetch/JS — không có JavaScript nào trong app này.

use dioxus::prelude::*;
use dioxus::server::axum::{
    extract::{Form, Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use serde::Deserialize;

use crate::ho_so::{self, HoSoTomTat, LoiHoSo, NhapHoSo};
use crate::ngay_tot_xau;
use crate::ui::{self, trang};
use crate::{DB, HTTP};

// ===========================================================================
// Tiện ích
// ===========================================================================

/// Trang lỗi đơn giản, dùng cho 404 / 400 ở tầng HTML.
fn trang_loi(code: StatusCode, tieu_de: &str, msg: &str) -> Response {
    let html = trang(
        tieu_de,
        rsx! {
            h1 { "{tieu_de}" }
            div { class: "flash err", "{msg}" }
            p { a { href: "/", "← Về trang chủ" } }
        },
    );
    (code, Html(html)).into_response()
}

fn loi_db(e: sqlx::Error) -> Response {
    tracing::error!(error = %e, "lỗi truy vấn DB");
    trang_loi(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Lỗi hệ thống",
        "Không truy vấn được cơ sở dữ liệu. Xem log server để biết chi tiết.",
    )
}

/// Render markdown → HTML cho trang chi tiết từ điển.
///
/// Nội dung markdown là **của chính dự án** (nhúng trong binary hoặc lấy từ thư
/// mục do người vận hành chỉ định), không phải input từ người lạ — nên bật HTML
/// thô trong markdown là chấp nhận được ở đây.
fn render_markdown(md: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(md, opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

// ===========================================================================
// Trang chủ
// ===========================================================================

pub async fn trang_chu() -> Response {
    let so_ho_so = tinhban_db::danh_sach_ho_so(&DB).await.map(|v| v.len()).unwrap_or(0);
    let so_muc = tinhban_db::tat_ca_tu_dien(&DB).await.map(|v| v.len()).unwrap_or(0);
    Html(trang("Trang chủ", ui::pages::trang_chu(so_ho_so, so_muc))).into_response()
}

// ===========================================================================
// Lập lá số / hồ sơ
// ===========================================================================

pub async fn form_la_so() -> Response {
    Html(trang("Lập lá số", ui::pages::form_la_so(None))).into_response()
}

pub async fn tao_la_so(Form(n): Form<NhapHoSo>) -> Response {
    match ho_so::tao(&DB, &n).await {
        Ok(id) => Redirect::to(&format!("/ho-so/{id}")).into_response(),
        Err(LoiHoSo::Db(e)) => loi_db(e),
        Err(e) => {
            // Lỗi do dữ liệu nhập → hiện lại form kèm thông báo, không 500.
            let html = trang("Lập lá số", ui::pages::form_la_so(Some(&e.to_string())));
            (StatusCode::BAD_REQUEST, Html(html)).into_response()
        }
    }
}

pub async fn danh_sach_ho_so() -> Response {
    match tinhban_db::danh_sach_ho_so(&DB).await {
        Ok(ds) => Html(trang("Hồ sơ", ui::pages::danh_sach_ho_so(&ds))).into_response(),
        Err(e) => loi_db(e),
    }
}

pub async fn chi_tiet_ho_so(Path(id): Path<i64>) -> Response {
    match tinhban_db::lay_ho_so(&DB, id).await {
        Ok(Some(h)) => {
            let (tuvi, bat_tu) = ho_so::giai_ma(&h);
            let canh_bao = ho_so::canh_bao_phien_ban(&h);
            let html = trang(
                &h.ten,
                ui::pages::chi_tiet_ho_so(&h, tuvi.as_ref(), bat_tu.as_ref(), canh_bao),
            );
            Html(html).into_response()
        }
        Ok(None) => trang_loi(
            StatusCode::NOT_FOUND,
            "Không tìm thấy hồ sơ",
            &format!("Không có hồ sơ nào mang số {id}."),
        ),
        Err(e) => loi_db(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct SuaHoSoForm {
    pub ten: Option<String>,
    pub ghi_chu: Option<String>,
}

pub async fn sua_ho_so(Path(id): Path<i64>, Form(f): Form<SuaHoSoForm>) -> Response {
    let ten = f.ten.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let ghi_chu = f.ghi_chu.as_deref().map(str::trim);
    match tinhban_db::sua_ho_so(&DB, id, ten, ghi_chu).await {
        Ok(true) => Redirect::to(&format!("/ho-so/{id}")).into_response(),
        Ok(false) => trang_loi(StatusCode::NOT_FOUND, "Không tìm thấy hồ sơ", "Hồ sơ đã bị xoá?"),
        Err(e) => loi_db(e),
    }
}

pub async fn xoa_ho_so(Path(id): Path<i64>) -> Response {
    match tinhban_db::xoa_ho_so(&DB, id).await {
        Ok(_) => Redirect::to("/ho-so").into_response(),
        Err(e) => loi_db(e),
    }
}

// ===========================================================================
// Từ điển
// ===========================================================================

#[derive(Debug, Deserialize, Default)]
pub struct TimKiem {
    #[serde(default)]
    pub q: String,
}

pub async fn trang_tu_dien(Query(t): Query<TimKiem>) -> Response {
    let q = t.q.trim();
    let (muc, la_tim) = if q.is_empty() {
        (tinhban_db::tat_ca_tu_dien(&DB).await, false)
    } else {
        (tinhban_db::tim_tu_dien(&DB, q, 50).await, true)
    };
    match muc {
        Ok(m) => Html(trang("Từ điển", ui::pages::trang_tu_dien(q, &m, la_tim))).into_response(),
        Err(e) => loi_db(e),
    }
}

pub async fn chi_tiet_tu_dien(Path(slug): Path<String>) -> Response {
    match tinhban_db::lay_tu_dien(&DB, &slug).await {
        Ok(Some(m)) => {
            let body = render_markdown(&m.body);
            Html(trang(&m.title, ui::pages::chi_tiet_tu_dien(&m, body))).into_response()
        }
        Ok(None) => trang_loi(
            StatusCode::NOT_FOUND,
            "Không tìm thấy mục từ điển",
            &format!("Không có mục nào mang mã {slug:?}."),
        ),
        Err(e) => loi_db(e),
    }
}

// ===========================================================================
// Ngày tốt/xấu (trang HTML — endpoint JSON đã có từ giai đoạn 5)
// ===========================================================================

#[derive(Debug, Deserialize, Default)]
pub struct ChonNgay {
    pub date: Option<String>,
}

pub async fn trang_ngay_tot_xau(Query(c): Query<ChonNgay>) -> Response {
    // Không truyền `date` → mặc định hôm nay theo giờ VN.
    let hom_nay = (chrono::Utc::now() + chrono::Duration::hours(7))
        .date_naive()
        .to_string();
    let ds = c.date.as_deref().map(str::trim).filter(|s| !s.is_empty()).unwrap_or(&hom_nay);

    let Ok(date) = chrono::NaiveDate::parse_from_str(ds, "%Y-%m-%d") else {
        return trang_loi(
            StatusCode::BAD_REQUEST,
            "Ngày không hợp lệ",
            &format!("{ds:?} không đúng định dạng YYYY-MM-DD."),
        );
    };

    match ngay_tot_xau::xem_ngay(&DB, &HTTP, date).await {
        Ok(r) => Html(trang(
            &format!("Ngày {ds}"),
            ui::ngay::trang_ngay(ds, Some(&r)),
        ))
        .into_response(),
        Err(e) => trang_loi(StatusCode::BAD_REQUEST, "Không xem được ngày này", &e.to_string()),
    }
}

// ===========================================================================
// API JSON
// ===========================================================================

pub async fn api_tu_dien(Query(t): Query<TimKiem>) -> Response {
    let q = t.q.trim();
    let r = if q.is_empty() {
        tinhban_db::tat_ca_tu_dien(&DB).await
    } else {
        tinhban_db::tim_tu_dien(&DB, q, 50).await
    };
    match r {
        Ok(m) => Json(serde_json::json!({
            "q": q,
            "count": m.len(),
            "items": m.iter().map(muc_json).collect::<Vec<_>>(),
        }))
        .into_response(),
        Err(e) => api_loi_db(e),
    }
}

pub async fn api_tu_dien_chi_tiet(Path(slug): Path<String>) -> Response {
    match tinhban_db::lay_tu_dien(&DB, &slug).await {
        Ok(Some(m)) => {
            let mut v = muc_json(&m);
            v["body"] = serde_json::Value::String(m.body.clone());
            Json(v).into_response()
        }
        Ok(None) => api_404(&format!("không có mục từ điển {slug:?}")),
        Err(e) => api_loi_db(e),
    }
}

fn muc_json(m: &tinhban_db::MucTuDien) -> serde_json::Value {
    serde_json::json!({
        "slug": m.slug, "title": m.title, "kind": m.kind, "nhom": m.nhom,
        "nguhanh": m.nguhanh, "amduong": m.amduong, "aliases": m.aliases,
    })
}

fn api_404(msg: &str) -> Response {
    (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": msg }))).into_response()
}

fn api_loi_db(e: sqlx::Error) -> Response {
    tracing::error!(error = %e, "lỗi DB (API)");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "lỗi cơ sở dữ liệu" })),
    )
        .into_response()
}

pub async fn api_tao_ho_so(Json(n): Json<NhapHoSo>) -> Response {
    match ho_so::tao(&DB, &n).await {
        Ok(id) => match tinhban_db::lay_ho_so(&DB, id).await {
            Ok(Some(h)) => (StatusCode::CREATED, Json(HoSoTomTat::from(&h))).into_response(),
            Ok(None) => api_404("hồ sơ vừa tạo không đọc lại được"),
            Err(e) => api_loi_db(e),
        },
        Err(LoiHoSo::Db(e)) => api_loi_db(e),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn api_danh_sach_ho_so() -> Response {
    match tinhban_db::danh_sach_ho_so(&DB).await {
        Ok(ds) => Json(ds.iter().map(HoSoTomTat::from).collect::<Vec<_>>()).into_response(),
        Err(e) => api_loi_db(e),
    }
}

pub async fn api_chi_tiet_ho_so(Path(id): Path<i64>) -> Response {
    match tinhban_db::lay_ho_so(&DB, id).await {
        Ok(Some(h)) => {
            // Trả lá số dưới dạng JSON lồng (đã lưu sẵn) thay vì chuỗi.
            let tuvi: serde_json::Value =
                serde_json::from_str(&h.tuvi_json).unwrap_or(serde_json::Value::Null);
            let bat_tu: serde_json::Value =
                serde_json::from_str(&h.bat_tu_json).unwrap_or(serde_json::Value::Null);
            let mut v = serde_json::to_value(HoSoTomTat::from(&h)).unwrap_or_default();
            v["tuvi"] = tuvi;
            v["bat_tu"] = bat_tu;
            Json(v).into_response()
        }
        Ok(None) => api_404(&format!("không có hồ sơ số {id}")),
        Err(e) => api_loi_db(e),
    }
}

pub async fn api_sua_ho_so(Path(id): Path<i64>, Json(f): Json<SuaHoSoForm>) -> Response {
    let ten = f.ten.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let ghi_chu = f.ghi_chu.as_deref().map(str::trim);
    match tinhban_db::sua_ho_so(&DB, id, ten, ghi_chu).await {
        Ok(true) => match tinhban_db::lay_ho_so(&DB, id).await {
            Ok(Some(h)) => Json(HoSoTomTat::from(&h)).into_response(),
            Ok(None) => api_404(&format!("không có hồ sơ số {id}")),
            Err(e) => api_loi_db(e),
        },
        Ok(false) => api_404(&format!("không có hồ sơ số {id}")),
        Err(e) => api_loi_db(e),
    }
}

pub async fn api_xoa_ho_so(Path(id): Path<i64>) -> Response {
    match tinhban_db::xoa_ho_so(&DB, id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => api_404(&format!("không có hồ sơ số {id}")),
        Err(e) => api_loi_db(e),
    }
}
