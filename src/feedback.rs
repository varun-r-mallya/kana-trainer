//! Answer lookup and the "you got it wrong" comparison panel.

use crate::data::{Card, Script, Set, LOOKALIKES};
use crate::term::{bold, center, cyan, dim, green, pad_right, red, yellow};

pub fn lookup<'a>(cards: &'a [Card], script: Script, typed: &str) -> Option<&'a Card> {
    if typed.is_empty() {
        return None;
    }
    cards
        .iter()
        .find(|c| c.script == script && c.accepts(typed))
}

pub fn by_kana<'a>(cards: &'a [Card], kana: &str) -> Option<&'a Card> {
    cards.iter().find(|c| c.kana == kana)
}

/// Positional char diff: mismatching chars are highlighted (left red, right green).
pub fn diff(a: &str, b: &str) -> (String, String) {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let mut l = String::new();
    let mut r = String::new();
    for i in 0..ac.len().max(bc.len()) {
        let (x, y) = (ac.get(i), bc.get(i));
        if let Some(x) = x {
            l.push_str(&if Some(x) == y {
                x.to_string()
            } else {
                red(&x.to_string())
            });
        }
        if let Some(y) = y {
            r.push_str(&if Some(y) == x {
                y.to_string()
            } else {
                green(&y.to_string())
            });
        }
    }
    (l, r)
}

pub const BOX_W: usize = 18;

pub fn kana_box(
    title: &str,
    kana: &str,
    rom_plain: &str,
    rom_col: &str,
    kana_col: &str,
    border: fn(&str) -> String,
) -> Vec<String> {
    let row = |inner: String| format!("{}{}{}", border("│"), inner, border("│"));
    vec![
        border(&format!("┌{}┐", "─".repeat(BOX_W))),
        row(center(title, &dim(title), BOX_W)),
        row(" ".repeat(BOX_W)),
        row(center(kana, kana_col, BOX_W)),
        row(" ".repeat(BOX_W)),
        row(center(rom_plain, rom_col, BOX_W)),
        border(&format!("└{}┘", "─".repeat(BOX_W))),
    ]
}

pub fn small_reading(c: char) -> &'static str {
    match c {
        'ゃ' | 'ャ' => "ya",
        'ゅ' | 'ュ' => "yu",
        'ょ' | 'ョ' => "yo",
        _ => "?",
    }
}

/// Builds the full "you got it wrong" panel as text.
pub fn render_feedback(cards: &[Card], target: &Card, typed: &str) -> String {
    let mut out = String::new();
    macro_rules! outln {
        () => { out.push('\n') };
        ($($a:tt)*) => {{ out.push_str(&format!($($a)*)); out.push('\n'); }};
    }
    let typed_card = lookup(cards, target.script, typed);
    let gave_up = typed.is_empty();

    // Side-by-side boxes.
    let (left_kana, left_rom_plain, left_rom_col, left_kana_col) = match typed_card {
        Some(tc) => {
            let (l, _) = diff(typed, target.primary());
            (tc.kana.to_string(), typed.to_string(), l, red(tc.kana))
        }
        None if gave_up => ("?".to_string(), "-".to_string(), dim("-"), red("?")),
        None => {
            let (l, _) = diff(typed, target.primary());
            ("?".to_string(), typed.to_string(), l, red("?"))
        }
    };
    let (_, right_rom_col) = diff(typed, target.primary());
    let left = kana_box(
        if gave_up {
            "YOU SKIPPED"
        } else {
            "YOU ANSWERED"
        },
        &left_kana,
        &left_rom_plain,
        &left_rom_col,
        &left_kana_col,
        red,
    );
    let right = kana_box(
        "CORRECT",
        target.kana,
        target.primary(),
        &right_rom_col,
        &green(target.kana),
        green,
    );
    outln!();
    for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        outln!(
            "   {}   {}   {}",
            l,
            if i == 3 {
                dim(" ≠ ")
            } else {
                "   ".to_string()
            },
            r
        );
    }
    outln!();

    // Explanation.
    match typed_card {
        Some(tc) => outln!(
            "   {} '{}' is {}, but this card is {} ({}).",
            red("✗"),
            typed,
            bold(tc.kana),
            bold(target.kana),
            bold(target.primary())
        ),
        None if gave_up => outln!(
            "   {} No worries, here is the answer: {} = {}",
            yellow("↷"),
            bold(target.kana),
            bold(target.primary())
        ),
        None => outln!(
            "   {} '{}' isn't a reading of any {}. Check the spelling: {} = {}",
            red("✗"),
            typed,
            if target.script == Script::Hira {
                "hiragana"
            } else {
                "katakana"
            },
            bold(target.kana),
            bold(target.primary())
        ),
    }
    if target.answers.len() > 1 {
        outln!(
            "     {}",
            dim(&format!(
                "also accepted: {}",
                target.answers[1..].join(", ")
            ))
        );
    }

    // Combo breakdown.
    if target.set == Set::Combo {
        let mut it = target.kana.chars();
        let (base, small) = (it.next().unwrap(), it.next().unwrap());
        let base_s = base.to_string();
        let base_rom = by_kana(cards, &base_s).map(|c| c.primary()).unwrap_or("?");
        outln!(
            "   {} {} ({}) + small {} ({})  →  {}   {}",
            cyan("⊕"),
            bold(&base_s),
            base_rom,
            bold(&small.to_string()),
            small_reading(small),
            bold(target.primary()),
            dim("(one blended beat, not two)")
        );
        let split_guess = format!("{}{}", base_rom, small_reading(small));
        if typed == base_rom || typed == split_guess {
            outln!("     {}", yellow("The small ゃ/ゅ/ょ fuses with the kana before it. Don't read it as a separate syllable."));
        }
    }

    // Look-alike / sibling strip.
    let mut members: Vec<&str> = Vec::new();
    if target.set == Set::Combo {
        let first: String = target.kana.chars().take(1).collect();
        for c in cards.iter().filter(|c| {
            c.script == target.script && c.set == Set::Combo && c.kana.starts_with(&first)
        }) {
            members.push(c.kana);
        }
        if let Some(b) = by_kana(cards, &first) {
            members.insert(0, b.kana);
        }
    } else {
        for g in LOOKALIKES {
            if g.contains(target.kana) {
                for ch in g.char_indices() {
                    let s = &g[ch.0..ch.0 + ch.1.len_utf8()];
                    if !members.contains(&s) {
                        members.push(s);
                    }
                }
            }
        }
    }
    if members.len() > 1 {
        let label = if target.set == Set::Combo {
            "Family"
        } else {
            "Look-alikes"
        };
        let mut line = format!("   {} {:<13}", cyan("≈"), label);
        for m in members {
            let rom = by_kana(cards, m).map(|c| c.primary()).unwrap_or("?");
            let plain = format!("{} {}", m, rom);
            let col = if m == target.kana {
                green(&plain)
            } else if typed_card.map(|t| t.kana) == Some(m) {
                red(&plain)
            } else {
                plain.clone()
            };
            line.push_str(&pad_right(&plain, &col, 10));
            line.push_str("  ");
        }
        outln!("{}", line);
    }
    outln!();
    out
}

pub fn show_feedback(cards: &[Card], target: &Card, typed: &str) {
    print!("{}", render_feedback(cards, target, typed));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::build_cards;
    use crate::term::{dw, COLOR};
    use std::sync::atomic::Ordering;

    fn plain() {
        COLOR.store(false, Ordering::Relaxed);
    }

    fn card<'a>(cards: &'a [Card], kana: &str) -> &'a Card {
        by_kana(cards, kana).unwrap()
    }

    #[test]
    fn lookup_is_script_aware() {
        let cards = build_cards();
        assert_eq!(lookup(&cards, Script::Hira, "sa").unwrap().kana, "さ");
        assert_eq!(lookup(&cards, Script::Kata, "sa").unwrap().kana, "サ");
        assert!(lookup(&cards, Script::Hira, "").is_none());
        assert!(lookup(&cards, Script::Hira, "xyz").is_none());
    }

    #[test]
    fn diff_of_equal_strings_is_unchanged() {
        plain();
        assert_eq!(diff("shi", "shi"), ("shi".to_string(), "shi".to_string()));
        assert_eq!(diff("sa", "sa"), ("sa".into(), "sa".into()));
    }

    #[test]
    fn diff_handles_different_lengths() {
        plain();
        assert_eq!(diff("s", "shi"), ("s".into(), "shi".into()));
        assert_eq!(diff("", "ka"), ("".into(), "ka".into()));
    }

    #[test]
    fn small_readings() {
        assert_eq!(small_reading('ゃ'), "ya");
        assert_eq!(small_reading('ュ'), "yu");
        assert_eq!(small_reading('ょ'), "yo");
        assert_eq!(small_reading('x'), "?");
    }

    #[test]
    fn boxes_have_equal_display_width() {
        plain();
        let b = kana_box("CORRECT", "きゃ", "kya", "kya", "きゃ", green);
        assert_eq!(b.len(), 7);
        let w = dw(&b[0]);
        assert!(b.iter().all(|l| dw(l) == w), "{b:#?}");
    }

    #[test]
    fn wrong_reading_shows_what_you_actually_named() {
        plain();
        let cards = build_cards();
        let out = render_feedback(&cards, card(&cards, "ち"), "sa");
        assert!(out.contains("YOU ANSWERED") && out.contains("CORRECT"));
        assert!(out.contains("'sa' is さ, but this card is ち (chi)"));
    }

    #[test]
    fn lookalikes_are_listed() {
        plain();
        let cards = build_cards();
        let out = render_feedback(&cards, card(&cards, "シ"), "tsu");
        assert!(out.contains("Look-alikes"));
        assert!(out.contains("シ shi") && out.contains("ツ tsu"));
    }

    #[test]
    fn combo_gets_breakdown_and_family() {
        plain();
        let cards = build_cards();
        let out = render_feedback(&cards, card(&cards, "きゃ"), "ki");
        assert!(out.contains("き (ki) + small ゃ (ya)"));
        assert!(out.contains("Family"));
        assert!(out.contains("きゅ kyu") && out.contains("きょ kyo"));
        assert!(out.contains("fuses with the kana before it"));
    }

    #[test]
    fn split_reading_of_combo_triggers_hint() {
        plain();
        let cards = build_cards();
        let out = render_feedback(&cards, card(&cards, "しゅ"), "shiyu");
        assert!(out.contains("fuses with the kana before it"));
        assert!(out.contains("isn't a reading"));
    }

    #[test]
    fn skipping_reveals_answer_without_blame() {
        plain();
        let cards = build_cards();
        let out = render_feedback(&cards, card(&cards, "ぬ"), "");
        assert!(out.contains("YOU SKIPPED"));
        assert!(out.contains("here is the answer"));
        assert!(!out.contains("isn't a reading"));
    }

    #[test]
    fn alternates_are_mentioned() {
        plain();
        let cards = build_cards();
        let out = render_feedback(&cards, card(&cards, "し"), "sa");
        assert!(out.contains("also accepted: si"));
    }

    #[test]
    fn no_em_dashes_in_output() {
        plain();
        let cards = build_cards();
        for c in &cards {
            for typed in ["", "zzz", "a"] {
                assert!(!render_feedback(&cards, c, typed).contains('\u{2014}'));
            }
        }
    }
}
