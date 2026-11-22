//! Chinese screenplay profile regression tests.
//!
//! These guard the layout model independently of PDF font provisioning.

use scriptcraft_doc::{Element, ElementFormats, Para, Paper, Pos, ScreenplayProfile, Script};

use super::{LineKind, PageMetrics, layout, wrap_chinese};

fn chinese_script(paras: Vec<Para>) -> Script {
    let mut script = Script::from_paras(paras).expect("valid screenplay");
    script.settings.screenplay_profile = ScreenplayProfile::MainlandChina;
    script.settings.elements = ElementFormats::for_profile(ScreenplayProfile::MainlandChina);
    script.settings.page.paper = Paper::A4;
    script.settings.more_contd = false;
    script.settings.auto_contd = false;
    script.normalize().expect("normalized screenplay");
    script
}

#[test]
fn chinese_a4_pagination_uses_forty_eight_body_rows() {
    let s = chinese_script(vec![Para::new(Element::Action, "雨声渐大。".repeat(260))]);
    let l = layout(&s);
    assert_eq!(l.profile, ScreenplayProfile::MainlandChina);
    assert_eq!(l.paper, Paper::A4);
    assert!(l.script_pages() > 1);
    let metrics = PageMetrics::for_profile(l.profile);
    for page in &l.pages {
        for line in page.lines.iter().filter(|line| line.kind == LineKind::Body) {
            assert!(line.y >= metrics.top_margin_pt);
            assert!(line.y < l.paper.height_pt() - metrics.bottom_margin_pt);
        }
    }
}

#[test]
fn chinese_line_breaks_preserve_utf8_offsets_and_caret_positions() {
    let source = "林晚抬头，看见门口有两个人。雨停了，街道上只剩下车灯。".repeat(5);
    let s = chinese_script(vec![Para::new(Element::Action, source.clone())]);
    let l = layout(&s);
    let lines: Vec<_> = l.body().map(|(_, line)| line).collect();
    assert!(lines.len() > 1);
    for line in lines {
        assert!(source.is_char_boundary(line.start));
        assert!(source.is_char_boundary(line.end));
        assert_eq!(source.get(line.start..line.end), Some(line.text.as_str()));
        assert!(!line.text.starts_with(['，', '。', '、', '：', '；', '！', '？']));
    }
    let start = l.caret(Pos::new(0, 0)).expect("start");
    let next = l.caret(Pos::new(0, "林".len())).expect("after first Han character");
    assert!((next.x - start.x - 12.0).abs() < 0.1);
    let hit = l.hit(start.page, next.x, next.y).expect("hit test");
    assert_eq!(hit, Pos::new(0, "林".len()));
}

#[test]
fn chinese_dialogue_has_no_automatic_hollywood_continuation_labels() {
    let s = chinese_script(vec![
        Para::new(Element::SceneHeading, "1. 内景 夜 咖啡馆"),
        Para::new(Element::Character, "林晚"),
        Para::new(Element::Dialogue, "你确定他在这里吗？".repeat(350)),
    ]);
    let l = layout(&s);
    assert!(l.script_pages() > 1);
    assert!(l.pages.iter().all(|p| p.lines.iter().all(|line| !matches!(line.kind, LineKind::More | LineKind::Contd))));
}

#[test]
fn cjk_wrapping_never_returns_mid_character_byte_offsets() {
    let src = "【第一场】内景—夜，咖啡馆。林晚说：“你终于来了！”";
    let spans = wrap_chinese(src, 16);
    assert!(!spans.is_empty());
    for (start, end) in spans {
        assert!(src.is_char_boundary(start));
        assert!(src.is_char_boundary(end));
    }
}

#[test]
fn hollywood_layout_keeps_twelve_point_original_metrics() {
    let script = Script::from_paras(vec![Para::new(Element::Action, "Hello world.")]).expect("script");
    let layout = layout(&script);
    assert_eq!(layout.profile, ScreenplayProfile::Hollywood);
    assert_eq!(layout.paper, Paper::Letter);
    assert_eq!(PageMetrics::for_profile(layout.profile).line_height_pt, 12.0);
}
