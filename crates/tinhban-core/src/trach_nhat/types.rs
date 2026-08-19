//! Kiểu dữ liệu cho Trạch Nhật (chọn ngày tốt/xấu).

use crate::{CanChi, LunarDate};
use chrono::NaiveDate;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ===========================================================================
// 12 vị Thần Sát (Hoàng Đạo / Hắc Đạo)
// ===========================================================================

/// 12 vị Thần trực nhật/trực giờ, theo thứ tự cố định của vòng
/// "Thanh Long thập nhị thần" (青龍十二神).
///
/// Thứ tự này là **bất biến**: vòng luôn chạy Thanh Long → Minh Đường → Thiên
/// Hình → … → Câu Trận rồi lặp lại. Thứ chỉ thay đổi giữa các ngày/tháng là
/// **điểm khởi** của vòng (xem [`super::hoang_dao`]).
///
/// Trong 12 vị, 6 vị là **Hoàng Đạo** (tốt) — Thanh Long, Minh Đường, Kim Quỹ,
/// Thiên Đức, Ngọc Đường, Tư Mệnh — và 6 vị là **Hắc Đạo** (xấu) — Thiên Hình,
/// Chu Tước, Bạch Hổ, Thiên Lao, Nguyên Vũ, Câu Trận.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum ThanSat {
    /// 青龍 — Hoàng Đạo, tốt mọi việc.
    ThanhLong = 0,
    /// 明堂 — Hoàng Đạo, tốt mọi việc.
    MinhDuong = 1,
    /// 天刑 — Hắc Đạo, kỵ kiện tụng.
    ThienHinh = 2,
    /// 朱雀 — Hắc Đạo, kỵ nhập trạch, khai trương.
    ChuTuoc = 3,
    /// 金匱 — Hoàng Đạo, tốt cầu tài, giá thú.
    KimQuy = 4,
    /// 天德 (còn gọi Bảo Quang / Kim Đường) — Hoàng Đạo, tốt mọi việc.
    ThienDuc = 5,
    /// 白虎 — Hắc Đạo, kỵ mai táng.
    BachHo = 6,
    /// 玉堂 — Hoàng Đạo, tốt mọi việc.
    NgocDuong = 7,
    /// 天牢 — Hắc Đạo, kỵ xuất hành.
    ThienLao = 8,
    /// 玄武 — Hắc Đạo, kỵ tranh chấp, mất của.
    NguyenVu = 9,
    /// 司命 — Hoàng Đạo, tốt mọi việc (nhất là ban ngày).
    TuMenh = 10,
    /// 勾陳 — Hắc Đạo, kỵ giá thú, xây cất.
    CauTran = 11,
}

impl ThanSat {
    /// Index 0..11 trong vòng (= giá trị `repr(u8)`).
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Chiều đảo. `None` nếu `i > 11`.
    pub fn from_index(i: u8) -> Option<Self> {
        Some(match i {
            0 => Self::ThanhLong,
            1 => Self::MinhDuong,
            2 => Self::ThienHinh,
            3 => Self::ChuTuoc,
            4 => Self::KimQuy,
            5 => Self::ThienDuc,
            6 => Self::BachHo,
            7 => Self::NgocDuong,
            8 => Self::ThienLao,
            9 => Self::NguyenVu,
            10 => Self::TuMenh,
            11 => Self::CauTran,
            _ => return None,
        })
    }

    /// Tên tiếng Việt có dấu.
    pub fn name_vn(self) -> &'static str {
        match self {
            Self::ThanhLong => "Thanh Long",
            Self::MinhDuong => "Minh Đường",
            Self::ThienHinh => "Thiên Hình",
            Self::ChuTuoc => "Chu Tước",
            Self::KimQuy => "Kim Quỹ",
            Self::ThienDuc => "Thiên Đức",
            Self::BachHo => "Bạch Hổ",
            Self::NgocDuong => "Ngọc Đường",
            Self::ThienLao => "Thiên Lao",
            Self::NguyenVu => "Nguyên Vũ",
            Self::TuMenh => "Tư Mệnh",
            Self::CauTran => "Câu Trận",
        }
    }

    /// `true` nếu là 1 trong 6 vị Hoàng Đạo (tốt).
    pub fn is_hoang_dao(self) -> bool {
        matches!(
            self,
            Self::ThanhLong
                | Self::MinhDuong
                | Self::KimQuy
                | Self::ThienDuc
                | Self::NgocDuong
                | Self::TuMenh
        )
    }

    /// Diễn giải ngắn theo truyền thống dân gian.
    pub fn y_nghia_vn(self) -> &'static str {
        match self {
            Self::ThanhLong => "Hoàng Đạo — tốt mọi việc",
            Self::MinhDuong => "Hoàng Đạo — tốt mọi việc, nhất là gặp gỡ quý nhân",
            Self::ThienHinh => "Hắc Đạo — kỵ kiện tụng, tranh chấp",
            Self::ChuTuoc => "Hắc Đạo — kỵ nhập trạch, khai trương; dễ khẩu thiệt",
            Self::KimQuy => "Hoàng Đạo — tốt cầu tài lộc, giá thú",
            Self::ThienDuc => "Hoàng Đạo — tốt mọi việc, nhất là tế tự, cầu phúc",
            Self::BachHo => "Hắc Đạo — kỵ mai táng, động thổ",
            Self::NgocDuong => "Hoàng Đạo — tốt mọi việc, nhất là hôn thú",
            Self::ThienLao => "Hắc Đạo — kỵ xuất hành, kiện tụng",
            Self::NguyenVu => "Hắc Đạo — kỵ tranh chấp, dễ mất của",
            Self::TuMenh => "Hoàng Đạo — tốt mọi việc, nhất là ban ngày",
            Self::CauTran => "Hắc Đạo — kỵ giá thú, xây cất",
        }
    }
}

/// Kết luận Hoàng Đạo / Hắc Đạo **của cả ngày**, kèm vị Thần trực ngày để người
/// dùng biết kết luận đến từ đâu (thay vì chỉ một nhãn tốt/xấu trần trụi).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HoangDaoHacDao {
    /// Vị Thần trực ngày.
    pub than: ThanSat,
    /// `true` = ngày Hoàng Đạo (tốt), `false` = ngày Hắc Đạo (xấu).
    pub is_hoang_dao: bool,
}

impl HoangDaoHacDao {
    /// "Hoàng Đạo" / "Hắc Đạo".
    pub fn nhan_vn(&self) -> &'static str {
        if self.is_hoang_dao {
            "Hoàng Đạo"
        } else {
            "Hắc Đạo"
        }
    }
}

// ===========================================================================
// Khung giờ
// ===========================================================================

/// Một trong 12 khung giờ Địa Chi của ngày (mỗi khung 2 tiếng), kèm vị Thần
/// trực giờ đó.
///
/// Giờ Tý bắt đầu lúc **23:00** (đêm hôm trước theo đồng hồ) và kết thúc
/// 00:59 — đây là khung duy nhất vắt qua nửa đêm, nên `start_hour = 23` và
/// `end_hour = 0`; đừng giả định `start_hour < end_hour`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HourRange {
    /// Địa Chi của khung giờ (Tý, Sửu, …).
    pub branch: crate::EarthlyBranch,
    /// Vị Thần trực khung giờ này.
    pub than: ThanSat,
    /// Giờ bắt đầu (0..=23). Giờ Tý = 23.
    pub start_hour: u8,
    /// Giờ cuối cùng thuộc khung (0..=23), tức khung kết thúc lúc
    /// `end_hour:59`. Giờ Tý = 0.
    pub end_hour: u8,
}

impl HourRange {
    /// `true` nếu khung giờ này do một vị Hoàng Đạo trực (giờ tốt).
    pub fn is_hoang_dao(&self) -> bool {
        self.than.is_hoang_dao()
    }

    /// Nhãn hiển thị, ví dụ `"Tý (23:00–00:59)"`.
    pub fn label_vn(&self) -> String {
        format!(
            "{} ({:02}:00–{:02}:59)",
            self.branch.name_vn(),
            self.start_hour,
            self.end_hour
        )
    }
}

// ===========================================================================
// 12 Trực
// ===========================================================================

/// 12 Trực (Kiến Trừ Thập Nhị Khách / 建除十二神), theo thứ tự cố định.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum Truc {
    /// 建 — Kiến.
    Kien = 0,
    /// 除 — Trừ.
    Tru = 1,
    /// 滿 — Mãn.
    Man = 2,
    /// 平 — Bình.
    Binh = 3,
    /// 定 — Định.
    Dinh = 4,
    /// 執 — Chấp.
    Chap = 5,
    /// 破 — Phá.
    Pha = 6,
    /// 危 — Nguy.
    Nguy = 7,
    /// 成 — Thành.
    Thanh = 8,
    /// 收 — Thu.
    Thu = 9,
    /// 開 — Khai.
    Khai = 10,
    /// 閉 — Bế.
    Be = 11,
}

impl Truc {
    /// Index 0..11.
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Chiều đảo. `None` nếu `i > 11`.
    pub fn from_index(i: u8) -> Option<Self> {
        Some(match i {
            0 => Self::Kien,
            1 => Self::Tru,
            2 => Self::Man,
            3 => Self::Binh,
            4 => Self::Dinh,
            5 => Self::Chap,
            6 => Self::Pha,
            7 => Self::Nguy,
            8 => Self::Thanh,
            9 => Self::Thu,
            10 => Self::Khai,
            11 => Self::Be,
            _ => return None,
        })
    }

    /// Tên tiếng Việt có dấu.
    pub fn name_vn(self) -> &'static str {
        match self {
            Self::Kien => "Kiến",
            Self::Tru => "Trừ",
            Self::Man => "Mãn",
            Self::Binh => "Bình",
            Self::Dinh => "Định",
            Self::Chap => "Chấp",
            Self::Pha => "Phá",
            Self::Nguy => "Nguy",
            Self::Thanh => "Thành",
            Self::Thu => "Thu",
            Self::Khai => "Khai",
            Self::Be => "Bế",
        }
    }

    /// Việc **nên** làm trong ngày mang Trực này.
    pub fn nen_lam_vn(self) -> &'static str {
        match self {
            Self::Kien => "Xuất hành, gặp quan chức, cầu danh, tuyển dụng",
            Self::Tru => "Chữa bệnh, trừ tà, giải trừ điều xấu, dọn dẹp, cúng tế",
            Self::Man => "Cầu tài lộc, khai trương, tế tự, nhập kho",
            Self::Binh => "Cưới hỏi, san nền, sửa đường, hòa giải tranh chấp",
            Self::Dinh => "Cưới hỏi, nhập học, ký kết, nhận chức, trồng cây",
            Self::Chap => "Lập khế ước, giao dịch, động thổ san nền, cầu thầy chữa bệnh",
            Self::Pha => "Phá dỡ nhà cũ, chữa răng, trị bệnh mãn tính",
            Self::Nguy => "Tế tự, cầu phúc, an thần vị",
            Self::Thanh => "Nhập học, khai trương, cưới hỏi, làm nhà, xuất hành",
            Self::Thu => "Thu nợ, nhập kho, mua vào, cầu tài, đánh bắt",
            Self::Khai => "Khai trương, nhập học, làm nhà, cưới hỏi, xuất hành",
            Self::Be => "Xây đắp tường, đặt táng, gắn cửa, làm cầu, trị bệnh (trừ bệnh mắt)",
        }
    }

    /// Việc **không nên** làm trong ngày mang Trực này.
    pub fn khong_nen_lam_vn(self) -> &'static str {
        match self {
            Self::Kien => "Động thổ, đào giếng, mai táng",
            Self::Tru => "Xuất hành xa, cưới hỏi, khai trương",
            Self::Man => "Uống thuốc, chữa bệnh, mai táng",
            Self::Binh => "Đào giếng, khơi mương, động thổ",
            Self::Dinh => "Kiện tụng, xuất hành, chữa bệnh",
            Self::Chap => "Xuất hành xa, di dời, mở kho",
            Self::Pha => "Hầu hết mọi việc trọng đại — cưới hỏi, khai trương, làm nhà, ký kết",
            Self::Nguy => "Xuất hành, leo cao, đi thuyền, mạo hiểm",
            Self::Thanh => "Kiện tụng, tranh chấp",
            Self::Thu => "Xuất tiền, cho vay, an táng, khai trương",
            Self::Khai => "Mai táng, động thổ đào huyệt",
            Self::Be => "Nhận chức, nhập học, chữa bệnh mắt, chăn nuôi",
        }
    }

    /// Đánh giá thô "tốt / xấu / trung bình" của Trực xét chung mọi việc.
    ///
    /// Chỉ là gợi ý nhanh — ý nghĩa thật của Trực phụ thuộc **loại việc** định
    /// làm (xem [`nen_lam_vn`](Self::nen_lam_vn) /
    /// [`khong_nen_lam_vn`](Self::khong_nen_lam_vn)).
    pub fn danh_gia_chung(self) -> TrucRating {
        match self {
            Self::Tru | Self::Dinh | Self::Thanh | Self::Khai => TrucRating::Tot,
            Self::Pha | Self::Nguy | Self::Be => TrucRating::Xau,
            _ => TrucRating::TrungBinh,
        }
    }
}

/// Đánh giá thô của một Trực xét chung mọi việc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TrucRating {
    Tot,
    TrungBinh,
    Xau,
}

// ===========================================================================
// Kiêng kỵ
// ===========================================================================

/// Một điều kiêng kỵ phổ biến áp dụng cho ngày đang xét.
///
/// Các mục này **độc lập** với Hoàng Đạo/Hắc Đạo: một ngày Hoàng Đạo vẫn có thể
/// rơi vào Tam Nương. Vì vậy chúng được trả về thành danh sách riêng, KHÔNG gộp
/// vào một điểm số duy nhất — người dùng tự quyết định mình quan tâm điều nào.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum KiengKy {
    /// Ngày Tam Nương — mùng 3, 7, 13, 18, 22, 27 Âm lịch.
    TamNuong,
    /// Ngày Nguyệt Kỵ — mùng 5, 14, 23 Âm lịch.
    NguyetKy,
    /// Ngày Sát Chủ — theo Chi của ngày, tra theo tháng Âm lịch.
    SatChu,
}

impl KiengKy {
    /// Tên tiếng Việt có dấu.
    pub fn name_vn(self) -> &'static str {
        match self {
            Self::TamNuong => "Tam Nương",
            Self::NguyetKy => "Nguyệt Kỵ",
            Self::SatChu => "Sát Chủ",
        }
    }

    /// Diễn giải ngắn.
    pub fn y_nghia_vn(self) -> &'static str {
        match self {
            Self::TamNuong => "Trăm sự đều kỵ, chính kỵ xuất hành, cưới hỏi",
            Self::NguyetKy => "Kỵ khởi sự, xuất hành, cưới hỏi, khai trương",
            Self::SatChu => "Kỵ xây cất, cưới gả, động thổ, an táng",
        }
    }
}

// ===========================================================================
// Kết quả tổng hợp
// ===========================================================================

/// Kết quả đánh giá tốt/xấu cho **một ngày Dương lịch**.
///
/// Đây là output của phần *tự tính thuật toán* — nguồn chính, không phụ thuộc
/// mạng. Tầng scrape (giai đoạn 5, phần B) chỉ **bổ sung** văn bản diễn giải
/// lên trên cấu trúc này, không tính lại bất cứ trường nào ở đây.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DayAssessment {
    /// Ngày Dương lịch được xét.
    pub solar_date: NaiveDate,
    /// Ngày Âm lịch tương ứng.
    pub lunar_date: LunarDate,
    /// Can Chi của ngày.
    pub day_can_chi: CanChi,
    /// Can Chi của tháng Âm lịch chứa ngày này.
    pub month_can_chi: CanChi,
    /// Can Chi của năm Âm lịch chứa ngày này.
    pub year_can_chi: CanChi,
    /// Tiết khí (trong 24) đang có hiệu lực, ví dụ `"Kinh Trập"`.
    pub tiet_khi: &'static str,
    /// Ngày Hoàng Đạo hay Hắc Đạo (kèm vị Thần trực ngày).
    pub hoang_dao_hac_dao: HoangDaoHacDao,
    /// Toàn bộ 12 khung giờ trong ngày kèm Thần trực giờ, theo thứ tự
    /// Tý → Hợi.
    pub cac_gio: Vec<HourRange>,
    /// Các khung giờ **tốt** (do Hoàng Đạo trực) — tập con của [`cac_gio`](Self::cac_gio).
    pub gio_hoang_dao: Vec<HourRange>,
    /// Các khung giờ **xấu** (do Hắc Đạo trực) — tập con của [`cac_gio`](Self::cac_gio).
    pub gio_hac_dao: Vec<HourRange>,
    /// Trực của ngày (12 Trực).
    pub truc: Truc,
    /// Các điều kiêng kỵ áp dụng cho ngày này (có thể rỗng).
    pub kieng_ky: Vec<KiengKy>,
}
