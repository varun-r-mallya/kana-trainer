//! kana-trainer: a mistake-driven hiragana / katakana trainer for the terminal.
//!
//! Learning loop:
//!   * cards you miss get heavier weights and are drawn more often (persisted between runs)
//!   * a missed card comes back a few questions later as a "second chance"
//!   * every miss shows a side-by-side comparison of what you typed vs the answer,
//!     look-alike kana, and (for combos) how the pieces blend
//!   * you must type the right answer once before moving on
//!   * the end-of-session report lists every confusion you made

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// ───────────────────────────── data ─────────────────────────────

type Row = (&'static str, &'static str, &'static [&'static str]);

const BASIC: &[Row] = &[
    ("あ", "ア", &["a"]), ("い", "イ", &["i"]), ("う", "ウ", &["u"]), ("え", "エ", &["e"]), ("お", "オ", &["o"]),
    ("か", "カ", &["ka"]), ("き", "キ", &["ki"]), ("く", "ク", &["ku"]), ("け", "ケ", &["ke"]), ("こ", "コ", &["ko"]),
    ("さ", "サ", &["sa"]), ("し", "シ", &["shi", "si"]), ("す", "ス", &["su"]), ("せ", "セ", &["se"]), ("そ", "ソ", &["so"]),
    ("た", "タ", &["ta"]), ("ち", "チ", &["chi", "ti"]), ("つ", "ツ", &["tsu", "tu"]), ("て", "テ", &["te"]), ("と", "ト", &["to"]),
    ("な", "ナ", &["na"]), ("に", "ニ", &["ni"]), ("ぬ", "ヌ", &["nu"]), ("ね", "ネ", &["ne"]), ("の", "ノ", &["no"]),
    ("は", "ハ", &["ha"]), ("ひ", "ヒ", &["hi"]), ("ふ", "フ", &["fu", "hu"]), ("へ", "ヘ", &["he"]), ("ほ", "ホ", &["ho"]),
    ("ま", "マ", &["ma"]), ("み", "ミ", &["mi"]), ("む", "ム", &["mu"]), ("め", "メ", &["me"]), ("も", "モ", &["mo"]),
    ("や", "ヤ", &["ya"]), ("ゆ", "ユ", &["yu"]), ("よ", "ヨ", &["yo"]),
    ("ら", "ラ", &["ra"]), ("り", "リ", &["ri"]), ("る", "ル", &["ru"]), ("れ", "レ", &["re"]), ("ろ", "ロ", &["ro"]),
    ("わ", "ワ", &["wa"]), ("を", "ヲ", &["wo", "o"]), ("ん", "ン", &["n", "nn"]),
];

const VOICED: &[Row] = &[
    ("が", "ガ", &["ga"]), ("ぎ", "ギ", &["gi"]), ("ぐ", "グ", &["gu"]), ("げ", "ゲ", &["ge"]), ("ご", "ゴ", &["go"]),
    ("ざ", "ザ", &["za"]), ("じ", "ジ", &["ji", "zi"]), ("ず", "ズ", &["zu"]), ("ぜ", "ゼ", &["ze"]), ("ぞ", "ゾ", &["zo"]),
    ("だ", "ダ", &["da"]), ("ぢ", "ヂ", &["di", "ji", "zi"]), ("づ", "ヅ", &["du", "zu"]), ("で", "デ", &["de"]), ("ど", "ド", &["do"]),
    ("ば", "バ", &["ba"]), ("び", "ビ", &["bi"]), ("ぶ", "ブ", &["bu"]), ("べ", "ベ", &["be"]), ("ぼ", "ボ", &["bo"]),
    ("ぱ", "パ", &["pa"]), ("ぴ", "ピ", &["pi"]), ("ぷ", "プ", &["pu"]), ("ぺ", "ペ", &["pe"]), ("ぽ", "ポ", &["po"]),
];

const COMBO: &[Row] = &[
    ("きゃ", "キャ", &["kya"]), ("きゅ", "キュ", &["kyu"]), ("きょ", "キョ", &["kyo"]),
    ("しゃ", "シャ", &["sha", "sya"]), ("しゅ", "シュ", &["shu", "syu"]), ("しょ", "ショ", &["sho", "syo"]),
    ("ちゃ", "チャ", &["cha", "tya"]), ("ちゅ", "チュ", &["chu", "tyu"]), ("ちょ", "チョ", &["cho", "tyo"]),
    ("にゃ", "ニャ", &["nya"]), ("にゅ", "ニュ", &["nyu"]), ("にょ", "ニョ", &["nyo"]),
    ("ひゃ", "ヒャ", &["hya"]), ("ひゅ", "ヒュ", &["hyu"]), ("ひょ", "ヒョ", &["hyo"]),
    ("みゃ", "ミャ", &["mya"]), ("みゅ", "ミュ", &["myu"]), ("みょ", "ミョ", &["myo"]),
    ("りゃ", "リャ", &["rya"]), ("りゅ", "リュ", &["ryu"]), ("りょ", "リョ", &["ryo"]),
    ("ぎゃ", "ギャ", &["gya"]), ("ぎゅ", "ギュ", &["gyu"]), ("ぎょ", "ギョ", &["gyo"]),
    ("じゃ", "ジャ", &["ja", "jya", "zya"]), ("じゅ", "ジュ", &["ju", "jyu", "zyu"]), ("じょ", "ジョ", &["jo", "jyo", "zyo"]),
    ("びゃ", "ビャ", &["bya"]), ("びゅ", "ビュ", &["byu"]), ("びょ", "ビョ", &["byo"]),
    ("ぴゃ", "ピャ", &["pya"]), ("ぴゅ", "ピュ", &["pyu"]), ("ぴょ", "ピョ", &["pyo"]),
];

/// Groups of kana that are commonly mistaken for one another.
const LOOKALIKES: &[&str] = &[
    "さきち", "ぬめ", "ねれわ", "はほ", "るろ", "いり", "こに", "あお", "すむ", "たな", "けは", "まも", "ぱば", "じぢ", "ずづ",
    "シツ", "ソン", "クケタ", "ウワフ", "コユ", "チテ", "ヌス", "マム", "ノメ", "アマ", "ミツ", "ナメ", "パバ", "ジヂ", "ズヅ",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Script {
    Hira,
    Kata,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Set {
    Basic,
    Voiced,
    Combo,
}

#[derive(Clone)]
struct Card {
    kana: &'static str,
    script: Script,
    set: Set,
    answers: &'static [&'static str],
}

impl Card {
    fn primary(&self) -> &'static str {
        self.answers[0]
    }
    fn accepts(&self, typed: &str) -> bool {
        self.answers.contains(&typed)
    }
}

fn build_cards() -> Vec<Card> {
    let mut v = Vec::new();
    for (set, rows) in [(Set::Basic, BASIC), (Set::Voiced, VOICED), (Set::Combo, COMBO)] {
        for &(h, k, a) in rows {
            v.push(Card { kana: h, script: Script::Hira, set, answers: a });
            v.push(Card { kana: k, script: Script::Kata, set, answers: a });
        }
    }
    v
}

// ───────────────────────────── terminal helpers ─────────────────────────────

static COLOR: AtomicBool = AtomicBool::new(true);

fn paint(code: &str, s: &str) -> String {
    if COLOR.load(Ordering::Relaxed) {
        format!("\x1b[{}m{}\x1b[0m", code, s)
    } else {
        s.to_string()
    }
}
fn bold(s: &str) -> String { paint("1", s) }
fn dim(s: &str) -> String { paint("2", s) }
fn red(s: &str) -> String { paint("1;31", s) }
fn green(s: &str) -> String { paint("1;32", s) }
fn yellow(s: &str) -> String { paint("1;33", s) }
fn cyan(s: &str) -> String { paint("1;36", s) }

/// Terminal display width (kana are double-width).
fn dw(s: &str) -> usize {
    s.chars().map(|c| if (c as u32) >= 0x2E80 { 2 } else { 1 }).sum()
}

fn center(plain: &str, colored: &str, w: usize) -> String {
    let total = w.saturating_sub(dw(plain));
    let l = total / 2;
    format!("{}{}{}", " ".repeat(l), colored, " ".repeat(total - l))
}

fn pad_right(plain: &str, colored: &str, w: usize) -> String {
    format!("{}{}", colored, " ".repeat(w.saturating_sub(dw(plain))))
}

fn prompt(msg: &str) -> Option<String> {
    print!("{}", msg);
    io::stdout().flush().ok();
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line.trim().to_lowercase()),
    }
}

// ───────────────────────────── stats (persisted) ─────────────────────────────

#[derive(Clone, Copy)]
struct Stat {
    seen: u32,
    wrong: u32,
    weight: f64,
}

impl Default for Stat {
    fn default() -> Self {
        Stat { seen: 0, wrong: 0, weight: 1.0 }
    }
}

fn stats_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("kana-trainer").join("stats.tsv"))
}

fn load_stats() -> HashMap<String, Stat> {
    let mut m = HashMap::new();
    let Some(text) = stats_path().and_then(|p| fs::read_to_string(p).ok()) else { return m };
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() == 4 {
            if let (Ok(seen), Ok(wrong), Ok(weight)) = (f[1].parse(), f[2].parse(), f[3].parse()) {
                m.insert(f[0].to_string(), Stat { seen, wrong, weight });
            }
        }
    }
    m
}

fn save_stats(m: &HashMap<String, Stat>) {
    let Some(p) = stats_path() else { return };
    if let Some(dir) = p.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let mut out = String::new();
    for (k, s) in m {
        out.push_str(&format!("{}\t{}\t{}\t{:.3}\n", k, s.seen, s.wrong, s.weight));
    }
    let _ = fs::write(p, out);
}

// ───────────────────────────── rng ─────────────────────────────

struct Rng(u64);
impl Rng {
    fn new() -> Self {
        let n = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(88172645463325252);
        Rng(n | 1)
    }
    fn next_f64(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 11) as f64 / (1u64 << 53) as f64
    }
}

// ───────────────────────────── feedback ─────────────────────────────

fn lookup<'a>(cards: &'a [Card], script: Script, typed: &str) -> Option<&'a Card> {
    if typed.is_empty() {
        return None;
    }
    cards.iter().find(|c| c.script == script && c.accepts(typed))
}

fn by_kana<'a>(cards: &'a [Card], kana: &str) -> Option<&'a Card> {
    cards.iter().find(|c| c.kana == kana)
}

/// Positional char diff: mismatching chars are highlighted (left red, right green).
fn diff(a: &str, b: &str) -> (String, String) {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let mut l = String::new();
    let mut r = String::new();
    for i in 0..ac.len().max(bc.len()) {
        let (x, y) = (ac.get(i), bc.get(i));
        if let Some(x) = x {
            l.push_str(&if Some(x) == y { x.to_string() } else { red(&x.to_string()) });
        }
        if let Some(y) = y {
            r.push_str(&if Some(y) == x { y.to_string() } else { green(&y.to_string()) });
        }
    }
    (l, r)
}

const BOX_W: usize = 18;

fn kana_box(title: &str, kana: &str, rom_plain: &str, rom_col: &str, kana_col: &str, border: fn(&str) -> String) -> Vec<String> {
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

fn small_reading(c: char) -> &'static str {
    match c {
        'ゃ' | 'ャ' => "ya",
        'ゅ' | 'ュ' => "yu",
        'ょ' | 'ョ' => "yo",
        _ => "?",
    }
}

fn show_feedback(cards: &[Card], target: &Card, typed: &str) {
    let typed_card = lookup(cards, target.script, typed);
    let gave_up = typed.is_empty();

    // Side-by-side boxes.
    let (left_kana, left_rom_plain, left_rom_col, left_kana_col) = match typed_card {
        Some(tc) => {
            let (l, _) = diff(typed, target.primary());
            (tc.kana.to_string(), typed.to_string(), l, red(tc.kana))
        }
        None if gave_up => ("?".to_string(), "—".to_string(), dim("—"), red("?")),
        None => {
            let (l, _) = diff(typed, target.primary());
            ("?".to_string(), typed.to_string(), l, red("?"))
        }
    };
    let (_, right_rom_col) = diff(typed, target.primary());
    let left = kana_box(
        if gave_up { "YOU SKIPPED" } else { "YOU ANSWERED" },
        &left_kana,
        &left_rom_plain,
        &left_rom_col,
        &left_kana_col,
        red,
    );
    let right = kana_box("CORRECT", target.kana, target.primary(), &right_rom_col, &green(target.kana), green);
    println!();
    for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        println!("   {}   {}   {}", l, if i == 3 { dim(" ≠ ") } else { "   ".to_string() }, r);
    }
    println!();

    // Explanation.
    match typed_card {
        Some(tc) => println!(
            "   {} '{}' is {}, but this card is {} ({}).",
            red("✗"), typed, bold(tc.kana), bold(target.kana), bold(target.primary())
        ),
        None if gave_up => println!("   {} No worries — here is the answer: {} = {}", yellow("↷"), bold(target.kana), bold(target.primary())),
        None => println!(
            "   {} '{}' isn't a reading of any {}. Check the spelling: {} = {}",
            red("✗"), typed,
            if target.script == Script::Hira { "hiragana" } else { "katakana" },
            bold(target.kana), bold(target.primary())
        ),
    }
    if target.answers.len() > 1 {
        println!("     {}", dim(&format!("also accepted: {}", target.answers[1..].join(", "))));
    }

    // Combo breakdown.
    if target.set == Set::Combo {
        let mut it = target.kana.chars();
        let (base, small) = (it.next().unwrap(), it.next().unwrap());
        let base_s = base.to_string();
        let base_rom = by_kana(cards, &base_s).map(|c| c.primary()).unwrap_or("?");
        println!(
            "   {} {} ({}) + small {} ({})  →  {}   {}",
            cyan("⊕"), bold(&base_s), base_rom, bold(&small.to_string()), small_reading(small), bold(target.primary()),
            dim("(one blended beat, not two)")
        );
        let split_guess = format!("{}{}", base_rom, small_reading(small));
        if typed == base_rom || typed == split_guess {
            println!("     {}", yellow("The small ゃ/ゅ/ょ fuses with the kana before it — don't read it as a separate syllable."));
        }
    }

    // Look-alike / sibling strip.
    let mut members: Vec<&str> = Vec::new();
    if target.set == Set::Combo {
        let first: String = target.kana.chars().take(1).collect();
        for c in cards.iter().filter(|c| c.script == target.script && c.set == Set::Combo && c.kana.starts_with(&first)) {
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
        let label = if target.set == Set::Combo { "Family" } else { "Look-alikes" };
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
        println!("{}", line);
    }
    println!();
}

// ───────────────────────────── session ─────────────────────────────

struct Config {
    scripts: Vec<Script>,
    sets: Vec<Set>,
    weak_only: bool,
    total: usize, // 0 = endless
}

fn menu(stats: &HashMap<String, Stat>) -> Option<Config> {
    println!();
    println!("  {}", bold("Which script?"));
    println!("    {} hiragana   {} katakana   {} both", cyan("1"), cyan("2"), cyan("3"));
    let scripts = loop {
        match prompt(&format!("  {} ", dim("[3] ›")))?.as_str() {
            "1" | "h" => break vec![Script::Hira],
            "2" | "k" => break vec![Script::Kata],
            "3" | "b" | "" => break vec![Script::Hira, Script::Kata],
            "q" => return None,
            _ => println!("  {}", yellow("Pick 1, 2 or 3.")),
        }
    };

    println!();
    println!("  {}", bold("Which sets? (combine freely, e.g. \"1 3\")"));
    println!("    {} single kana        あ か さ …", cyan("1"));
    println!("    {} voiced singles     が ざ ぱ …", cyan("2"));
    println!("    {} combined kana      きゃ しゅ ちょ …", cyan("3"));
    let weak_count = stats.values().filter(|s| s.wrong > 0).count();
    println!(
        "    {} weak spots only    {}",
        cyan("w"),
        dim(&format!("({} kana you've missed before, across all sets)", weak_count))
    );
    let (sets, weak_only) = loop {
        let ans = prompt(&format!("  {} ", dim("[all] ›")))?;
        if ans == "q" {
            return None;
        }
        if ans.is_empty() || ans == "all" || ans == "a" {
            break (vec![Set::Basic, Set::Voiced, Set::Combo], false);
        }
        if ans == "w" {
            if weak_count == 0 {
                println!("  {}", yellow("No recorded mistakes yet — play a normal round first."));
                continue;
            }
            break (vec![Set::Basic, Set::Voiced, Set::Combo], true);
        }
        let mut v = Vec::new();
        for t in ans.split(|c: char| !c.is_alphanumeric()).filter(|t| !t.is_empty()) {
            match t {
                "1" => v.push(Set::Basic),
                "2" => v.push(Set::Voiced),
                "3" => v.push(Set::Combo),
                _ => {}
            }
        }
        if v.is_empty() {
            println!("  {}", yellow("Enter some of 1, 2, 3 — or w / all."));
        } else {
            break (v, false);
        }
    };

    println!();
    let total = loop {
        let ans = prompt(&format!("  {} {} ", bold("How many questions?"), dim("[30, 0 = endless] ›")))?;
        if ans.is_empty() {
            break 30;
        }
        if ans == "q" {
            return None;
        }
        if let Ok(n) = ans.parse::<usize>() {
            break n;
        }
        println!("  {}", yellow("Enter a number."));
    };
    Some(Config { scripts, sets, weak_only, total })
}

struct Miss {
    typed: Vec<String>,
}

fn pick(active: &[usize], exclude: &[usize], stats: &HashMap<String, Stat>, cards: &[Card], rng: &mut Rng) -> usize {
    let cand: Vec<usize> = active.iter().copied().filter(|i| !exclude.contains(i)).collect();
    let cand = if cand.is_empty() { active.to_vec() } else { cand };
    let w: Vec<f64> = cand.iter().map(|&i| stats.get(cards[i].kana).copied().unwrap_or_default().weight).collect();
    let mut r = rng.next_f64() * w.iter().sum::<f64>();
    for (k, &wt) in w.iter().enumerate() {
        if r < wt {
            return cand[k];
        }
        r -= wt;
    }
    *cand.last().unwrap()
}

/// Returns true if the user quit early.
fn run_session(cards: &[Card], cfg: &Config, stats: &mut HashMap<String, Stat>) -> bool {
    let active: Vec<usize> = cards
        .iter()
        .enumerate()
        .filter(|(_, c)| cfg.scripts.contains(&c.script) && cfg.sets.contains(&c.set))
        .filter(|(_, c)| !cfg.weak_only || stats.get(c.kana).map_or(false, |s| s.wrong > 0))
        .map(|(i, _)| i)
        .collect();
    if active.is_empty() {
        println!("  {}", yellow("Nothing to practise with that selection."));
        return false;
    }

    let mut rng = Rng::new();
    let started = Instant::now();
    let mut pending: Vec<(usize, usize)> = Vec::new(); // (due prompt#, card idx)
    let mut misses: HashMap<usize, Miss> = HashMap::new();
    let (mut asked, mut correct, mut streak, mut best, mut recovered, mut prompts) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let mut last: Option<usize> = None;
    let mut quit = false;

    println!();
    println!("  {}", dim("Type the romaji and press Enter.   ? = show answer   q = finish"));

    loop {
        let (ci, retry) = if let Some(p) = pending.iter().position(|p| p.0 <= prompts) {
            (pending.remove(p).1, true)
        } else if cfg.total > 0 && asked >= cfg.total {
            if pending.is_empty() {
                break;
            }
            (pending.remove(0).1, true)
        } else {
            let mut ex: Vec<usize> = pending.iter().map(|p| p.1).collect();
            if active.len() > 1 {
                ex.extend(last);
            }
            (pick(&active, &ex, stats, cards, &mut rng), false)
        };
        last = Some(ci);
        prompts += 1;
        let card = &cards[ci];

        let head = if retry {
            yellow("↺ second chance")
        } else if cfg.total > 0 {
            bold(&format!("{}/{}", asked + 1, cfg.total))
        } else {
            bold(&format!("#{}", asked + 1))
        };
        println!();
        println!("  ── {} ─────────────── {}", head, dim(&format!("streak {}", streak)));
        println!();
        println!("        {}", bold(card.kana));
        println!();
        let Some(mut typed) = prompt(&format!("  {} ", cyan("romaji ›"))) else {
            quit = true;
            break;
        };
        if typed == "q" {
            quit = true;
            break;
        }
        if typed == "?" {
            typed.clear();
        }

        let st = stats.entry(card.kana.to_string()).or_default();
        st.seen += 1;
        if !typed.is_empty() && card.accepts(&typed) {
            if retry {
                recovered += 1;
                println!("  {} {}", green("✓ recovered!"), dim("that one is sticking now"));
            } else {
                correct += 1;
                streak += 1;
                best = best.max(streak);
                st.weight = (st.weight * 0.7).max(0.4);
                println!("  {} {}", green("✓"), dim(card.primary()));
            }
        } else {
            st.wrong += 1;
            st.weight = if retry { (st.weight + 0.5).min(25.0) } else { (st.weight * 1.8 + 1.0).min(25.0) };
            streak = 0;
            misses.entry(ci).or_insert(Miss { typed: vec![] }).typed.push(typed.clone());
            show_feedback(cards, card, &typed);
            // Lock it in: retype the right answer (Enter to skip).
            loop {
                match prompt(&format!("  {} ", yellow("type the correct reading to lock it in ›"))) {
                    None => {
                        quit = true;
                        break;
                    }
                    Some(a) if a.is_empty() => break,
                    Some(a) if card.accepts(&a) => {
                        println!("  {}", green("✓ locked in — you'll see this one again soon."));
                        break;
                    }
                    Some(_) => println!("  {} {}", red("✗"), dim(&format!("it's \"{}\"", card.primary()))),
                }
            }
            pending.push((prompts + 3, ci));
        }
        if !retry {
            asked += 1;
        }
        save_stats(stats);
        if quit {
            break;
        }
    }

    // ── report ──
    println!();
    println!("  {}", bold("═══════════════ Session report ═══════════════"));
    let pct = if asked > 0 { correct * 100 / asked } else { 0 };
    let secs = started.elapsed().as_secs();
    println!(
        "  Score: {}   Best streak: {}   Recovered: {}   Time: {}m{:02}s",
        bold(&format!("{}/{} ({}%)", correct, asked, pct)), best, recovered, secs / 60, secs % 60
    );
    if misses.is_empty() {
        if asked > 0 {
            println!("  {}", green("Flawless — no mistakes this session!"));
        }
    } else {
        println!();
        println!("  {}", bold("Your mistakes  (target  ←  what you said)"));
        let mut list: Vec<_> = misses.iter().collect();
        list.sort_by(|a, b| b.1.typed.len().cmp(&a.1.typed.len()).then(a.0.cmp(b.0)));
        for (&ci, m) in list {
            let c = &cards[ci];
            let plain = format!("{} {}", c.kana, c.primary());
            let mut said: Vec<String> = Vec::new();
            for t in &m.typed {
                let s = match lookup(cards, c.script, t) {
                    Some(tc) => format!("'{}' = {}", t, red(tc.kana)),
                    None if t.is_empty() => dim("(skipped)"),
                    None => format!("'{}'", red(t)),
                };
                said.push(s);
            }
            println!("    {}  ←  {}   {}", pad_right(&plain, &green(&plain), 12), said.join(", "), dim(&format!("×{}", m.typed.len())));
        }
    }
    let mut weak: Vec<(&String, &Stat)> = stats.iter().filter(|(_, s)| s.wrong > 0 && s.weight > 1.2).collect();
    weak.sort_by(|a, b| b.1.weight.partial_cmp(&a.1.weight).unwrap());
    if !weak.is_empty() {
        println!();
        print!("  {} ", bold("Weakest overall:"));
        for (k, s) in weak.iter().take(8) {
            let rom = by_kana(cards, k).map(|c| c.primary()).unwrap_or("?");
            print!("{} {} {}   ", k, rom, dim(&format!("({}/{} missed)", s.wrong, s.seen)));
        }
        println!();
        println!("  {}", dim("Choose \"w\" at the set prompt to drill only these."));
    }
    println!();
    quit
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("kana-trainer — mistake-driven hiragana/katakana practice\n\n  --reset     forget saved statistics\n  --no-color  disable colours (NO_COLOR is also honoured)");
        return;
    }
    if std::env::var_os("NO_COLOR").is_some() || args.iter().any(|a| a == "--no-color") {
        COLOR.store(false, Ordering::Relaxed);
    }
    if args.iter().any(|a| a == "--reset") {
        if let Some(p) = stats_path() {
            let _ = fs::remove_file(p);
        }
        println!("Statistics cleared.");
        return;
    }

    let cards = build_cards();
    let mut stats = load_stats();

    println!();
    println!("  {}", bold("かな  KANA TRAINER"));
    println!("  {}", dim("Mistakes are the lesson: what you miss comes back more often."));

    loop {
        let Some(cfg) = menu(&stats) else { break };
        if run_session(&cards, &cfg, &mut stats) {
            break;
        }
        match prompt(&format!("  {} ", bold("Another round? [Y/n] ›"))) {
            Some(a) if a == "n" || a == "q" => break,
            None => break,
            _ => {}
        }
    }
    save_stats(&stats);
    println!("\n  がんばって！ (ganbatte!)\n");
}
