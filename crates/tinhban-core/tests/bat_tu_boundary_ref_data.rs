// =============================================================================
// Case Bát Tự GẦN BIÊN TIẾT KHÍ — bổ sung khi audit Bug #7 (epoch sai).
//
// # Nguồn giá trị kỳ vọng (KHÔNG tự sinh từ code dự án)
//
// Mốc tiết khí lấy từ **Đài Thiên văn Hồng Kông** (Hong Kong Observatory),
// bảng "Gregorian-Lunar Calendar Conversion Table":
//   https://www.hko.gov.hk/en/gts/time/calendar/text/files/T{year}e.txt
// HKO công bố theo giờ HK (UTC+8); đã quy về giờ VN (UTC+7) — mốc rơi trong
// khoảng 00:00–00:59 giờ HK thì lùi 1 ngày khi sang giờ VN.
//
// Từ mốc tiết khí đó, 4 trụ được suy ra bằng các quy tắc Can Chi của giai đoạn 2
// (đã kiểm chứng riêng bằng 163 test vector). Trụ Ngày của cả 11 case còn được
// đối chiếu độc lập với licham365.vn: **11/11 khớp**.
//
// # Vì sao cần bộ case này
//
// 7 case Bát Tự của giai đoạn 4 đều KHÔNG rơi vào vùng bug — nhưng đó là may,
// không phải thiết kế: case5 (2000-05-05) nằm ĐÚNG trên mốc Lập Hạ, chỉ thoát
// vì mốc đó tình cờ thuộc 50% mốc mà bug không đẩy lệch. Bộ case dưới đây cố ý
// nhắm thẳng vào vùng rủi ro.
//
// Index: Can Giáp=0..Quý=9; Chi Tý=0..Hợi=11.
// =============================================================================

#[allow(dead_code)]
pub struct BoundaryCase {
    pub label: &'static str,
    /// (năm, tháng, ngày, giờ) dương lịch.
    pub birth: (i32, u32, u32, u8),
    /// Ghi chú vì sao chọn ca này.
    pub note: &'static str,
    pub bt_year: i32,
    /// 1..12 (Dần..Sửu).
    pub bt_month: u8,
    pub year_can: u8, pub year_chi: u8,
    pub month_can: u8, pub month_chi: u8,
    pub day_can: u8, pub day_chi: u8,
    pub hour_can: u8, pub hour_chi: u8,
    /// `true` nếu mã TRƯỚC khi sửa epoch cho kết quả SAI ở ca này.
    pub sai_truoc_khi_sua: bool,
    /// Kết quả sai của mã cũ (rỗng nếu mã cũ vốn đã đúng).
    pub ket_qua_cu: &'static str,
}

pub const BOUNDARY_CASES: &[BoundaryCase] = &[
    BoundaryCase {
        label: "lx_1980_dung_ngay",
        birth: (1980, 2, 4, 10),
        note: "sinh ĐÚNG ngày Lập Xuân 1980 (mốc thật 4/2, mã cũ tính 5/2)",
        bt_year: 1980, bt_month: 1,
        year_can: 6, year_chi: 8,
        month_can: 4, month_chi: 2,
        day_can: 3, day_chi: 7,
        hour_can: 1, hour_chi: 5,
        sai_truoc_khi_sua: true,
        ket_qua_cu: "trước khi sửa: Năm Kỷ Mùi, Tháng Đinh Sửu",
    },
    BoundaryCase {
        label: "lx_1980_truoc_1n",
        birth: (1980, 2, 3, 10),
        note: "đối chứng: trước Lập Xuân 1 ngày → phải giữ năm Kỷ Mùi",
        bt_year: 1979, bt_month: 12,
        year_can: 5, year_chi: 7,
        month_can: 3, month_chi: 1,
        day_can: 2, day_chi: 6,
        hour_can: 9, hour_chi: 5,
        sai_truoc_khi_sua: false,
        ket_qua_cu: "",
    },
    BoundaryCase {
        label: "lx_1991_dung_ngay",
        birth: (1991, 2, 4, 10),
        note: "sinh ĐÚNG ngày Lập Xuân 1991",
        bt_year: 1991, bt_month: 1,
        year_can: 7, year_chi: 7,
        month_can: 6, month_chi: 2,
        day_can: 1, day_chi: 5,
        hour_can: 7, hour_chi: 5,
        sai_truoc_khi_sua: true,
        ket_qua_cu: "trước khi sửa: Năm Canh Ngọ, Tháng Kỷ Sửu",
    },
    BoundaryCase {
        label: "lx_2000_dung_ngay",
        birth: (2000, 2, 4, 10),
        note: "sinh ĐÚNG ngày Lập Xuân 2000",
        bt_year: 2000, bt_month: 1,
        year_can: 6, year_chi: 4,
        month_can: 4, month_chi: 2,
        day_can: 8, day_chi: 4,
        hour_can: 1, hour_chi: 5,
        sai_truoc_khi_sua: true,
        ket_qua_cu: "trước khi sửa: Năm Kỷ Mão, Tháng Đinh Sửu",
    },
    BoundaryCase {
        label: "lx_2024_dung_ngay",
        birth: (2024, 2, 4, 10),
        note: "sinh ĐÚNG ngày Lập Xuân 2024",
        bt_year: 2024, bt_month: 1,
        year_can: 0, year_chi: 4,
        month_can: 2, month_chi: 2,
        day_can: 4, day_chi: 10,
        hour_can: 3, hour_chi: 5,
        sai_truoc_khi_sua: true,
        ket_qua_cu: "trước khi sửa: Năm Quý Mão, Tháng Ất Sửu",
    },
    BoundaryCase {
        label: "lx_2024_truoc_1n",
        birth: (2024, 2, 3, 10),
        note: "đối chứng: trước Lập Xuân 2024 một ngày",
        bt_year: 2023, bt_month: 12,
        year_can: 9, year_chi: 3,
        month_can: 1, month_chi: 1,
        day_can: 3, day_chi: 9,
        hour_can: 1, hour_chi: 5,
        sai_truoc_khi_sua: false,
        ket_qua_cu: "",
    },
    BoundaryCase {
        label: "lx_2025_dung_ngay",
        birth: (2025, 2, 3, 10),
        note: "sinh ĐÚNG ngày Lập Xuân 2025 (mốc rơi 3/2, không phải 4/2)",
        bt_year: 2025, bt_month: 1,
        year_can: 1, year_chi: 5,
        month_can: 4, month_chi: 2,
        day_can: 9, day_chi: 3,
        hour_can: 3, hour_chi: 5,
        sai_truoc_khi_sua: true,
        ket_qua_cu: "trước khi sửa: Năm Giáp Thìn, Tháng Đinh Sửu",
    },
    BoundaryCase {
        label: "lx_2025_truoc_1n",
        birth: (2025, 2, 2, 10),
        note: "đối chứng: trước Lập Xuân 2025 một ngày",
        bt_year: 2024, bt_month: 12,
        year_can: 0, year_chi: 4,
        month_can: 3, month_chi: 1,
        day_can: 8, day_chi: 2,
        hour_can: 1, hour_chi: 5,
        sai_truoc_khi_sua: false,
        ket_qua_cu: "",
    },
    BoundaryCase {
        label: "tieu_han_2010",
        birth: (2010, 1, 5, 14),
        note: "sinh ĐÚNG ngày Tiểu Hàn 2010 → ranh giới TRỤ THÁNG (Tý→Sửu)",
        bt_year: 2009, bt_month: 12,
        year_can: 5, year_chi: 1,
        month_can: 3, month_chi: 1,
        day_can: 1, day_chi: 3,
        hour_can: 9, hour_chi: 7,
        sai_truoc_khi_sua: true,
        ket_qua_cu: "trước khi sửa: Năm Kỷ Sửu, Tháng Bính Tý",
    },
    BoundaryCase {
        label: "thanh_minh_2020",
        birth: (2020, 4, 4, 14),
        note: "sinh ĐÚNG ngày Thanh Minh 2020 → ranh giới TRỤ THÁNG (Mão→Thìn)",
        bt_year: 2020, bt_month: 3,
        year_can: 6, year_chi: 0,
        month_can: 6, month_chi: 4,
        day_can: 3, day_chi: 1,
        hour_can: 3, hour_chi: 7,
        sai_truoc_khi_sua: true,
        ket_qua_cu: "trước khi sửa: Năm Canh Tý, Tháng Kỷ Mão",
    },
    BoundaryCase {
        label: "bach_lo_1985",
        birth: (1985, 9, 7, 14),
        note: "sinh ĐÚNG ngày Bạch Lộ 1985 (mốc VN 7/9; HKO ghi 8/9 giờ HK)",
        bt_year: 1985, bt_month: 8,
        year_can: 1, year_chi: 1,
        month_can: 1, month_chi: 9,
        day_can: 5, day_chi: 9,
        hour_can: 7, hour_chi: 7,
        sai_truoc_khi_sua: true,
        ket_qua_cu: "trước khi sửa: Năm Ất Sửu, Tháng Giáp Thân",
    },
];
