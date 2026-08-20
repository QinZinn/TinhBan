//! Trang xem ngày tốt/xấu — hiển thị kết quả của `/api/ngay-tot-xau`.

use dioxus::prelude::*;

use crate::ngay_tot_xau::NgayTotXauResponse;

/// Form chọn ngày + kết quả (nếu có).
pub fn trang_ngay(date: &str, kq: Option<&NgayTotXauResponse>) -> Element {
    rsx! {
        h1 { "Xem ngày tốt/xấu" }
        form { class: "card", method: "get", action: "/ngay-tot-xau",
            label { r#for: "date", "Chọn ngày Dương lịch" }
            div { style: "display:flex; gap:.5rem",
                input {
                    id: "date", name: "date", r#type: "date", value: "{date}",
                    min: "1900-01-01", max: "2100-12-31",
                }
                button { r#type: "submit", "Xem" }
            }
        }
        if let Some(r) = kq {
            {ket_qua_ngay(r)}
        }
    }
}

fn ket_qua_ngay(r: &NgayTotXauResponse) -> Element {
    let t = &r.tu_tinh;
    let pill = if t.ngay_hoang_dao { "pill good" } else { "pill bad" };
    rsx! {
        h2 { "{t.ngay_duong} — âm lịch {t.ngay_am}" }
        div { class: "card",
            p {
                span { class: "{pill}", "Ngày {t.ket_luan_ngay}" }
                " "
                span { class: "pill", "Trực {t.truc}" }
                " "
                span { class: "pill", "Tiết {t.tiet_khi}" }
            }
            table {
                tbody {
                    tr { th { "Ngày" } td { "{t.ngay_can_chi}" } }
                    tr { th { "Tháng" } td { "{t.thang_can_chi}" } }
                    tr { th { "Năm" } td { "{t.nam_can_chi}" } }
                    tr {
                        th { "Thần trực ngày" }
                        td { "{t.than_truc_ngay} — {t.y_nghia_than}" }
                    }
                    tr {
                        th { "Giờ Hoàng Đạo" }
                        td {
                            {t.gio_hoang_dao.iter()
                                .map(|g| format!("{} {}", g.chi, g.khung))
                                .collect::<Vec<_>>().join(" · ")}
                        }
                    }
                    tr {
                        th { "Giờ Hắc Đạo" }
                        td { class: "muted",
                            {t.gio_hac_dao.iter()
                                .map(|g| format!("{} {}", g.chi, g.khung))
                                .collect::<Vec<_>>().join(" · ")}
                        }
                    }
                    tr {
                        th { "Trực {t.truc} nên làm" }
                        td { "{t.truc_nen_lam}" }
                    }
                    tr {
                        th { "Không nên" }
                        td { "{t.truc_khong_nen_lam}" }
                    }
                    tr {
                        th { "Kiêng kỵ" }
                        td {
                            if t.kieng_ky.is_empty() {
                                span { class: "muted", "Không có" }
                            } else {
                                for k in t.kieng_ky.iter() {
                                    div { span { class: "pill bad", "{k.ten}" } " {k.y_nghia}" }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(note) = r.ghi_chu.as_ref() {
            div { class: "flash err", "{note}" }
        }

        if let Some(dg) = r.dien_giai.as_ref() {
            h2 { "Diễn giải chi tiết" }
            p { class: "small muted",
                "Nguồn: "
                a { href: "{dg.source_url}", "licham365.vn" }
                {match r.nguon_dien_giai {
                    Some(crate::ngay_tot_xau::NguonDienGiai::Cache) => " (đọc từ cache)",
                    Some(crate::ngay_tot_xau::NguonDienGiai::Scrape) => " (vừa tải mới)",
                    None => "",
                }}
            }
            for s in dg.sections.iter() {
                div { class: "card",
                    h3 { style: "margin-top:0", "{s.title}" }
                    div { style: "white-space:pre-line", "{s.body}" }
                }
            }
        }
    }
}
