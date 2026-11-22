//! Chinese mainland screenplay text metrics and line breaking.
//!
//! The Chinese profile uses a proportional CJK face: full-width Han characters and punctuation
//! occupy two half-width cells; Latin letters and digits occupy one. This is deliberately separate
//! from the fixed Courier metrics used by the Hollywood profile.

use serde::{Deserialize, Serialize};

/// The document's screenplay layout profile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScreenplayProfile {
    /// US Letter, Courier, ten characters per inch and six lines per inch.
    #[default]
    Hollywood,
    /// Mainland China film/TV screenplay conventions on A4 paper.
    MainlandChina,
}

impl ScreenplayProfile {
    pub const ALL: [Self; 2] = [Self::Hollywood, Self::MainlandChina];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Hollywood => "hollywood",
            Self::MainlandChina => "mainlandChina",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Hollywood => "Hollywood",
            Self::MainlandChina => "Mainland China film/TV",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().replace([' ', '-', '_'], "").as_str() {
            "hollywood" | "us" | "finaldraft" => Some(Self::Hollywood),
            "mainlandchina" | "china" | "chinese" | "cn" => Some(Self::MainlandChina),
            _ => None,
        }
    }

    pub const fn is_default(&self) -> bool {
        matches!(self, Self::Hollywood)
    }
}

/// Physical text metrics for one screenplay profile. Coordinates are PDF points from the
/// page's top-left; `cell_width_pt` is the advance of one half-width cell.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageMetrics {
    pub font_size_pt: f32,
    pub line_height_pt: f32,
    pub cell_width_pt: f32,
    pub baseline_offset_pt: f32,
    pub top_margin_pt: f32,
    pub bottom_margin_pt: f32,
    pub left_margin_pt: f32,
    pub right_margin_pt: f32,
    /// Whether full-width Unicode characters consume two half-width cells.
    pub wide_chars: bool,
}

impl PageMetrics {
    /// The reviewed first-pass defaults. Per-production submission templates may override them.
    pub const fn for_profile(profile: ScreenplayProfile) -> Self {
        match profile {
            ScreenplayProfile::Hollywood => Self {
                font_size_pt: 12.0,
                line_height_pt: 12.0,
                cell_width_pt: 7.2,
                baseline_offset_pt: 9.5,
                top_margin_pt: 72.0,
                bottom_margin_pt: 72.0,
                left_margin_pt: 108.0,
                right_margin_pt: 72.0,
                wide_chars: false,
            },
            ScreenplayProfile::MainlandChina => Self {
                font_size_pt: 12.0,
                line_height_pt: 14.4,
                cell_width_pt: 6.0,
                baseline_offset_pt: 11.2,
                top_margin_pt: 72.0,
                bottom_margin_pt: 72.0,
                left_margin_pt: 72.0,
                right_margin_pt: 72.0,
                wide_chars: true,
            },
        }
    }

    /// Half-width cells occupied by text under these metrics.
    pub fn cells(self, text: &str) -> usize {
        if self.wide_chars { display_cells(text) } else { text.chars().count() }
    }

    /// Available body rows for a page of the supplied height.
    pub fn body_rows(self, page_height_pt: f32) -> usize {
        let available = page_height_pt - self.top_margin_pt - self.bottom_margin_pt;
        let rows = (available / self.line_height_pt).floor();
        if rows.is_finite() && rows >= 1.0 { (rows as usize).min(10_000) } else { 1 }
    }
}

/// Whether a character is a zero-advance combining/format character for screenplay line measure.
fn zero_width(c: char) -> bool {
    matches!(
        c as u32,
        0x0300..=0x036F
            | 0x0483..=0x0489
            | 0x0591..=0x05BD
            | 0x05BF
            | 0x05C1..=0x05C2
            | 0x0610..=0x061A
            | 0x064B..=0x065F
            | 0x0670
            | 0x06D6..=0x06ED
            | 0x1AB0..=0x1AFF
            | 0x1DC0..=0x1DFF
            | 0x200B..=0x200F
            | 0x202A..=0x202E
            | 0x2060..=0x206F
            | 0x20D0..=0x20FF
            | 0xFE00..=0xFE0F
            | 0xFE20..=0xFE2F
            | 0xE0100..=0xE01EF
            | 0x1F3FB..=0x1F3FF
    )
}

/// Whether a character normally occupies one full-em CJK cell.
fn full_width(c: char) -> bool {
    matches!(
        c as u32,
        0x1100..=0x11FF
            | 0x2E80..=0x2FFF
            | 0x3000..=0x303F
            | 0x3040..=0x30FF
            | 0x3100..=0x312F
            | 0x3130..=0x318F
            | 0x31A0..=0x31BF
            | 0x31C0..=0x31EF
            | 0x3200..=0x32FF
            | 0x3300..=0x33FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xA000..=0xA4CF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE1F
            | 0xFE30..=0xFE6F
            | 0xFF01..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F000..=0x1FAFF
            | 0x20000..=0x3FFFD
    )
}

/// The display width in half-width cells. Combining marks and formatting characters add no width.
pub fn display_cells(text: &str) -> usize {
    text.chars().map(|c| if zero_width(c) { 0 } else if full_width(c) { 2 } else { 1 }).sum()
}

#[derive(Clone, Copy)]
struct Unit {
    start: usize,
    end: usize,
    cells: usize,
    first: char,
    joined: bool,
}

fn units(text: &str) -> Vec<Unit> {
    let mut out: Vec<Unit> = Vec::new();
    for (start, c) in text.char_indices() {
        let end = start + c.len_utf8();
        if zero_width(c) {
            if let Some(last) = out.last_mut() {
                last.end = end;
                if c == '\u{200D}' {
                    last.joined = true;
                }
            } else {
                out.push(Unit { start, end, cells: 0, first: c, joined: c == '\u{200D}' });
            }
        } else if let Some(last) = out.last_mut().filter(|last| last.joined) {
            last.end = end;
            last.cells = last.cells.max(if full_width(c) { 2 } else { 1 });
            last.joined = false;
        } else {
            out.push(Unit { start, end, cells: if full_width(c) { 2 } else { 1 }, first: c, joined: false });
        }
    }
    out
}

fn opening_punctuation(c: char) -> bool {
    matches!(c, '(' | '[' | '{' | '（' | '［' | '｛' | '《' | '〈' | '「' | '『' | '【' | '〔' | '〖' | '〘' | '〚' | '“' | '‘')
}

fn closing_punctuation(c: char) -> bool {
    matches!(
        c,
        ')' | ']' | '}' | '，' | '。' | '、' | '；' | '：' | '？' | '！' | '）' | '］' | '｝' | '》' | '〉' | '」' | '』'
            | '】' | '〕' | '〗' | '〙' | '〛' | '”' | '’' | '…' | '％' | '‰' | '℃'
    )
}

/// Wrap Chinese screenplay text by visual cells, returning UTF-8 byte ranges for each line.
///
/// Lines never start with common Chinese closing punctuation and, where possible, never end
/// with an opening bracket/quote. CJK characters and full-width punctuation occupy two cells;
/// ASCII letters, digits and spaces occupy one. Spaces at a wrap boundary are omitted.
pub fn wrap_chinese(text: &str, max_cells: usize) -> Vec<(usize, usize)> {
    if text.is_empty() {
        return vec![(0, 0)];
    }
    let items = units(text);
    if items.is_empty() {
        return vec![(0, text.len())];
    }

    let width = max_cells.max(1);
    let mut out = Vec::new();
    let mut start = 0;

    while start < items.len() {
        while start < items.len() && items.get(start).is_some_and(|u| u.first.is_whitespace()) {
            start += 1;
        }
        if start >= items.len() {
            break;
        }

        let mut end = start;
        let mut used = 0usize;
        let mut last_space = None;
        while let Some(unit) = items.get(end) {
            if used.saturating_add(unit.cells) > width && end > start {
                break;
            }
            used = used.saturating_add(unit.cells);
            if unit.first.is_whitespace() {
                last_space = Some(end);
            }
            end += 1;
            if used >= width {
                break;
            }
        }

        if end < items.len() {
            if let Some(space) = last_space.filter(|space| *space > start) {
                end = space;
            }
            while end > start && items.get(end - 1).is_some_and(|u| u.first.is_whitespace()) {
                end -= 1;
            }
            if end > start && items.get(end - 1).is_some_and(|u| opening_punctuation(u.first)) {
                end -= 1;
            }
            if end == start {
                end = (start + 1).min(items.len());
            }
            while end < items.len() && items.get(end).is_some_and(|u| closing_punctuation(u.first)) {
                end += 1;
            }
        }

        if end <= start {
            end = (start + 1).min(items.len());
        }
        let byte_start = items.get(start).map_or(0, |u| u.start);
        let byte_end = items.get(end - 1).map_or(text.len(), |u| u.end);
        out.push((byte_start, byte_end));
        start = end;
    }

    if out.is_empty() {
        vec![(0, 0)]
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_have_independent_physical_metrics() {
        let us = PageMetrics::for_profile(ScreenplayProfile::Hollywood);
        let cn = PageMetrics::for_profile(ScreenplayProfile::MainlandChina);
        assert_eq!(us.font_size_pt, 12.0);
        assert_eq!(us.line_height_pt, 12.0);
        assert_eq!(us.cell_width_pt, 7.2);
        assert_eq!(cn.font_size_pt, 12.0);
        assert_eq!(cn.line_height_pt, 14.4);
        assert_eq!(cn.cell_width_pt, 6.0);
        assert_eq!(cn.body_rows(841.89), 48);
        assert_eq!(us.body_rows(792.0), 54);
        assert!(cn.wide_chars);
        assert!(!us.wide_chars);
    }

    #[test]
    fn cjk_display_width_counts_fullwidth_and_halfwidth_cells() {
        assert_eq!(display_cells("中文剧本"), 8);
        assert_eq!(display_cells("A1中文"), 6);
        assert_eq!(display_cells("「你好，世界。」"), 14);
        assert_eq!(display_cells("e\u{0301}"), 1);
    }

    #[test]
    fn wraps_chinese_text_on_utf8_boundaries() {
        let text = "第一场 内景 夜 咖啡馆";
        let lines = wrap_chinese(text, 8);
        assert!(lines.len() > 1);
        for (start, end) in &lines {
            assert!(text.is_char_boundary(*start));
            assert!(text.is_char_boundary(*end));
        }
        assert_eq!(lines.first().and_then(|(a, b)| text.get(*a..*b)), Some("第一场"));
    }

    #[test]
    fn kinsoku_keeps_closing_punctuation_off_the_next_line() {
        let text = "甲乙丙丁，戊己庚辛";
        let lines = wrap_chinese(text, 8);
        let rendered: Vec<&str> = lines.iter().filter_map(|(a, b)| text.get(*a..*b)).collect();
        assert!(rendered.iter().all(|line| !line.starts_with(['，', '。', '、', '；', '：', '？', '！'])));
    }

    #[test]
    fn opening_brackets_do_not_end_a_line_when_a_break_can_be_moved() {
        let text = "甲乙丙丁（戊己庚辛）";
        let lines = wrap_chinese(text, 8);
        let rendered: Vec<&str> = lines.iter().filter_map(|(a, b)| text.get(*a..*b)).collect();
        assert!(rendered.iter().all(|line| !line.ends_with(['（', '《', '「', '『', '【'])));
    }

    #[test]
    fn long_ascii_words_and_overwide_characters_still_make_progress() {
        let lines = wrap_chinese("screenplay 中文", 1);
        assert!(lines.len() > 1);
        assert_eq!(display_cells("中"), 2);
        assert_eq!(wrap_chinese("", 10), vec![(0, 0)]);
    }
}
