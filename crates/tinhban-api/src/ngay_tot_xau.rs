//! Endpoint "ngày tốt/xấu": ghép **kết quả tự tính** (nguồn chính, offline) với
//! **diễn giải scrape** (nguồn phụ, có cache + fallback).
//!
//! # Nguyên tắc: nguồn phụ không bao giờ được làm hỏng nguồn chính
//!
//! Kết quả tự tính từ `tinhban_core::trach_nhat` luôn được trả về. Mọi thứ liên
//! quan tới licham365.vn — mạng, HTML đổi cấu trúc, site sập — chỉ ảnh hưởng
//! trường `dien_giai` (thành `null`) kèm `ghi_chu` giải thích, chứ không bao giờ
//! biến thành lỗi 5xx hay JSON rỗng khó hiểu.
//!
//! # Chính sách cache
//!
//! | Trạng thái cache | Hành vi |
//! |------------------|---------|
//! | có, `ok`         | Dùng luôn, **không gọi mạng** (nội dung một ngày là tĩnh → cache vĩnh viễn) |
//! | có, `error`, còn "mới" (< [`ERROR_RETRY_AFTER`]) | Không thử lại, trả fallback kèm lỗi đã lưu |
//! | có, `error`, đã cũ | Thử scrape lại |
//! | chưa có          | Scrape, rồi lưu kết quả (kể cả lỗi) |
//!
//! Bản ghi lỗi có TTL còn bản ghi tốt thì không: một lần site sập không được
//! đóng băng ngày đó vĩnh viễn, nhưng cũng không nên thử lại mỗi lần bấm F5.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::Duration;
use tinhban_core::{
    can_chi_display, danh_gia_ngay, DayAssessment, HourRange, KiengKy, LunarError,
};

use crate::scrape::licham365::{self, Licham365Detail};

/// Sau bao lâu thì thử scrape lại một ngày từng lỗi.
pub const ERROR_RETRY_AFTER: Duration = Duration::from_secs(60 * 60);

// ===========================================================================
// DTO — hình dạng JSON trả ra
// ===========================================================================

/// Khung giờ, dạng thân thiện cho client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GioDto {
    pub chi: String,
    pub than: String,
    pub tot: bool,
    /// Ví dụ `"23:00–00:59"`.
    pub khung: String,
}

impl From<&HourRange> for GioDto {
    fn from(g: &HourRange) -> Self {
        Self {
            chi: g.branch.name_vn().to_string(),
            than: g.than.name_vn().to_string(),
            tot: g.is_hoang_dao(),
            khung: format!("{:02}:00–{:02}:59", g.start_hour, g.end_hour),
        }
    }
}

/// Một điều kiêng kỵ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiengKyDto {
    pub ten: String,
    pub y_nghia: String,
}

/// Phần tự tính — luôn có mặt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuTinhDto {
    pub ngay_duong: String,
    pub ngay_am: String,
    pub thang_am_nhuan: bool,
    pub ngay_can_chi: String,
    pub thang_can_chi: String,
    pub nam_can_chi: String,
    pub tiet_khi: String,
    pub ngay_hoang_dao: bool,
    pub ket_luan_ngay: String,
    pub than_truc_ngay: String,
    pub y_nghia_than: String,
    pub truc: String,
    pub truc_nen_lam: String,
    pub truc_khong_nen_lam: String,
    pub gio_hoang_dao: Vec<GioDto>,
    pub gio_hac_dao: Vec<GioDto>,
    pub kieng_ky: Vec<KiengKyDto>,
}

impl From<&DayAssessment> for TuTinhDto {
    fn from(a: &DayAssessment) -> Self {
        Self {
            ngay_duong: a.solar_date.to_string(),
            ngay_am: format!(
                "{}/{}/{}{}",
                a.lunar_date.day,
                a.lunar_date.month,
                a.lunar_date.year,
                if a.lunar_date.is_leap_month {
                    " (nhuận)"
                } else {
                    ""
                }
            ),
            thang_am_nhuan: a.lunar_date.is_leap_month,
            ngay_can_chi: can_chi_display(a.day_can_chi),
            thang_can_chi: can_chi_display(a.month_can_chi),
            nam_can_chi: can_chi_display(a.year_can_chi),
            tiet_khi: a.tiet_khi.to_string(),
            ngay_hoang_dao: a.hoang_dao_hac_dao.is_hoang_dao,
            ket_luan_ngay: a.hoang_dao_hac_dao.nhan_vn().to_string(),
            than_truc_ngay: a.hoang_dao_hac_dao.than.name_vn().to_string(),
            y_nghia_than: a.hoang_dao_hac_dao.than.y_nghia_vn().to_string(),
            truc: a.truc.name_vn().to_string(),
            truc_nen_lam: a.truc.nen_lam_vn().to_string(),
            truc_khong_nen_lam: a.truc.khong_nen_lam_vn().to_string(),
            gio_hoang_dao: a.gio_hoang_dao.iter().map(GioDto::from).collect(),
            gio_hac_dao: a.gio_hac_dao.iter().map(GioDto::from).collect(),
            kieng_ky: a
                .kieng_ky
                .iter()
                .map(|k: &KiengKy| KiengKyDto {
                    ten: k.name_vn().to_string(),
                    y_nghia: k.y_nghia_vn().to_string(),
                })
                .collect(),
        }
    }
}

/// Nguồn của phần diễn giải, để client biết dữ liệu đến từ đâu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NguonDienGiai {
    /// Đọc từ cache SQLite, không gọi mạng.
    Cache,
    /// Vừa scrape mới trong request này.
    Scrape,
}

/// Kết quả trả về của `GET /api/ngay-tot-xau`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NgayTotXauResponse {
    /// Phần tự tính — **luôn có**.
    pub tu_tinh: TuTinhDto,
    /// Phần diễn giải chi tiết từ licham365.vn — `null` khi không lấy được.
    pub dien_giai: Option<Licham365Detail>,
    /// Diễn giải đến từ đâu (`null` khi không có diễn giải).
    pub nguon_dien_giai: Option<NguonDienGiai>,
    /// Giải thích cho người dùng khi `dien_giai` là `null`.
    pub ghi_chu: Option<String>,
}

// ===========================================================================
// Logic chính
// ===========================================================================

/// Bản ghi lỗi trong cache đã đủ cũ để thử lại chưa.
fn nen_thu_lai(fetched_at: &str) -> bool {
    let Ok(t) = chrono::DateTime::parse_from_rfc3339(fetched_at)
        .or_else(|_| chrono::DateTime::parse_from_str(fetched_at, "%Y-%m-%dT%H:%M:%SZ"))
    else {
        // Không đọc được mốc thời gian → coi như cũ, cho thử lại. Thà gọi thừa
        // một lần còn hơn kẹt vĩnh viễn vì một chuỗi hỏng trong DB.
        return true;
    };
    let tuoi = chrono::Utc::now().signed_duration_since(t.with_timezone(&chrono::Utc));
    tuoi.num_seconds() >= ERROR_RETRY_AFTER.as_secs() as i64
}

/// Lấy phần diễn giải: ưu tiên cache, chỉ gọi mạng khi cần.
///
/// Không bao giờ trả `Err` — mọi hỏng hóc quy về `(None, Some(ghi_chú))`.
async fn lay_dien_giai(
    pool: &SqlitePool,
    client: &reqwest::Client,
    date: NaiveDate,
) -> (Option<Licham365Detail>, Option<NguonDienGiai>, Option<String>) {
    let key = date.to_string();

    // --- 1. Cache
    match tinhban_db::get_licham365_cache(pool, &key).await {
        Ok(Some(c)) if c.ok => {
            if let Some(json) = c.payload.as_deref() {
                match serde_json::from_str::<Licham365Detail>(json) {
                    Ok(d) => {
                        tracing::debug!(date = %key, "diễn giải: dùng cache");
                        return (Some(d), Some(NguonDienGiai::Cache), None);
                    }
                    Err(e) => {
                        // Payload cũ không còn khớp struct (ví dụ đổi schema) →
                        // bỏ qua cache và scrape lại, không làm hỏng request.
                        tracing::warn!(date = %key, error = %e, "cache hỏng, sẽ scrape lại");
                    }
                }
            }
        }
        Ok(Some(c)) if !nen_thu_lai(&c.fetched_at) => {
            let ly_do = c.error.unwrap_or_else(|| "không rõ".to_string());
            tracing::debug!(date = %key, "diễn giải: lỗi còn mới trong cache, không thử lại");
            return (
                None,
                None,
                Some(format!(
                    "Không lấy được diễn giải chi tiết từ licham365.vn ({ly_do}). \
                     Đang hiển thị kết quả tính toán cơ bản. Sẽ tự thử lại sau ít phút."
                )),
            );
        }
        Ok(_) => {}
        Err(e) => {
            // Cache hỏng không được chặn tính năng — cứ scrape.
            tracing::warn!(date = %key, error = %e, "đọc cache lỗi, bỏ qua cache");
        }
    }

    // --- 2. Scrape
    match licham365::fetch_detail(client, date).await {
        Ok(detail) => {
            match serde_json::to_string(&detail) {
                Ok(json) => {
                    if let Err(e) =
                        tinhban_db::put_licham365_ok(pool, &key, &json, &detail.source_url).await
                    {
                        tracing::warn!(date = %key, error = %e, "ghi cache thất bại");
                    }
                }
                Err(e) => tracing::warn!(date = %key, error = %e, "serialize cache thất bại"),
            }
            tracing::info!(date = %key, so_muc = detail.sections.len(), "scrape thành công");
            (Some(detail), Some(NguonDienGiai::Scrape), None)
        }
        Err(e) => {
            let ly_do = e.to_string();
            tracing::warn!(date = %key, error = %ly_do, "scrape licham365 thất bại — rơi về kết quả tự tính");
            let url = licham365::build_url(date);
            if let Err(e2) = tinhban_db::put_licham365_error(pool, &key, &ly_do, &url).await {
                tracing::warn!(date = %key, error = %e2, "ghi cache lỗi thất bại");
            }
            (
                None,
                None,
                Some(format!(
                    "Không lấy được diễn giải chi tiết từ licham365.vn ({ly_do}). \
                     Đang hiển thị kết quả tính toán cơ bản."
                )),
            )
        }
    }
}

/// Dựng phản hồi đầy đủ cho một ngày.
///
/// Chỉ trả `Err` khi **phần tự tính** không làm được (ngày ngoài 1900–2100) —
/// tức lỗi của chính người gọi, không phải lỗi nguồn phụ.
pub async fn xem_ngay(
    pool: &SqlitePool,
    client: &reqwest::Client,
    date: NaiveDate,
) -> Result<NgayTotXauResponse, LunarError> {
    let assessment = danh_gia_ngay(date)?;
    let (dien_giai, nguon, ghi_chu) = lay_dien_giai(pool, client, date).await;
    Ok(NgayTotXauResponse {
        tu_tinh: TuTinhDto::from(&assessment),
        dien_giai,
        nguon_dien_giai: nguon,
        ghi_chu,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ban_ghi_loi_moi_thi_khong_thu_lai() {
        let vua_xong = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        assert!(!nen_thu_lai(&vua_xong));
    }

    #[test]
    fn ban_ghi_loi_cu_thi_thu_lai() {
        let lau_roi = (chrono::Utc::now() - chrono::Duration::hours(3))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        assert!(nen_thu_lai(&lau_roi));
    }

    #[test]
    fn moc_thoi_gian_hong_thi_cu_thu_lai() {
        assert!(nen_thu_lai("không phải ngày giờ"));
        assert!(nen_thu_lai(""));
    }

    /// DTO phải giữ đủ thông tin cốt lõi và không rỗng.
    #[test]
    fn dto_tu_tinh_day_du() {
        let a = danh_gia_ngay(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()).unwrap();
        let dto = TuTinhDto::from(&a);
        assert_eq!(dto.ngay_duong, "2024-03-15");
        assert_eq!(dto.ngay_am, "6/2/2024");
        assert_eq!(dto.ngay_can_chi, "Mậu Dần");
        assert_eq!(dto.truc, "Bế");
        assert_eq!(dto.tiet_khi, "Kinh Trập");
        assert!(dto.ngay_hoang_dao);
        assert_eq!(dto.gio_hoang_dao.len(), 6);
        assert_eq!(dto.gio_hac_dao.len(), 6);
        assert_eq!(dto.gio_hoang_dao[0].khung, "23:00–00:59");
        assert!(dto.gio_hoang_dao.iter().all(|g| g.tot));
        assert!(dto.gio_hac_dao.iter().all(|g| !g.tot));
    }

    /// JSON phải serialize được và giữ nguyên các khoá quan trọng.
    #[test]
    fn response_serialize_duoc() {
        let a = danh_gia_ngay(NaiveDate::from_ymd_opt(2026, 8, 19).unwrap()).unwrap();
        let r = NgayTotXauResponse {
            tu_tinh: TuTinhDto::from(&a),
            dien_giai: None,
            nguon_dien_giai: None,
            ghi_chu: Some("thử".into()),
        };
        let j = serde_json::to_string(&r).unwrap();
        for k in ["tu_tinh", "dien_giai", "ghi_chu", "gio_hoang_dao", "truc"] {
            assert!(j.contains(k), "thiếu khoá {k} trong JSON");
        }
    }
}
