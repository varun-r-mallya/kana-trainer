//! Kana tables and the card model.

pub type Row = (&'static str, &'static str, &'static [&'static str]);

pub const BASIC: &[Row] = &[
    ("あ", "ア", &["a"]),
    ("い", "イ", &["i"]),
    ("う", "ウ", &["u"]),
    ("え", "エ", &["e"]),
    ("お", "オ", &["o"]),
    ("か", "カ", &["ka"]),
    ("き", "キ", &["ki"]),
    ("く", "ク", &["ku"]),
    ("け", "ケ", &["ke"]),
    ("こ", "コ", &["ko"]),
    ("さ", "サ", &["sa"]),
    ("し", "シ", &["shi", "si"]),
    ("す", "ス", &["su"]),
    ("せ", "セ", &["se"]),
    ("そ", "ソ", &["so"]),
    ("た", "タ", &["ta"]),
    ("ち", "チ", &["chi", "ti"]),
    ("つ", "ツ", &["tsu", "tu"]),
    ("て", "テ", &["te"]),
    ("と", "ト", &["to"]),
    ("な", "ナ", &["na"]),
    ("に", "ニ", &["ni"]),
    ("ぬ", "ヌ", &["nu"]),
    ("ね", "ネ", &["ne"]),
    ("の", "ノ", &["no"]),
    ("は", "ハ", &["ha"]),
    ("ひ", "ヒ", &["hi"]),
    ("ふ", "フ", &["fu", "hu"]),
    ("へ", "ヘ", &["he"]),
    ("ほ", "ホ", &["ho"]),
    ("ま", "マ", &["ma"]),
    ("み", "ミ", &["mi"]),
    ("む", "ム", &["mu"]),
    ("め", "メ", &["me"]),
    ("も", "モ", &["mo"]),
    ("や", "ヤ", &["ya"]),
    ("ゆ", "ユ", &["yu"]),
    ("よ", "ヨ", &["yo"]),
    ("ら", "ラ", &["ra"]),
    ("り", "リ", &["ri"]),
    ("る", "ル", &["ru"]),
    ("れ", "レ", &["re"]),
    ("ろ", "ロ", &["ro"]),
    ("わ", "ワ", &["wa"]),
    ("を", "ヲ", &["wo", "o"]),
    ("ん", "ン", &["n", "nn"]),
];

pub const VOICED: &[Row] = &[
    ("が", "ガ", &["ga"]),
    ("ぎ", "ギ", &["gi"]),
    ("ぐ", "グ", &["gu"]),
    ("げ", "ゲ", &["ge"]),
    ("ご", "ゴ", &["go"]),
    ("ざ", "ザ", &["za"]),
    ("じ", "ジ", &["ji", "zi"]),
    ("ず", "ズ", &["zu"]),
    ("ぜ", "ゼ", &["ze"]),
    ("ぞ", "ゾ", &["zo"]),
    ("だ", "ダ", &["da"]),
    ("ぢ", "ヂ", &["di", "ji", "zi"]),
    ("づ", "ヅ", &["du", "zu"]),
    ("で", "デ", &["de"]),
    ("ど", "ド", &["do"]),
    ("ば", "バ", &["ba"]),
    ("び", "ビ", &["bi"]),
    ("ぶ", "ブ", &["bu"]),
    ("べ", "ベ", &["be"]),
    ("ぼ", "ボ", &["bo"]),
    ("ぱ", "パ", &["pa"]),
    ("ぴ", "ピ", &["pi"]),
    ("ぷ", "プ", &["pu"]),
    ("ぺ", "ペ", &["pe"]),
    ("ぽ", "ポ", &["po"]),
];

pub const COMBO: &[Row] = &[
    ("きゃ", "キャ", &["kya"]),
    ("きゅ", "キュ", &["kyu"]),
    ("きょ", "キョ", &["kyo"]),
    ("しゃ", "シャ", &["sha", "sya"]),
    ("しゅ", "シュ", &["shu", "syu"]),
    ("しょ", "ショ", &["sho", "syo"]),
    ("ちゃ", "チャ", &["cha", "tya"]),
    ("ちゅ", "チュ", &["chu", "tyu"]),
    ("ちょ", "チョ", &["cho", "tyo"]),
    ("にゃ", "ニャ", &["nya"]),
    ("にゅ", "ニュ", &["nyu"]),
    ("にょ", "ニョ", &["nyo"]),
    ("ひゃ", "ヒャ", &["hya"]),
    ("ひゅ", "ヒュ", &["hyu"]),
    ("ひょ", "ヒョ", &["hyo"]),
    ("みゃ", "ミャ", &["mya"]),
    ("みゅ", "ミュ", &["myu"]),
    ("みょ", "ミョ", &["myo"]),
    ("りゃ", "リャ", &["rya"]),
    ("りゅ", "リュ", &["ryu"]),
    ("りょ", "リョ", &["ryo"]),
    ("ぎゃ", "ギャ", &["gya"]),
    ("ぎゅ", "ギュ", &["gyu"]),
    ("ぎょ", "ギョ", &["gyo"]),
    ("じゃ", "ジャ", &["ja", "jya", "zya"]),
    ("じゅ", "ジュ", &["ju", "jyu", "zyu"]),
    ("じょ", "ジョ", &["jo", "jyo", "zyo"]),
    ("びゃ", "ビャ", &["bya"]),
    ("びゅ", "ビュ", &["byu"]),
    ("びょ", "ビョ", &["byo"]),
    ("ぴゃ", "ピャ", &["pya"]),
    ("ぴゅ", "ピュ", &["pyu"]),
    ("ぴょ", "ピョ", &["pyo"]),
];

/// Groups of kana that are commonly mistaken for one another.
pub const LOOKALIKES: &[&str] = &[
    "さきち",
    "ぬめ",
    "ねれわ",
    "はほ",
    "るろ",
    "いり",
    "こに",
    "あお",
    "すむ",
    "たな",
    "けは",
    "まも",
    "ぱば",
    "じぢ",
    "ずづ",
    "シツ",
    "ソン",
    "クケタ",
    "ウワフ",
    "コユ",
    "チテ",
    "ヌス",
    "マム",
    "ノメ",
    "アマ",
    "ミツ",
    "ナメ",
    "パバ",
    "ジヂ",
    "ズヅ",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Script {
    Hira,
    Kata,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Set {
    Basic,
    Voiced,
    Combo,
}

#[derive(Clone, Debug)]
pub struct Card {
    pub kana: &'static str,
    pub script: Script,
    pub set: Set,
    pub answers: &'static [&'static str],
}

impl Card {
    pub fn primary(&self) -> &'static str {
        self.answers[0]
    }
    pub fn accepts(&self, typed: &str) -> bool {
        self.answers.contains(&typed)
    }
}

pub fn build_cards() -> Vec<Card> {
    let mut v = Vec::new();
    for (set, rows) in [
        (Set::Basic, BASIC),
        (Set::Voiced, VOICED),
        (Set::Combo, COMBO),
    ] {
        for &(h, k, a) in rows {
            v.push(Card {
                kana: h,
                script: Script::Hira,
                set,
                answers: a,
            });
            v.push(Card {
                kana: k,
                script: Script::Kata,
                set,
                answers: a,
            });
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn card_counts() {
        let cards = build_cards();
        assert_eq!(BASIC.len(), 46);
        assert_eq!(VOICED.len(), 25);
        assert_eq!(COMBO.len(), 33);
        assert_eq!(cards.len(), (46 + 25 + 33) * 2);
    }

    #[test]
    fn every_card_has_answers_and_lowercase_ascii() {
        for c in build_cards() {
            assert!(!c.answers.is_empty(), "{} has no answers", c.kana);
            for a in c.answers {
                assert!(
                    a.chars().all(|ch| ch.is_ascii_lowercase()),
                    "bad answer {a}"
                );
            }
        }
    }

    #[test]
    fn kana_strings_are_unique_per_script() {
        let cards = build_cards();
        let set: HashSet<&str> = cards.iter().map(|c| c.kana).collect();
        assert_eq!(set.len(), cards.len());
    }

    #[test]
    fn scripts_use_the_right_blocks() {
        for c in build_cards() {
            let ok = c.kana.chars().all(|ch| match c.script {
                Script::Hira => ('\u{3041}'..='\u{309F}').contains(&ch),
                Script::Kata => ('\u{30A0}'..='\u{30FF}').contains(&ch),
            });
            assert!(ok, "{} is in the wrong block for {:?}", c.kana, c.script);
        }
    }

    #[test]
    fn combos_are_two_chars_singles_are_one() {
        for c in build_cards() {
            let n = c.kana.chars().count();
            assert_eq!(n, if c.set == Set::Combo { 2 } else { 1 }, "{}", c.kana);
        }
    }

    #[test]
    fn alternate_spellings_accepted() {
        let cards = build_cards();
        let shi = cards.iter().find(|c| c.kana == "し").unwrap();
        assert!(shi.accepts("shi") && shi.accepts("si") && !shi.accepts("chi"));
        let wo = cards.iter().find(|c| c.kana == "を").unwrap();
        assert!(wo.accepts("wo") && wo.accepts("o"));
        assert_eq!(wo.primary(), "wo");
    }

    #[test]
    fn lookalike_members_all_exist_as_cards() {
        let cards = build_cards();
        for g in LOOKALIKES {
            for ch in g.chars() {
                let s = ch.to_string();
                assert!(cards.iter().any(|c| c.kana == s), "{s} missing");
            }
        }
    }
}
