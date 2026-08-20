//! Hồ sơ người đã xem: lập lá số từ thông tin nhập vào rồi lưu lại.
//!
//! # Vì sao lưu lá số dưới dạng JSON thay vì tính lại mỗi lần
//!
//! Hai lý do:
//!  1. Hiển thị lại hồ sơ cũ không cần chạy lại engine (nhanh, và không phụ
//!     thuộc việc engine có đổi hay không).
//!  2. Giữ đúng "lá số đã từng thấy". Nếu engine về sau được sửa (dự án này đã
//!     có tiền lệ: Bug #7 làm sai trụ Năm/Tháng Bát Tự cho ngày sinh rơi đúng mốc
//!     tiết khí), hồ sơ cũ vẫn nguyên trạng và ta biết nó được tính bằng phiên
//!     bản nào nhờ cột `engine_ver`.
//!
//! Đổi lại, hồ sơ cũ **không tự hưởng** bản sửa lỗi. [`canh_bao_phien_ban`] lo
//! việc báo cho người dùng biết điều đó.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tinhban_core::{
    lap_bat_tu, lap_la_so, BatTuChart, BirthMoment, Gender, TuViChart,
};
use tinhban_db::{HoSo, HoSoMoi};

/// Phiên bản engine hiện tại — ghi vào mỗi hồ sơ mới.
pub fn engine_ver() -> &'static str {
    tinhban_core::version()
}

/// Lỗi khi tạo hồ sơ.
#[derive(Debug)]
pub enum LoiHoSo {
    /// Dữ liệu người dùng nhập không hợp lệ.
    DuLieu(String),
    /// Engine không lập được lá số (thường do ngoài phạm vi 1900–2100).
    Engine(String),
    Db(sqlx::Error),
}

impl std::fmt::Display for LoiHoSo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuLieu(m) => write!(f, "{m}"),
            Self::Engine(m) => write!(f, "không lập được lá số: {m}"),
            Self::Db(e) => write!(f, "lỗi cơ sở dữ liệu: {e}"),
        }
    }
}

impl From<sqlx::Error> for LoiHoSo {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}

/// Dữ liệu nhập vào để lập lá số (dùng chung cho form HTML và API JSON).
#[derive(Debug, Clone, Deserialize)]
pub struct NhapHoSo {
    pub ten: String,
    /// 'YYYY-MM-DD'.
    pub ngay_sinh: String,
    pub gio: u8,
    #[serde(default)]
    pub phut: u8,
    /// `nam` | `nu`.
    pub gioi_tinh: String,
    #[serde(default)]
    pub ghi_chu: String,
}

/// Kiểm tra và chuẩn hoá dữ liệu nhập.
fn kiem_tra(n: &NhapHoSo) -> Result<(NaiveDate, BirthMoment, Gender), LoiHoSo> {
    let ten = n.ten.trim();
    if ten.is_empty() {
        return Err(LoiHoSo::DuLieu("Họ tên không được để trống.".into()));
    }
    let date = NaiveDate::parse_from_str(n.ngay_sinh.trim(), "%Y-%m-%d").map_err(|_| {
        LoiHoSo::DuLieu(format!(
            "Ngày sinh {:?} không hợp lệ — cần định dạng YYYY-MM-DD.",
            n.ngay_sinh
        ))
    })?;
    if n.gio > 23 {
        return Err(LoiHoSo::DuLieu("Giờ sinh phải trong khoảng 0–23.".into()));
    }
    if n.phut > 59 {
        return Err(LoiHoSo::DuLieu("Phút sinh phải trong khoảng 0–59.".into()));
    }
    let gender = match n.gioi_tinh.trim() {
        "nam" => Gender::Nam,
        "nu" => Gender::Nu,
        other => {
            return Err(LoiHoSo::DuLieu(format!(
                "Giới tính {other:?} không hợp lệ — chỉ nhận 'nam' hoặc 'nu'."
            )))
        }
    };
    Ok((
        date,
        BirthMoment {
            solar_date: date,
            hour: n.gio,
            minute: n.phut,
        },
        gender,
    ))
}

/// Lập cả hai lá số cho một thời điểm sinh.
pub fn lap_ca_hai(
    birth: BirthMoment,
    gender: Gender,
) -> Result<(TuViChart, BatTuChart), LoiHoSo> {
    let tuvi = lap_la_so(birth, gender).map_err(|e| LoiHoSo::Engine(e.to_string()))?;
    let bat_tu = lap_bat_tu(birth, gender).map_err(|e| LoiHoSo::Engine(e.to_string()))?;
    Ok((tuvi, bat_tu))
}

/// Lập lá số rồi lưu thành hồ sơ mới. Trả về `id`.
pub async fn tao(pool: &SqlitePool, n: &NhapHoSo) -> Result<i64, LoiHoSo> {
    let (date, birth, gender) = kiem_tra(n)?;
    let (tuvi, bat_tu) = lap_ca_hai(birth, gender)?;

    let tuvi_json = serde_json::to_string(&tuvi)
        .map_err(|e| LoiHoSo::Engine(format!("serialize Tử Vi: {e}")))?;
    let bat_tu_json = serde_json::to_string(&bat_tu)
        .map_err(|e| LoiHoSo::Engine(format!("serialize Bát Tự: {e}")))?;

    let id = tinhban_db::them_ho_so(
        pool,
        HoSoMoi {
            ten: n.ten.trim(),
            solar_date: &date.to_string(),
            hour: n.gio,
            minute: n.phut,
            gender: if gender == Gender::Nam { "nam" } else { "nu" },
            ghi_chu: n.ghi_chu.trim(),
            tuvi_json: &tuvi_json,
            bat_tu_json: &bat_tu_json,
            engine_ver: engine_ver(),
        },
    )
    .await?;
    tracing::info!(id, ten = n.ten.trim(), "đã lưu hồ sơ mới");
    Ok(id)
}

/// Giải mã hai lá số đã lưu. Trả `None` cho phần nào không đọc được, thay vì
/// làm hỏng cả trang — hồ sơ vẫn xem được các phần còn lại.
pub fn giai_ma(h: &HoSo) -> (Option<TuViChart>, Option<BatTuChart>) {
    let tuvi = serde_json::from_str::<TuViChart>(&h.tuvi_json)
        .map_err(|e| tracing::warn!(id = h.id, error = %e, "không đọc được tuvi_json"))
        .ok();
    let bat_tu = serde_json::from_str::<BatTuChart>(&h.bat_tu_json)
        .map_err(|e| tracing::warn!(id = h.id, error = %e, "không đọc được bat_tu_json"))
        .ok();
    (tuvi, bat_tu)
}

/// Cảnh báo nếu hồ sơ được tính bằng phiên bản engine cũ hơn bản đang chạy.
///
/// Không tự động tính lại: lá số cũ là **dữ liệu người dùng đã thấy**, im lặng
/// thay đổi nó thì tệ hơn là báo cho họ biết.
pub fn canh_bao_phien_ban(h: &HoSo) -> Option<String> {
    let hien_tai = engine_ver();
    if h.engine_ver == hien_tai {
        return None;
    }
    Some(format!(
        "Lá số này được tính bằng engine phiên bản {} , bản đang chạy là {}. \
         Nội dung hiển thị giữ nguyên như lúc lưu. Nếu muốn áp dụng các bản sửa \
         sau đó, hãy lập lại lá số mới với cùng ngày giờ sinh.",
        h.engine_ver, hien_tai
    ))
}

/// Bản rút gọn của hồ sơ để trả JSON (không kèm lá số — nặng).
#[derive(Debug, Serialize)]
pub struct HoSoTomTat {
    pub id: i64,
    pub ten: String,
    pub ngay_sinh: String,
    pub gio: u8,
    pub phut: u8,
    pub gioi_tinh: String,
    pub ghi_chu: String,
    pub engine_ver: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&HoSo> for HoSoTomTat {
    fn from(h: &HoSo) -> Self {
        Self {
            id: h.id,
            ten: h.ten.clone(),
            ngay_sinh: h.solar_date.clone(),
            gio: h.hour,
            phut: h.minute,
            gioi_tinh: h.gender.clone(),
            ghi_chu: h.ghi_chu.clone(),
            engine_ver: h.engine_ver.clone(),
            created_at: h.created_at.clone(),
            updated_at: h.updated_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nhap(ngay: &str, gio: u8, gt: &str) -> NhapHoSo {
        NhapHoSo {
            ten: "Nguyễn Văn A".into(),
            ngay_sinh: ngay.into(),
            gio,
            phut: 0,
            gioi_tinh: gt.into(),
            ghi_chu: String::new(),
        }
    }

    #[test]
    fn kiem_tra_chan_du_lieu_sai() {
        assert!(kiem_tra(&nhap("1991-10-24", 7, "nam")).is_ok());

        let mut n = nhap("1991-10-24", 7, "nam");
        n.ten = "   ".into();
        assert!(matches!(kiem_tra(&n), Err(LoiHoSo::DuLieu(_))), "tên rỗng phải bị chặn");

        assert!(matches!(
            kiem_tra(&nhap("24/10/1991", 7, "nam")),
            Err(LoiHoSo::DuLieu(_))
        ), "ngày sai định dạng phải bị chặn");

        assert!(matches!(
            kiem_tra(&nhap("1991-10-24", 25, "nam")),
            Err(LoiHoSo::DuLieu(_))
        ), "giờ 25 phải bị chặn");

        assert!(matches!(
            kiem_tra(&nhap("1991-10-24", 7, "khac")),
            Err(LoiHoSo::DuLieu(_))
        ), "giới tính lạ phải bị chặn");
    }

    /// Ngoài phạm vi 1900–2100 phải báo lỗi Engine chứ không panic.
    #[test]
    fn ngoai_pham_vi_bao_loi_engine() {
        let (_, birth, g) = kiem_tra(&nhap("1899-05-05", 7, "nam")).unwrap();
        assert!(matches!(lap_ca_hai(birth, g), Err(LoiHoSo::Engine(_))));
    }

    /// Lá số phải serialize rồi deserialize lại y nguyên — đây là điều kiện để
    /// lưu JSON vào DB rồi hiển thị lại mà không mất dữ liệu.
    #[test]
    fn la_so_roundtrip_qua_json() {
        let (_, birth, g) = kiem_tra(&nhap("1991-10-24", 7, "nam")).unwrap();
        let (tuvi, bat_tu) = lap_ca_hai(birth, g).unwrap();

        let tj = serde_json::to_string(&tuvi).unwrap();
        let bj = serde_json::to_string(&bat_tu).unwrap();
        let tuvi2: TuViChart = serde_json::from_str(&tj).unwrap();
        let bat_tu2: BatTuChart = serde_json::from_str(&bj).unwrap();

        assert_eq!(tuvi.menh_branch, tuvi2.menh_branch);
        assert_eq!(tuvi.than_branch, tuvi2.than_branch);
        assert_eq!(tuvi.cuc, tuvi2.cuc);
        assert_eq!(tuvi.lunar, tuvi2.lunar);
        for (a, b) in tuvi.palaces.iter().zip(tuvi2.palaces.iter()) {
            assert_eq!(a.name, b.name);
            assert_eq!(a.branch, b.branch);
            assert_eq!(a.stars, b.stars);
            assert_eq!(a.truong_sinh, b.truong_sinh);
            assert_eq!(a.is_menh, b.is_menh);
        }
        assert_eq!(bat_tu.year_pillar.can_chi, bat_tu2.year_pillar.can_chi);
        assert_eq!(bat_tu.month_pillar.can_chi, bat_tu2.month_pillar.can_chi);
        assert_eq!(bat_tu.day_pillar.can_chi, bat_tu2.day_pillar.can_chi);
        assert_eq!(bat_tu.hour_pillar.can_chi, bat_tu2.hour_pillar.can_chi);
        assert_eq!(bat_tu.nguhanh_count, bat_tu2.nguhanh_count);
        assert_eq!(
            bat_tu.day_pillar.hidden_stems,
            bat_tu2.day_pillar.hidden_stems
        );
    }
}
