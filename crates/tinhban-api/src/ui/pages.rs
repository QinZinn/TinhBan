//! Các trang của app.

use dioxus::prelude::*;
use tinhban_core::{BatTuChart, TuViChart};
use tinhban_db::{HoSo, MucTuDien};

use super::la_so::{ban_tu_vi, bang_bat_tu};

/// Trang chủ.
pub fn trang_chu(so_ho_so: usize, so_muc_tu_dien: usize) -> Element {
    rsx! {
        h1 { "Tinh Bàn" }
        p { class: "muted", "Toolkit tử vi cá nhân — Tử Vi Đẩu Số, Bát Tự, Trạch Nhật." }
        div { class: "row",
            div { class: "card",
                h2 { style: "margin-top:0", "Lập lá số" }
                p { class: "small muted", "Nhập ngày giờ sinh, xem Tử Vi và Bát Tự cùng lúc." }
                a { class: "btn", href: "/la-so/moi", "Lập lá số mới" }
            }
            div { class: "card",
                h2 { style: "margin-top:0", "Hồ sơ đã lưu" }
                p { class: "small muted", "{so_ho_so} hồ sơ." }
                a { class: "btn btn-ghost", href: "/ho-so", "Xem danh sách" }
            }
            div { class: "card",
                h2 { style: "margin-top:0", "Từ điển" }
                p { class: "small muted", "{so_muc_tu_dien} mục: chính tinh, phụ tinh, 12 cung." }
                a { class: "btn btn-ghost", href: "/tu-dien", "Tra cứu" }
            }
            div { class: "card",
                h2 { style: "margin-top:0", "Ngày tốt/xấu" }
                p { class: "small muted", "Hoàng Đạo, giờ tốt, 12 Trực, kiêng kỵ." }
                a { class: "btn btn-ghost", href: "/ngay-tot-xau", "Xem ngày" }
            }
        }
    }
}

/// Form lập lá số mới.
pub fn form_la_so(loi: Option<&str>) -> Element {
    rsx! {
        h1 { "Lập lá số mới" }
        if let Some(msg) = loi {
            div { class: "flash err", "{msg}" }
        }
        form { class: "card", method: "post", action: "/la-so/moi",
            label { r#for: "ten", "Họ tên" }
            input { id: "ten", name: "ten", required: true, placeholder: "Nguyễn Văn A" }

            div { class: "row",
                div {
                    label { r#for: "ngay_sinh", "Ngày sinh (Dương lịch)" }
                    input {
                        id: "ngay_sinh", name: "ngay_sinh", r#type: "date",
                        required: true, min: "1900-01-01", max: "2100-12-31",
                    }
                }
                div {
                    label { r#for: "gio", "Giờ sinh" }
                    input {
                        id: "gio", name: "gio", r#type: "number",
                        min: "0", max: "23", value: "12", required: true,
                    }
                }
                div {
                    label { r#for: "phut", "Phút" }
                    input {
                        id: "phut", name: "phut", r#type: "number",
                        min: "0", max: "59", value: "0", required: true,
                    }
                }
                div {
                    label { r#for: "gioi_tinh", "Giới tính" }
                    select { id: "gioi_tinh", name: "gioi_tinh",
                        option { value: "nam", "Nam" }
                        option { value: "nu", "Nữ" }
                    }
                }
            }

            label { r#for: "ghi_chu", "Ghi chú" }
            textarea { id: "ghi_chu", name: "ghi_chu", placeholder: "Tuỳ chọn…" }

            p { class: "small muted",
                "Giờ sinh dùng đồng hồ 24h theo giờ Việt Nam. Giờ Tý bắt đầu 23:00. "
                "Phạm vi hỗ trợ: 1900–2100."
            }
            p { style: "margin-bottom:0", button { r#type: "submit", "Lập lá số và lưu hồ sơ" } }
        }
    }
}

/// Chi tiết một hồ sơ: thông tin + lá số Tử Vi + Bát Tự.
pub fn chi_tiet_ho_so(
    h: &HoSo,
    tuvi: Option<&TuViChart>,
    bat_tu: Option<&BatTuChart>,
    canh_bao_ver: Option<String>,
) -> Element {
    let ten = h.ten.clone();
    let ngay = h.solar_date.clone();
    let (hh, mm) = (h.hour, h.minute);
    let gt = if h.gender == "nam" { "Nam" } else { "Nữ" };
    let ghi_chu = h.ghi_chu.clone();
    let id = h.id;
    let created = h.created_at.clone();

    rsx! {
        h1 { "{ten}" }
        p { class: "muted",
            "{ngay} · {hh:02}:{mm:02} · {gt}"
            span { class: "small", " · lưu lúc {created}" }
        }
        if let Some(msg) = canh_bao_ver {
            div { class: "flash err", "{msg}" }
        }

        h2 { "Lá số Tử Vi Đẩu Số" }
        match tuvi {
            Some(c) => ban_tu_vi(c),
            None => rsx! { div { class: "flash err", "Không đọc được lá số Tử Vi đã lưu." } },
        }

        h2 { "Tứ Trụ Bát Tự" }
        match bat_tu {
            Some(c) => bang_bat_tu(c),
            None => rsx! { div { class: "flash err", "Không đọc được lá số Bát Tự đã lưu." } },
        }

        h2 { "Ghi chú" }
        form { class: "card", method: "post", action: "/ho-so/{id}/sua",
            label { r#for: "ten_moi", "Họ tên" }
            input { id: "ten_moi", name: "ten", value: "{ten}", required: true }
            label { r#for: "ghi_chu", "Ghi chú" }
            textarea { id: "ghi_chu", name: "ghi_chu", "{ghi_chu}" }
            p { class: "small muted",
                "Không sửa được ngày giờ sinh: đổi ngày sinh là một lá số khác, "
                "nên hãy lập hồ sơ mới thay vì sửa tại chỗ."
            }
            p { style: "margin-bottom:0", button { r#type: "submit", "Lưu thay đổi" } }
        }

        form {
            method: "post", action: "/ho-so/{id}/xoa",
            style: "margin-top:1rem",
            button {
                r#type: "submit",
                class: "btn-ghost",
                style: "border-color:#9a2222;color:#9a2222",
                "Xoá hồ sơ này"
            }
        }
    }
}

/// Danh sách hồ sơ.
pub fn danh_sach_ho_so(ds: &[HoSo]) -> Element {
    rsx! {
        h1 { "Hồ sơ đã lưu" }
        p { a { class: "btn", href: "/la-so/moi", "Lập lá số mới" } }
        if ds.is_empty() {
            div { class: "card muted", "Chưa có hồ sơ nào." }
        } else {
            table {
                thead {
                    tr {
                        th { "Họ tên" }
                        th { "Ngày sinh" }
                        th { "Giờ" }
                        th { "Giới tính" }
                        th { "Ghi chú" }
                        th { "Tạo lúc" }
                    }
                }
                tbody {
                    for h in ds.iter() {
                        tr {
                            td { a { href: "/ho-so/{h.id}", "{h.ten}" } }
                            td { "{h.solar_date}" }
                            td { "{h.hour:02}:{h.minute:02}" }
                            td { {if h.gender == "nam" { "Nam" } else { "Nữ" }} }
                            td { class: "small muted", "{h.ghi_chu}" }
                            td { class: "small muted", "{h.created_at}" }
                        }
                    }
                }
            }
        }
    }
}

/// Trang từ điển: ô tìm kiếm + kết quả.
pub fn trang_tu_dien(q: &str, ket_qua: &[MucTuDien], la_tim_kiem: bool) -> Element {
    let nhom = |k: &str| -> Vec<&MucTuDien> {
        ket_qua.iter().filter(|m| m.kind == k).collect()
    };
    rsx! {
        h1 { "Từ điển Tử Vi" }
        form { class: "card", method: "get", action: "/tu-dien",
            label { r#for: "q", "Tìm sao hoặc cung" }
            div { style: "display:flex; gap:.5rem",
                input {
                    id: "q", name: "q", value: "{q}",
                    placeholder: "vd: tu vi, dao hoa, cung menh… (gõ không dấu cũng được)",
                }
                button { r#type: "submit", "Tìm" }
            }
        }

        if la_tim_kiem {
            h2 { "Kết quả cho \"{q}\" ({ket_qua.len()})" }
            if ket_qua.is_empty() {
                div { class: "card muted", "Không tìm thấy mục nào." }
            } else {
                {danh_sach_muc(ket_qua)}
            }
        } else {
            for (kind, ten) in [
                ("chinh-tinh", "14 chính tinh"),
                ("phu-tinh", "Phụ tinh"),
                ("cung", "12 cung"),
            ] {
                h2 { "{ten}" }
                {danh_sach_muc(&nhom(kind).into_iter().cloned().collect::<Vec<_>>())}
            }
        }
    }
}

fn danh_sach_muc(muc: &[MucTuDien]) -> Element {
    rsx! {
        div { class: "row",
            for m in muc.iter() {
                div { class: "card", style: "flex:1 1 14rem",
                    div { a { href: "/tu-dien/{m.slug}", strong { "{m.title}" } } }
                    div { class: "small muted",
                        if !m.nhom.is_empty() { span { class: "pill", "{m.nhom}" } }
                        if !m.nguhanh.is_empty() {
                            " "
                            span { class: "pill", "{m.nguhanh}" }
                        }
                        if !m.amduong.is_empty() {
                            " "
                            span { class: "pill", "{m.amduong}" }
                        }
                    }
                    if !m.aliases.is_empty() {
                        div { class: "small muted", style: "margin-top:.3rem", "{m.aliases}" }
                    }
                }
            }
        }
    }
}

/// Chi tiết một mục từ điển. `body_html` đã render sẵn từ markdown.
pub fn chi_tiet_tu_dien(m: &MucTuDien, body_html: String) -> Element {
    let kind_vn = match m.kind.as_str() {
        "chinh-tinh" => "Chính tinh",
        "phu-tinh" => "Phụ tinh",
        _ => "Cung",
    };
    rsx! {
        p { class: "small", a { href: "/tu-dien", "← Từ điển" } }
        h1 { "{m.title}" }
        p {
            span { class: "pill", "{kind_vn}" }
            if !m.nhom.is_empty() {
                " "
                span { class: "pill", "{m.nhom}" }
            }
            if !m.nguhanh.is_empty() {
                " "
                span { class: "pill", "Hành {m.nguhanh}" }
            }
            if !m.amduong.is_empty() {
                " "
                span { class: "pill", "{m.amduong}" }
            }
        }
        if !m.aliases.is_empty() {
            p { class: "muted small", "Tên gọi khác: {m.aliases}" }
        }
        div { class: "card md", dangerous_inner_html: "{body_html}" }
    }
}
