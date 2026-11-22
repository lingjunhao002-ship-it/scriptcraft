//! Courier Badi, a Courier-metric screenplay typeface derived from Courier Prime by Alan Dague-Greene for Quote-Unquote
//! Apps, with Cyrillic by Dmitry Novikov and extensions by the Courier Badi contributors, licensed under the SIL Open Font License 1.1 (see `fonts/OFL.txt` and ATTRIBUTION.md).
//! Spacing glyphs advance approximately 0.6 em, so 12-point text sets at exactly ten characters per inch.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

/// Courier Badi Regular.
pub const REGULAR: &[u8] = include_bytes!("../fonts/CourierBadi-Regular.ttf");
/// Courier Badi Bold.
pub const BOLD: &[u8] = include_bytes!("../fonts/CourierBadi-Bold.ttf");
/// Courier Badi Italic.
pub const ITALIC: &[u8] = include_bytes!("../fonts/CourierBadi-Italic.ttf");
/// Courier Badi Bold Italic.
pub const BOLD_ITALIC: &[u8] = include_bytes!("../fonts/CourierBadi-BoldItalic.ttf");
/// The font's licence text.
pub const LICENSE: &str = include_str!("../fonts/OFL.txt");
/// The family name.
pub const FAMILY: &str = "Courier Badi";

/// Load a redistributable CJK OpenType font supplied alongside the application or by an
/// explicit environment variable. The font is deliberately not copied from Windows system
/// fonts, whose PDF embedding permissions are not necessarily suitable for redistribution.
///
/// For a portable Windows package, place `NotoSansSC-Regular.otf` in the `fonts` directory next
/// to `scriptcraft.exe`. For development, set `SCRIPTCRAFT_CJK_FONT` to an absolute OTF path.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_chinese_regular() -> Option<Vec<u8>> {
    let from_env = std::env::var_os("SCRIPTCRAFT_CJK_FONT").map(std::path::PathBuf::from);
    let packaged = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("fonts").join("NotoSansSC-Regular.otf")));
    from_env.or(packaged).and_then(|path| std::fs::read(path).ok()).filter(|bytes| !bytes.is_empty())
}

/// Browser builds require a separately provisioned embedded font asset.
#[cfg(target_arch = "wasm32")]
pub fn load_chinese_regular() -> Option<Vec<u8>> {
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn russian_alphabet_has_screenplay_metrics_in_every_face() {
        let alphabet = "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя«»—";
        for data in [super::REGULAR, super::BOLD, super::ITALIC, super::BOLD_ITALIC] {
            let face = ttf_parser::Face::parse(data, 0).unwrap();
            for c in alphabet.chars().chain("ABCabc012 ".chars()) {
                let glyph = face.glyph_index(c).unwrap_or_else(|| panic!("missing {c}"));
                let advance = f32::from(face.glyph_hor_advance(glyph).unwrap()) / f32::from(face.units_per_em());
                assert!((advance - 0.6).abs() < 0.001, "{c}: {advance}");
            }
        }
    }

    #[test]
    fn fonts_are_truetype() {
        for f in [super::REGULAR, super::BOLD, super::ITALIC, super::BOLD_ITALIC] {
            assert_eq!(f.get(..4), Some(&[0, 1, 0, 0][..]));
        }
        assert!(super::LICENSE.contains("SIL Open Font License"));
    }
}
