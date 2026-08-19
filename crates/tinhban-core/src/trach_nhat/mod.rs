//! Trạch Nhật (擇日) — xem ngày tốt/xấu.
//!
//! Cho một ngày Dương lịch, [`danh_gia_ngay`] trả về [`DayAssessment`] gồm:
//!
//! | Thành phần            | Neo theo                        | Module |
//! |-----------------------|---------------------------------|--------|
//! | Giờ Hoàng Đạo/Hắc Đạo | Chi của **ngày**                | [`hoang_dao`] |
//! | Ngày Hoàng Đạo/Hắc Đạo| Chi của tháng **Âm lịch**       | [`hoang_dao`] |
//! | 12 Trực               | Chi của tháng **tiết khí**      | [`truc`] |
//! | Tam Nương / Nguyệt Kỵ | Ngày **Âm lịch**                | [`kieng_ky`] |
//! | Sát Chủ               | Chi ngày × tháng **Âm lịch**    | [`kieng_ky`] |
//!
//! **Cột "Neo theo" là phần dễ sai nhất của Trạch Nhật**: ngày Hoàng Đạo dùng
//! tháng Âm lịch còn 12 Trực dùng tháng tiết khí — hai loại tháng khác nhau,
//! lệch nhau vài ngày mỗi tháng. Cả hai lựa chọn đều đã được đối chiếu số liệu
//! thật (xem `README.md` cùng thư mục), đừng "thống nhất" chúng lại.
//!
//! Toàn bộ tính toán ở đây là **thuần offline** — không gọi mạng, không đọc DB.
//! Phần diễn giải chi tiết lấy từ licham365.vn nằm ở tầng `tinhban-api`, và chỉ
//! *bổ sung* lên kết quả này.

pub mod hoang_dao;
pub mod kieng_ky;
pub mod truc;
pub mod types;

pub use types::{
    DayAssessment, HoangDaoHacDao, HourRange, KiengKy, ThanSat, Truc, TrucRating,
};

use crate::astronomy::jd_from_date;
use crate::bat_tu::tiet_khi::current_tiet_khi;
use crate::{
    day_can_chi, month_can_chi, solar_to_lunar, year_can_chi, EarthlyBranch, LunarError,
};
use chrono::{Datelike, Duration, NaiveDate};

/// Đánh giá tốt/xấu cho **một ngày Dương lịch**.
///
/// Trả [`LunarError::OutOfRange`] nếu `date` ngoài phạm vi 1900–2100 mà lõi âm
/// lịch hỗ trợ.
pub fn danh_gia_ngay(date: NaiveDate) -> Result<DayAssessment, LunarError> {
    let lunar = solar_to_lunar(date)?;
    let day_cc = day_can_chi(date)?;
    let year_cc = year_can_chi(lunar.year)?;
    let month_cc = month_can_chi(year_cc.stem, lunar.month)?;

    // Chi của tháng Âm lịch: tháng 1 → Dần, tháng 2 → Mão, … (tháng nhuận dùng
    // chung Chi với tháng chính cùng số).
    let thang_am_branch = month_cc.branch;

    let jd = jd_from_date(date.day() as i64, date.month() as i64, date.year() as i64);

    let cac_gio = hoang_dao::cac_gio_trong_ngay(day_cc);
    let gio_hoang_dao = cac_gio.iter().copied().filter(|g| g.is_hoang_dao()).collect();
    let gio_hac_dao = cac_gio
        .iter()
        .copied()
        .filter(|g| !g.is_hoang_dao())
        .collect();

    Ok(DayAssessment {
        solar_date: date,
        lunar_date: lunar,
        day_can_chi: day_cc,
        month_can_chi: month_cc,
        year_can_chi: year_cc,
        tiet_khi: current_tiet_khi(jd).1,
        hoang_dao_hac_dao: hoang_dao::danh_gia_ngay(day_cc, thang_am_branch),
        cac_gio,
        gio_hoang_dao,
        gio_hac_dao,
        truc: truc::truc_of_day(day_cc, jd),
        kieng_ky: kieng_ky::kieng_ky_cua_ngay(lunar, day_cc.branch),
    })
}

/// Đánh giá cho một **khoảng ngày** (bao gồm cả `from` và `to`).
///
/// Trả [`LunarError::InvalidLunarDate`] nếu `to < from`. Dừng ngay ở ngày đầu
/// tiên lỗi (ví dụ vượt 2100) thay vì trả kết quả cụt lặng lẽ.
pub fn danh_gia_khoang(
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<DayAssessment>, LunarError> {
    if to < from {
        return Err(LunarError::InvalidLunarDate(format!(
            "khoảng ngày không hợp lệ: {from} > {to}"
        )));
    }
    let mut out = Vec::new();
    let mut d = from;
    while d <= to {
        out.push(danh_gia_ngay(d)?);
        d += Duration::days(1);
    }
    Ok(out)
}

/// Chi của tháng tiết khí chứa `date` — tiện cho UI muốn hiển thị "tháng Trực".
pub fn chi_thang_tiet_khi(date: NaiveDate) -> EarthlyBranch {
    let jd = jd_from_date(date.day() as i64, date.month() as i64, date.year() as i64);
    let i = crate::bat_tu::tiet_khi::tiet_month_branch_index(jd);
    EarthlyBranch::from_index(i).expect("index < 12")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn khoang_ngay_tra_du_so_ngay() {
        let a = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let b = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let v = danh_gia_khoang(a, b).unwrap();
        assert_eq!(v.len(), 31);
        assert_eq!(v[0].solar_date, a);
        assert_eq!(v[30].solar_date, b);
    }

    #[test]
    fn khoang_nguoc_bao_loi() {
        let a = NaiveDate::from_ymd_opt(2024, 3, 10).unwrap();
        let b = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        assert!(danh_gia_khoang(a, b).is_err());
    }

    #[test]
    fn ngoai_pham_vi_bao_loi() {
        let d = NaiveDate::from_ymd_opt(1899, 12, 31).unwrap();
        assert!(matches!(danh_gia_ngay(d), Err(LunarError::OutOfRange(_))));
    }

    /// Giờ Hoàng Đạo + Hắc Đạo phải hợp lại thành đúng 12 khung giờ, không
    /// chồng lấn.
    #[test]
    fn gio_tot_va_xau_phu_kin_12_khung() {
        let d = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let a = danh_gia_ngay(d).unwrap();
        assert_eq!(a.cac_gio.len(), 12);
        assert_eq!(a.gio_hoang_dao.len(), 6);
        assert_eq!(a.gio_hac_dao.len(), 6);
        assert!(a
            .gio_hoang_dao
            .iter()
            .all(|g| !a.gio_hac_dao.contains(g)));
    }
}
