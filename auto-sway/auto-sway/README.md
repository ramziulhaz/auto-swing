# Auto Sway (plug-in efek EffectCraft)

## Build
    rustup target add wasm32-unknown-unknown
    cargo build --release --target wasm32-unknown-unknown

Hasil: `target/wasm32-unknown-unknown/release/auto_sway.wasm`

## Pasang
- Di app: Effect > Load Effect Plug-in... > pilih `auto_sway.wasm`
- Atau CLI: `effectcraft-cli exec effect.plugins.load '{"path": "target/wasm32-unknown-unknown/release/auto_sway.wasm"}'`
- Atau taruh `.wasm` di folder `Plug-ins` di sebelah file settings EffectCraft (dimuat saat start).

Efek muncul di kategori Distort sebagai "Auto Sway".

## Supaya bergerak otomatis (penting)
Ada bug yang masih terbuka di EffectCraft (issue #1977): cache layer tidak memperhitungkan
waktu untuk plug-in, jadi efek yang hanya bergantung pada waktu bisa "membeku".
Workaround: pasang expression `time` pada parameter **Time (s)** (parameter pertama).
Setelah bug itu diperbaiki, cukup centang **Use Layer Time**.

## Parameter
| Parameter | Fungsi |
|---|---|
| Time (s) | jam efek; isi dengan expression `time` |
| Use Layer Time | pakai waktu layer langsung (setelah bug cache diperbaiki) |
| Mode | Swing = rotasi seperti bandul; Sway = membengkok seperti rumput/pohon |
| Angle (deg) | sudut ayunan maksimum ke kiri/kanan |
| Center Angle (deg) | sudut tengah/istirahat; ayunan bolak-balik di sekitar nilai ini |
| Speed (cycles/s) | kecepatan: berapa kali bolak-balik per detik |
| Phase (deg) | geser awal gelombang (untuk beberapa layer supaya tidak serempak) |
| Wobble | tambahan gerak sekunder agar lebih natural |
| Bend Falloff | (mode Sway) 1 = bengkok linear, >1 = ujung lebih melengkung |
| Pivot | titik poros; default tengah-atas (cocok untuk bandul/gantungan) |

## Catatan
- Efek hanya bisa memutar piksel di dalam batas layer. Kalau ujung gambar terpotong saat berayun,
  taruh layer di precomp yang lebih besar dari gambarnya.
- Plug-in `.wasm` jalan di desktop dan CLI, tidak di build web.
