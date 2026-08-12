//! Tinh Bàn — core domain logic.
//!
//! Ở giai đoạn 1 (scaffolding) crate này chỉ chứa các hàm placeholder nhỏ để xác
//! nhận pipeline build/test của workspace. Logic thật (lịch âm, an sao Tử Vi Đẩu
//! Số, Bát Tự/Tứ Trụ, từ điển tử vi, xem ngày tốt/xấu) sẽ được thêm ở các giai
//! đoạn sau, từng module riêng để dễ test độc lập.

/// tên app hiển thị ra UI/log.
pub const APP_NAME: &str = "Tinh Bàn";

pub fn app_name() -> &'static str {
    APP_NAME
}

/// version lấy từ Cargo.toml (workspace.package.version).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Placeholder cho logic Tử Vi của giai đoạn sau.
/// Trả về chuỗi mô tả để các hàm có thể được wiring tạm mà chưa crash.
pub fn placeholder() -> &'static str {
    "chưa implement ở giai đoạn 1"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_name_matches() {
        assert_eq!(app_name(), "Tinh Bàn");
        assert_eq!(APP_NAME, "Tinh Bàn");
    }

    #[test]
    fn version_is_semver_like() {
        let v = version();
        assert!(!v.is_empty(), "version must not be empty");
        // dạng major.minor.patch trở đi, bắt đầu bằng chữ số.
        assert!(
            v.chars().next().is_some_and(|c| c.is_ascii_digit()),
            "version should start with a digit, got: {v}"
        );
    }

    #[test]
    fn placeholder_is_documented() {
        assert!(placeholder().contains("giai đoạn 1"));
    }
}