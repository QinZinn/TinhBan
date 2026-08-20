//! Sao trong lá số Tử Vi — enum `Sao` + phân loại + vị trí an.
//!
//! Module con:
//! - `chinh_tinh`: 14 chính tinh (Tử Vi tinh hệ + Thiên Phủ tinh hệ).
//! - `phu_tinh`: phụ tinh phổ biến (Tả Phù / Hữu Bật / Văn Xương / Văn Khúc /
//!   Thiên Khôi / Thiên Việt / Lộc Tồn / Kình Dương / Đà La / Hỏa Tinh /
//!   Linh Tinh / Thiên Mã / Đào Hoa).

pub mod chinh_tinh;
pub mod phu_tinh;

/// Sao Tử Vi (đại diện cho 14 chính tinh + 14 phụ tinh phổ biến đã implement
/// ở giai đoạn 3). Mỗi variant biết tên tiếng Việt + phân loại.
///
/// Derive `PartialEq, Eq, Hash` để có thể tra bằng btree/hash map nếu cần.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Sao {
    // === 14 chính tinh ===
    TuVi,
    ThienCo,
    ThaiDuong,
    VuKhuc,
    ThienDong,
    LiemTrinh,
    ThienPhu,
    ThaiAm,
    ThamLang,
    CuMon,
    ThienTuong,
    ThienLuong,
    ThatSat,
    PhaQuan,

    // === Phụ tinh đã implement ở giai đoạn 3 ===
    TaPhu,
    HuuBat,
    VanXuong,
    VanKhuc,
    ThienKhoi,
    ThienViet,
    LocTon,
    KinhDuong,
    DaLa,
    HoaTinh,
    LinhTinh,
    ThienMa,
    DaoHoa,
}

/// Phân loại sao.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SaoCategory {
    /// 14 chính tinh (Tử Vi + 6 sao chuỗi Bắc Đẩu + Thiên Phủ + 6 sao chuỗi
    /// Nam Đẩu-Phá Quân).
    ChinhTinh,
    /// Phụ tinh (đã implement ở giai đoạn 3).
    PhuTinh,
}

impl Sao {
    /// Trả phân loại của sao.
    pub fn category(self) -> SaoCategory {
        use Sao::*;
        match self {
            TuVi | ThienCo | ThaiDuong | VuKhuc | ThienDong | LiemTrinh | ThienPhu
            | ThaiAm | ThamLang | CuMon | ThienTuong | ThienLuong | ThatSat
            | PhaQuan => SaoCategory::ChinhTinh,
            _ => SaoCategory::PhuTinh,
        }
    }

    /// Tên tiếng Việt display, có dấu.
    pub fn name_vn(self) -> &'static str {
        use Sao::*;
        match self {
            TuVi => "Tử Vi",
            ThienCo => "Thiên Cơ",
            ThaiDuong => "Thái Dương",
            VuKhuc => "Vũ Khúc",
            ThienDong => "Thiên Đồng",
            LiemTrinh => "Liêm Trinh",
            ThienPhu => "Thiên Phủ",
            ThaiAm => "Thái Âm",
            ThamLang => "Tham Lang",
            CuMon => "Cự Môn",
            ThienTuong => "Thiên Tướng",
            ThienLuong => "Thiên Lương",
            ThatSat => "Thất Sát",
            PhaQuan => "Phá Quân",
            TaPhu => "Tả Phù",
            HuuBat => "Hữu Bật",
            VanXuong => "Văn Xương",
            VanKhuc => "Văn Khúc",
            ThienKhoi => "Thiên Khôi",
            ThienViet => "Thiên Việt",
            LocTon => "Lộc Tồn",
            KinhDuong => "Kình Dương",
            DaLa => "Đà La",
            HoaTinh => "Hỏa Tinh",
            LinhTinh => "Linh Tinh",
            ThienMa => "Thiên Mã",
            DaoHoa => "Đào Hoa",
        }
    }
}

impl Sao {
    /// Toàn bộ sao đã implement, theo thứ tự khai báo: 14 chính tinh rồi tới
    /// phụ tinh.
    ///
    /// Dùng để sinh/kiểm tra dữ liệu từ điển — nếu thêm sao mới vào enum mà quên
    /// viết mục từ điển, test `tu_dien` sẽ đỏ.
    pub const ALL: [Sao; 27] = {
        use Sao::*;
        [
            TuVi, ThienCo, ThaiDuong, VuKhuc, ThienDong, LiemTrinh, ThienPhu,
            ThaiAm, ThamLang, CuMon, ThienTuong, ThienLuong, ThatSat, PhaQuan,
            TaPhu, HuuBat, VanXuong, VanKhuc, ThienKhoi, ThienViet, LocTon,
            KinhDuong, DaLa, HoaTinh, LinhTinh, ThienMa, DaoHoa,
        ]
    };

    /// Slug ổn định dùng làm khoá tra từ điển và đường dẫn URL
    /// (`/tu-dien/sao-tu-vi`).
    ///
    /// Cố ý viết tay thay vì sinh từ `name_vn()`: slug là **khoá dữ liệu**, đổi
    /// slug sẽ làm hỏng link đã lưu, nên nó phải độc lập với việc chỉnh sửa tên
    /// hiển thị.
    pub fn slug(self) -> &'static str {
        use Sao::*;
        match self {
            TuVi => "sao-tu-vi",
            ThienCo => "sao-thien-co",
            ThaiDuong => "sao-thai-duong",
            VuKhuc => "sao-vu-khuc",
            ThienDong => "sao-thien-dong",
            LiemTrinh => "sao-liem-trinh",
            ThienPhu => "sao-thien-phu",
            ThaiAm => "sao-thai-am",
            ThamLang => "sao-tham-lang",
            CuMon => "sao-cu-mon",
            ThienTuong => "sao-thien-tuong",
            ThienLuong => "sao-thien-luong",
            ThatSat => "sao-that-sat",
            PhaQuan => "sao-pha-quan",
            TaPhu => "sao-ta-phu",
            HuuBat => "sao-huu-bat",
            VanXuong => "sao-van-xuong",
            VanKhuc => "sao-van-khuc",
            ThienKhoi => "sao-thien-khoi",
            ThienViet => "sao-thien-viet",
            LocTon => "sao-loc-ton",
            KinhDuong => "sao-kinh-duong",
            DaLa => "sao-da-la",
            HoaTinh => "sao-hoa-tinh",
            LinhTinh => "sao-linh-tinh",
            ThienMa => "sao-thien-ma",
            DaoHoa => "sao-dao-hoa",
        }
    }
}

impl std::fmt::Display for Sao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name_vn())
    }
}