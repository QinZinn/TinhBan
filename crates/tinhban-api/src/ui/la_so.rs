//! Vẽ lá số: bàn 12 cung Tử Vi (lưới 4×4) và bảng Tứ Trụ Bát Tự.

use dioxus::prelude::*;
use tinhban_core::{
    can_chi_display, hour_can_chi, BatTuChart, HeavenlyStem, Palace,
    SaoCategory, TuViChart,
};

/// Vị trí của 12 Địa Chi trên lưới 4×4 của địa bàn, theo cách vẽ truyền thống.
///
/// ```text
///   Tỵ    Ngọ   Mùi   Thân
///   Thìn  ┌─ thông tin ─┐  Dậu
///   Mão   └─  giữa bàn ─┘  Tuất
///   Dần   Sửu   Tý    Hợi
/// ```
///
/// Đi từ Dần (góc dưới trái) theo chiều kim đồng hồ là đúng thứ tự 12 Chi. Bảng
/// dưới đây map `EarthlyBranch::index()` → `(hàng, cột)` 1-based của CSS Grid.
const O_LUOI: [(u8, u8); 12] = [
    (4, 3), // 0  Tý
    (4, 2), // 1  Sửu
    (4, 1), // 2  Dần
    (3, 1), // 3  Mão
    (2, 1), // 4  Thìn
    (1, 1), // 5  Tỵ
    (1, 2), // 6  Ngọ
    (1, 3), // 7  Mùi
    (1, 4), // 8  Thân
    (2, 4), // 9  Dậu
    (3, 4), // 10 Tuất
    (4, 4), // 11 Hợi
];

/// Một ô cung trên địa bàn.
fn o_cung(palace: &Palace) -> Element {
    let (r, c) = O_LUOI[palace.branch.index() as usize];
    let style = format!("grid-row: {r}; grid-column: {c};");
    let class = if palace.is_menh { "cung menh" } else { "cung" };

    let chinh: Vec<_> = palace
        .stars
        .iter()
        .filter(|s| s.category() == SaoCategory::ChinhTinh)
        .copied()
        .collect();
    let phu: Vec<_> = palace
        .stars
        .iter()
        .filter(|s| s.category() == SaoCategory::PhuTinh)
        .copied()
        .collect();

    rsx! {
        div { class: "{class}", style: "{style}",
            span { class: "chi", "{palace.branch.name_vn()}" }
            span { class: "ten",
                "{palace.name.name_vn()}"
                if palace.is_than { span { class: "muted small", " · Thân" } }
            }
            div { class: "sao",
                for s in chinh {
                    div {
                        a { class: "ct", href: "/tu-dien/{s.slug()}", "{s.name_vn()}" }
                    }
                }
                for s in phu {
                    div {
                        a { class: "pt", href: "/tu-dien/{s.slug()}", "{s.name_vn()}" }
                    }
                }
            }
            if !palace.truong_sinh.is_empty() {
                div { class: "ts",
                    {palace.truong_sinh.iter().map(|t| t.name_vn()).collect::<Vec<_>>().join(", ")}
                }
            }
        }
    }
}

/// Bàn 12 cung Tử Vi.
pub fn ban_tu_vi(chart: &TuViChart) -> Element {
    let lunar = chart.lunar;
    // Địa Chi của giờ sinh. Dùng lại `hour_can_chi` của lõi thay vì tự tính —
    // Can truyền vào không ảnh hưởng tới Chi, nên đưa Giáp làm giá trị bù.
    let gio = hour_can_chi(HeavenlyStem::Giap, chart.birth.hour)
        .map(|cc| cc.branch.name_vn())
        .unwrap_or("");
    rsx! {
        div { class: "diaban",
            for p in chart.palaces.iter() {
                {o_cung(p)}
            }
            div { class: "giua",
                dl {
                    dt { "Dương lịch" }
                    dd { "{chart.birth.solar_date} · {chart.birth.hour:02}:{chart.birth.minute:02}" }
                    dt { "Âm lịch" }
                    dd {
                        "{lunar.day}/{lunar.month}/{lunar.year}"
                        if lunar.is_leap_month { " (nhuận)" }
                    }
                    dt { "Giờ sinh" }
                    dd { "{gio}" }
                    dt { "Giới tính" }
                    dd { {gioi_tinh_vn(chart.gender)} }
                    dt { "Cục" }
                    dd { "{chart.cuc.name_vn()}" }
                    dt { "Mệnh tại" }
                    dd { "{chart.menh_branch.name_vn()}" }
                    dt { "Thân tại" }
                    dd { "{chart.than_branch.name_vn()}" }
                }
                p { class: "small muted", style: "margin:.6rem 0 0",
                    "Bấm tên sao để xem giải nghĩa trong từ điển."
                }
            }
        }
    }
}

fn gioi_tinh_vn(g: tinhban_core::Gender) -> &'static str {
    match g {
        tinhban_core::Gender::Nam => "Nam",
        tinhban_core::Gender::Nu => "Nữ",
    }
}

/// Bảng Tứ Trụ Bát Tự: 4 cột (Năm/Tháng/Ngày/Giờ) × các hàng thuộc tính.
pub fn bang_bat_tu(chart: &BatTuChart) -> Element {
    let tru = [
        ("Năm", &chart.year_pillar),
        ("Tháng", &chart.month_pillar),
        ("Ngày", &chart.day_pillar),
        ("Giờ", &chart.hour_pillar),
    ];
    let n = &chart.nguhanh_count;
    rsx! {
        table {
            thead {
                tr {
                    th { "" }
                    for (ten, _) in tru.iter() { th { "Trụ {ten}" } }
                }
            }
            tbody {
                tr {
                    th { "Can Chi" }
                    for (_, p) in tru.iter() {
                        td { strong { {can_chi_display(p.can_chi)} } }
                    }
                }
                tr {
                    th { "Thiên Can" }
                    for (_, p) in tru.iter() { td { "{p.can_chi.stem.name_vn()}" } }
                }
                tr {
                    th { "Địa Chi" }
                    for (_, p) in tru.iter() { td { "{p.can_chi.branch.name_vn()}" } }
                }
                tr {
                    th { "Thập Thần" }
                    for (_, p) in tru.iter() {
                        td {
                            match p.ten_god {
                                Some(g) => rsx! { "{g.name_vn()}" },
                                // Trụ Ngày chính là Nhật Chủ nên không có Thập Thần.
                                None => rsx! { span { class: "muted", "Nhật Chủ" } },
                            }
                        }
                    }
                }
                tr {
                    th { "Tàng Can" }
                    for (_, p) in tru.iter() {
                        td { class: "small",
                            for (can, tg) in p.hidden_stems.iter() {
                                div { "{can.name_vn()} — {tg.name_vn()}" }
                            }
                        }
                    }
                }
            }
        }
        p { class: "small muted",
            "Nhật Chủ: "
            strong { "{chart.day_pillar.can_chi.stem.name_vn()}" }
            " · Ngũ Hành trong 8 chữ — "
            "Kim {n.kim}, Mộc {n.moc}, Thủy {n.thuy}, Hỏa {n.hoa}, Thổ {n.tho}"
        }
    }
}
