//! Thuật toán thiên văn của Hồ Ngọc Đức — nội bộ (private). Tất cả hàm trong
//! module này làm việc trên số nguyên JD (= trưa UTC của ngày dương lịch)
//! và trả về thông tin âm lịch dạng thô (i64) để `lib.rs` convert ra public
//! types (`LunarDate`, v.v.).
//!
//! Đối chiếu coefficient với `doanguyen/lasotuvi/Lich_HND.py` (nguồn chính thức
//! từ tác giả Hồ Ngọc Đức), `vanng822/ramlich/amlich/src/fns.rs`,
//! `kunkka19xx/look/core/lunar/src/lib.rs`, `J2TEAM/vibe.j2team.org/.../lunar.ts`.

use std::f64::consts::PI;

/// Múi giờ Việt Nam (UTC+7), đơn vị giờ.
pub const VN_TZ: f64 = 7.0;

const NM_EPOCH_JD: f64 = 2415021.076998695;
const SYNODIC_MONTH: f64 = 29.530588853;

// ===========================================================================
// JD <-> Gregorian
// ===========================================================================

/// Julian Day Number tại trưa UTC của `dd/mm/yyyy` Dương lịch.
/// Phần Gregorian (≥ 2299161, i.e. ≥ 15/10/1582) và Julian (cũ).
#[allow(clippy::collapsible_if)]
pub fn jd_from_date(dd: i64, mm: i64, yy: i64) -> i64 {
    let a = (14 - mm) / 12;
    let y = yy + 4800 - a;
    let m = mm + 12 * a - 3;
    let mut jd =
        dd + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    if jd < 2299161 {
        jd = dd + (153 * m + 2) / 5 + 365 * y + y / 4 - 32083;
    }
    jd
}

/// Đảo của [`jd_from_date`], trả về (day, month, year) Gregorian.
pub fn jd_to_date(jd: i64) -> (i64, i64, i64) {
    let jd = jd as f64;
    let (b, c) = if jd > 2299160.0 {
        let a = jd + 32044.0;
        let b = ((4.0 * a + 3.0) / 146097.0).floor();
        let c = a - (b * 146097.0 / 4.0).floor();
        (b, c)
    } else {
        (0.0, jd + 32082.0)
    };
    let d = ((4.0 * c + 3.0) / 1461.0).floor();
    let e = c - (1461.0 * d / 4.0).floor();
    let m = ((5.0 * e + 2.0) / 153.0).floor();
    let day = e - ((153.0 * m + 2.0) / 5.0).floor() + 1.0;
    let month = m + 3.0 - 12.0 * (m / 10.0).floor();
    let year = b * 100.0 + d - 4800.0 + (m / 10.0).floor();
    (day as i64, month as i64, year as i64)
}

// ===========================================================================
// Thiên văn: NewMoon, SunLongitude, getSunLongitude, getNewMoonDay
// ===========================================================================

/// Julian Date (số thực) của lần sóc (new moon) thứ `k` kể từ epoch 1/1/1900.
///
/// Toán đầy đủ của Ho Ngoc Duc, gồm các hiệu chỉnh số dư (mean anomaly M,
/// M' của Mặt Trăng, F = argument of latitude, và hai chu kỳ phụ ±2F).
#[allow(clippy::excessive_precision)]
fn new_moon(k: i64) -> f64 {
    let k = k as f64;
    let t = k / 1236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let dr = PI / 180.0;
    let mut jd1 = 2415020.75933 + 29.53058868 * k + 0.0001178 * t2 - 0.000000155 * t3;
    jd1 += 0.00033 * ((166.56 + 132.87 * t - 0.009173 * t2) * dr).sin();
    let m = 359.2242 + 29.10535608 * k - 0.0000333 * t2 - 0.00000347 * t3;
    let mpr = 306.0253 + 385.81691806 * k + 0.0107306 * t2 + 0.00001236 * t3;
    let f = 21.2964 + 390.67050646 * k - 0.0016528 * t2 - 0.00000239 * t3;
    let mut c1 = (0.1734 - 0.000393 * t) * (m * dr).sin() + 0.0021 * (dr * 2.0 * m).sin();
    c1 -= 0.4068 * (mpr * dr).sin() + 0.0161 * (dr * 2.0 * mpr).sin();
    c1 -= 0.0004 * (dr * 3.0 * mpr).sin();
    c1 += 0.0104 * (dr * 2.0 * f).sin() - 0.0051 * (dr * (m + mpr)).sin();
    c1 -= 0.0074 * (dr * (m - mpr)).sin() + 0.0004 * (dr * (2.0 * f + m)).sin();
    c1 -= 0.0004 * (dr * (2.0 * f - m)).sin() - 0.0006 * (dr * (2.0 * f + mpr)).sin();
    c1 += 0.0010 * (dr * (2.0 * f - mpr)).sin() + 0.0005 * (dr * (2.0 * mpr + m)).sin();
    let delta_t = if t < -11.0 {
        0.001 + 0.000839 * t + 0.0002261 * t2 - 0.00000845 * t3 - 0.000000081 * t * t3
    } else {
        -0.000278 + 0.000265 * t + 0.000262 * t2
    };
    jd1 + c1 - delta_t
}

/// Kính độ Mặt Trời (radian) tại trưa UTC của JD nguyên `jdn`, đã hiệu chỉnh
/// theo timezone `time_zone`. Toán của Ho Ngoc Duc "NEW" `getSunLongitude`:
/// `T = (jdn - 2451545.5 - time_zone/24.0) / 36525.0`. Tất cả shift (cho UTC
/// epoch + múi giờ) đều ở đây — caller chỉ truyền JDN nguyên (= trưa UTC).
///
/// Lưu ý BUG từng gặp: Rust ban đầu double-shift 0.5 (gọi `sun_longitude(jdn -
/// 0.5 - tz/24)` ngoài RỒI trừ `2451545.5` trong) → lệch sl_idx cho môt sốJDN
/// (vd Dec 22 2003 alkal sl=8 thay vì 9 → phá `get_lunar_month_11` → sai Tết
/// 2004). Phiên bản này bất biến để match các port `vanng822/ramlich`,
/// `kunkka19xx/lunar` và prototype Python nội bộ (đã đối chiếu).
#[allow(clippy::excessive_precision)]
pub(crate) fn sun_longitude_at_noon(jdn: i64, time_zone: f64) -> f64 {
    let t = (jdn as f64 - 2451545.5 - time_zone / 24.0) / 36525.0;
    let t2 = t * t;
    let dr = PI / 180.0;
    let m = 357.52910 + 35999.05030 * t - 0.0001559 * t2 - 0.00000048 * t * t2;
    let l0 = 280.46645 + 36000.76983 * t + 0.0003032 * t2;
    let mut dl = (1.914600 - 0.004817 * t - 0.000014 * t2) * (dr * m).sin();
    dl += (0.019993 - 0.000101 * t) * (dr * 2.0 * m).sin() + 0.000290 * (dr * 3.0 * m).sin();
    let mut l = l0 + dl;
    // Nutation-in-longitude correction (omega term):
    let omega = 125.04 - 1934.136 * t;
    l -= 0.00569 + 0.00478 * (omega * dr).sin();
    let l_rad = l * dr;
    // Normalize to [0, 2*pi).
    l_rad - 2.0 * PI * (l_rad / (2.0 * PI)).floor()
}

/// Kính độ Mặt Trời tại 00:00 VN local của JD nguyên `jdn` (Ho's convention:
/// local midnight start-of-VN-day D = `jdn - 0.5 - tz/24`). Trả về độ [0, 360),
/// công thức theo Ho's NEW `getSunLongitude` với hiệu chỉnh nutation (omega
/// term) và epoch 2451545.5.
///
/// Bug từng gặp: phiên bản đầu double-convert (deg→rad→deg) → kết quả luôn ở
/// 0-5° cho mọi ngày trong năm → `find_tiet_khi_jd` không tìm thấy transition.
/// Fixed: chỉ convert 1 lần (dr = π/180), normalize trong deg [0, 360).
#[allow(clippy::excessive_precision)]
pub(crate) fn sun_longitude_deg_at_local_midnight(jdn: i64, time_zone: f64) -> f64 {
    let real_jd = jdn as f64 - 0.5 - time_zone / 24.0;
    let t = (real_jd - 2451545.5) / 36525.0;
    let t2 = t * t;
    let dr = PI / 180.0;
    let m = 357.52910 + 35999.05030 * t - 0.0001559 * t2 - 0.00000048 * t * t2;
    let l0 = 280.46645 + 36000.76983 * t + 0.0003032 * t2;
    let mut dl = (1.914600 - 0.004817 * t - 0.000014 * t2) * (dr * m).sin();
    dl += (0.019993 - 0.000101 * t) * (dr * 2.0 * m).sin() + 0.000290 * (dr * 3.0 * m).sin();
    let mut l_deg = l0 + dl;
    let omega = 125.04 - 1934.136 * t;
    l_deg -= 0.00569 + 0.00478 * (omega * dr).sin();
    (l_deg % 360.0 + 360.0) % 360.0
}


/// Chỉ số 0..11 của "tháng thái dương" (khoảng 30° kinh độ Mặt Trời) tại trưa
/// VN của ngày có JD nguyên `jdn` (= trưa UTC của ngày Dương).
fn get_sun_longitude(jdn: i64, time_zone: f64) -> i64 {
    let l = sun_longitude_at_noon(jdn, time_zone);
    (l / PI * 6.0).floor() as i64
}

/// JD nguyên (trưa UTC) của ngày bắt đầu tháng âm chứa sóc thứ `k` tại timezone
/// `time_zone`.
fn get_new_moon_day(k: i64, time_zone: f64) -> i64 {
    (new_moon(k) + 0.5 + time_zone / 24.0).floor() as i64
}

/// JD của *mùng 1 tháng 11 âm* (tháng chứa Đông Chí — winter solstice) của năm
/// Dương lịch `yy` tại `time_zone`. Nếu sóc rơi vào sau Đông chí (sl ≥ 9,
/// i.e., ≤ 270° đã qua), tháng 11 âm bắt đầu tại sóc trước đó (k-1).
fn get_lunar_month_11(yy: i64, time_zone: f64) -> i64 {
    let off = jd_from_date(31, 12, yy) as f64 - 2415021.0;
    let k = (off / 29.530588853).floor() as i64;
    let nm = get_new_moon_day(k, time_zone);
    if get_sun_longitude(nm, time_zone) >= 9 {
        get_new_moon_day(k - 1, time_zone)
    } else {
        nm
    }
}

/// Offset (theo số lunar months sau tháng 11 âm) của tháng nhuận trong năm
/// nhuận. Phát hiện lần đầu tiên mà sl-index không đổi giữa hai sóc liên tiếp
/// (= tháng không chứa Trung khí = nhuận).
fn get_leap_month_offset(a11: i64, time_zone: f64) -> i64 {
    let k = ((a11 as f64 - NM_EPOCH_JD) / SYNODIC_MONTH + 0.5).floor() as i64;
    let mut i = 1;
    let mut arc = get_sun_longitude(get_new_moon_day(k + i, time_zone), time_zone);
    loop {
        let last = arc;
        i += 1;
        arc = get_sun_longitude(get_new_moon_day(k + i, time_zone), time_zone);
        if arc == last || i >= 14 {
            break;
        }
    }
    i - 1
}

// ===========================================================================
// Public-ish raw converters (only used internally by super)
// ===========================================================================

pub struct LunarRaw {
    pub day: i64,
    pub month: i64,
    pub year: i64,
    pub leap: bool,
}

/// Dương → Âm (thô).
pub fn solar_to_lunar_raw(dd: i64, mm: i64, yy: i64, tz: f64) -> LunarRaw {
    let day_number = jd_from_date(dd, mm, yy);
    solar_to_lunar_raw_jd(day_number, tz)
}

/// Dương (đã có JD) → Âm. Logic của Ho Ngoc Duc / kunkka19xx port.
pub fn solar_to_lunar_raw_jd(day_number: i64, tz: f64) -> LunarRaw {
    let k = ((day_number as f64 - NM_EPOCH_JD) / SYNODIC_MONTH).floor() as i64;
    let mut month_start = get_new_moon_day(k + 1, tz);
    if month_start > day_number {
        month_start = get_new_moon_day(k, tz);
    }
    let (_, _, solar_year) = jd_to_date(day_number);
    let mut a11 = get_lunar_month_11(solar_year, tz);
    let mut b11 = a11;
    let mut lunar_year;
    if a11 >= month_start {
        lunar_year = solar_year;
        a11 = get_lunar_month_11(solar_year - 1, tz);
    } else {
        lunar_year = solar_year + 1;
        b11 = get_lunar_month_11(solar_year + 1, tz);
    }
    let lunar_day = day_number - month_start + 1;
    let diff = ((month_start - a11) as f64 / 29.0).floor() as i64;
    let mut lunar_leap = false;
    let mut lunar_month = diff + 11;
    if b11 - a11 > 365 {
        let leap_offset = get_leap_month_offset(a11, tz);
        if diff >= leap_offset {
            lunar_month = diff + 10;
            if diff == leap_offset {
                lunar_leap = true;
            }
        }
    }
    if lunar_month > 12 {
        lunar_month -= 12;
    }
    if lunar_month >= 11 && diff < 4 {
        lunar_year -= 1;
    }
    LunarRaw {
        day: lunar_day,
        month: lunar_month,
        year: lunar_year,
        leap: lunar_leap,
    }
}

/// Âm → Dương (thô) — trả về JD nguyên của ngày Dương lịch tương ứng.
///
/// Đối chiếu `tradecatlabs/fatecat/.../amlich.rs-master/src/lib.rs` (method
/// `LunarDay::to_julian_days`).
pub fn lunar_to_solar_raw(dd: i64, mm: i64, yy: i64, leap: bool, tz: f64) -> i64 {
    let (a11, b11) = if mm < 11 {
        (
            get_lunar_month_11(yy - 1, tz),
            get_lunar_month_11(yy, tz),
        )
    } else {
        (
            get_lunar_month_11(yy, tz),
            get_lunar_month_11(yy + 1, tz),
        )
    };
    let mut off = mm - 11;
    if off < 0 {
        off += 12;
    }
    if b11 - a11 > 365 {
        let leap_off = get_leap_month_offset(a11, tz);
        let mut leap_month = leap_off - 2;
        if leap_month < 0 {
            leap_month += 12;
        }
        if leap && leap_month != mm {
            // Tháng nhuận không tồn tại — fallback vẫn trả JD của tháng phi-nhuận
            // (caller `lunar_to_solar` sẽ đối chiếu roundtrip và báo lỗi).
        } else if leap || off >= leap_off {
            off += 1;
        }
    }
    let k = (0.5 + (a11 as f64 - NM_EPOCH_JD) / SYNODIC_MONTH).trunc() as i64;
    let month_start = get_new_moon_day(k + off, tz);
    month_start + dd - 1
}