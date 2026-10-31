//! Auto Sway: efek ayun otomatis untuk EffectCraft (plug-in API v1, WebAssembly).
//!
//! Build:  cargo build --release --target wasm32-unknown-unknown
//! Hasil:  target/wasm32-unknown-unknown/release/auto_sway.wasm
//!
//! Aturan sandbox: modul TIDAK boleh punya import. Jangan pakai crate yang
//! memanggil host (wasm-bindgen, dll). Kode ini murni Rust tanpa dependency.

use std::f64::consts::TAU;

// ---------------------------------------------------------------------------
// Manifest. Urutan "params" = urutan nilai di array f64 yang diterima ec_render.
// slider/angle/checkbox/popup = 1 nilai, point = 2 nilai (sudah dalam piksel buffer).
// Index: 0 clock | 1 use_layer_time | 2 mode | 3 amplitude | 4 center | 5 speed
//        6 phase | 7 wobble | 8 falloff | 9,10 pivot (x,y)
// ---------------------------------------------------------------------------
const NPARAMS: i32 = 11;

const MANIFEST: &str = r#"{
  "api": 1,
  "id": "com.user.auto-sway",
  "name": "Auto Sway",
  "category": "Distort",
  "version": "1.0.0",
  "author": "you",
  "description": "Ayunan otomatis: layer berayun/bergoyang bolak-balik di sekitar titik pivot.",
  "params": [
    {"id": "clock", "name": "Time (s)", "type": "slider", "default": 0, "min": -100000, "max": 100000, "sliderMin": 0, "sliderMax": 10, "decimals": 3},
    {"id": "use_layer_time", "name": "Use Layer Time", "type": "checkbox", "default": false},
    {"id": "mode", "name": "Mode", "type": "popup", "options": ["Swing (rotate)", "Sway (bend)"], "default": 0},
    {"id": "amplitude", "name": "Angle (deg)", "type": "slider", "default": 15, "min": 0, "max": 180, "sliderMax": 90, "decimals": 1},
    {"id": "center", "name": "Center Angle (deg)", "type": "slider", "default": 0, "min": -180, "max": 180, "decimals": 1},
    {"id": "speed", "name": "Speed (cycles/s)", "type": "slider", "default": 0.5, "min": 0, "max": 20, "sliderMax": 4, "decimals": 2},
    {"id": "phase", "name": "Phase (deg)", "type": "slider", "default": 0, "min": 0, "max": 360, "decimals": 0},
    {"id": "wobble", "name": "Wobble", "type": "slider", "default": 0, "min": 0, "max": 100, "decimals": 0},
    {"id": "falloff", "name": "Bend Falloff", "type": "slider", "default": 1.5, "min": 0.1, "max": 5, "decimals": 2},
    {"id": "pivot", "name": "Pivot", "type": "point", "default": [0.5, 0.0]}
  ]
}"#;

// Buffer yang dipakai host (parameter + piksel). u64 supaya 8-byte aligned.
static mut BUF: Vec<u64> = Vec::new();

#[no_mangle]
pub extern "C" fn ec_api_version() -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn ec_manifest_ptr() -> i32 {
    MANIFEST.as_ptr() as usize as i32
}

#[no_mangle]
pub extern "C" fn ec_manifest_len() -> i32 {
    MANIFEST.len() as i32
}

#[no_mangle]
pub extern "C" fn ec_alloc(bytes: i32) -> i32 {
    let words = (bytes.max(0) as usize + 7) / 8 + 1;
    unsafe {
        let buf = &mut *std::ptr::addr_of_mut!(BUF);
        if buf.len() < words {
            *buf = vec![0u64; words];
        }
        buf.as_mut_ptr() as usize as i32
    }
}

/// Bilinear sampling pada piksel premultiplied RGBA f32. Di luar buffer = transparan.
#[inline]
fn sample(src: &[f32], w: usize, h: usize, sx: f64, sy: f64, out: &mut [f32]) {
    let fx = sx - 0.5;
    let fy = sy - 0.5;
    let x0f = fx.floor();
    let y0f = fy.floor();
    let tx = (fx - x0f) as f32;
    let ty = (fy - y0f) as f32;
    let x0 = x0f as i64;
    let y0 = y0f as i64;
    let mut acc = [0f32; 4];
    for (dy, wy) in [(0i64, 1.0 - ty), (1i64, ty)] {
        for (dx, wx) in [(0i64, 1.0 - tx), (1i64, tx)] {
            let xx = x0 + dx;
            let yy = y0 + dy;
            if xx < 0 || yy < 0 || xx >= w as i64 || yy >= h as i64 {
                continue;
            }
            let wgt = wx * wy;
            let i = (yy as usize * w + xx as usize) * 4;
            for c in 0..4 {
                acc[c] += src[i + c] * wgt;
            }
        }
    }
    out.copy_from_slice(&acc);
}

#[no_mangle]
pub extern "C" fn ec_render(
    pixels: i32,
    width: i32,
    height: i32,
    params: i32,
    nparams: i32,
    time: f64,
    _scale: f64,
) -> i32 {
    if width <= 0 || height <= 0 || nparams < NPARAMS {
        return 1;
    }
    let (w, h) = (width as usize, height as usize);
    let p = unsafe { std::slice::from_raw_parts((params as u32) as usize as *const f64, nparams as usize) };
    let px = unsafe { std::slice::from_raw_parts_mut((pixels as u32) as usize as *mut f32, w * h * 4) };

    let clock = p[0];
    let use_layer_time = p[1] >= 0.5;
    let mode = p[2].round() as i32;
    let amp = p[3];
    let center = p[4];
    let speed = p[5];
    let phase = p[6].to_radians();
    let wobble = p[7] / 100.0;
    let falloff = p[8].max(0.1);
    let (pivx, pivy) = (p[9], p[10]);

    // Sumber waktu: parameter "Time (s)" (aman dari bug cache) atau layer time.
    let t = if use_layer_time { time } else { clock };

    // Gelombang: sinus + harmonik ke-2 (wobble) supaya gerakan terasa lebih natural.
    let a = TAU * speed * t + phase;
    let s = (a.sin() + 0.35 * wobble * (2.0 * a + 0.8).sin()) / (1.0 + 0.35 * wobble);
    let theta = (center + amp * s).to_radians();

    if theta.abs() < 1e-9 {
        return 0; // tidak ada perubahan
    }

    let src: Vec<f32> = px.to_vec();

    if mode == 0 {
        // SWING: rotasi seluruh layer di sekitar pivot (positif = searah jarum jam).
        let (sn, cs) = theta.sin_cos();
        for y in 0..h {
            let dy = y as f64 + 0.5 - pivy;
            for x in 0..w {
                let dx = x as f64 + 0.5 - pivx;
                let sx = pivx + dx * cs + dy * sn;
                let sy = pivy - dx * sn + dy * cs;
                let i = (y * w + x) * 4;
                sample(&src, w, h, sx, sy, &mut px[i..i + 4]);
            }
        }
    } else {
        // SWAY: membengkok. Semakin jauh dari pivot (arah vertikal), semakin besar
        // geseran horizontalnya (seperti rumput / pohon / bendera tertiup angin).
        let extent = pivy.max(h as f64 - pivy).max(1.0);
        let k = theta.sin() * extent;
        for y in 0..h {
            let yc = y as f64 + 0.5;
            let dist = (yc - pivy).abs();
            let u = (dist / extent).min(1.0).powf(falloff);
            let dir = if yc < pivy { 1.0 } else { -1.0 };
            let off = dir * k * u;
            for x in 0..w {
                let i = (y * w + x) * 4;
                sample(&src, w, h, x as f64 + 0.5 - off, yc, &mut px[i..i + 4]);
            }
        }
    }
    0
}
