//! Các nguồn scrape bổ sung.
//!
//! Hiện chỉ có [`licham365`]. Kiến trúc để mở cho nguồn khác: mỗi nguồn tự lo
//! URL + parse của mình và trả về một cấu trúc "các mục diễn giải"; tầng
//! `ngay_tot_xau` chỉ cần biết "có bổ sung được hay không".

pub mod licham365;
