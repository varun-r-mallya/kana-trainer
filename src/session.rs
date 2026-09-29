//! Interactive menu and the quiz loop.

use std::collections::HashMap;
use std::time::Instant;

use crate::data::{Card, Script, Set};
use crate::feedback::{by_kana, lookup, show_feedback};
use crate::rng::Rng;
use crate::stats::{save_stats, Stat};
use crate::term::{bold, cyan, dim, green, pad_right, prompt, red, yellow};

pub struct Config {
    pub scripts: Vec<Script>,
    pub sets: Vec<Set>,
    pub weak_only: bool,
    pub total: usize, // 0 = endless
}

pub fn menu(stats: &HashMap<String, Stat>) -> Option<Config> {
    println!();
    println!("  {}", bold("Which script?"));
    println!(
        "    {} hiragana   {} katakana   {} both",
        cyan("1"),
        cyan("2"),
        cyan("3")
    );
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
        dim(&format!(
            "({} kana you've missed before, across all sets)",
            weak_count
        ))
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
                println!(
                    "  {}",
                    yellow("No recorded mistakes yet. Play a normal round first.")
                );
                continue;
            }
            break (vec![Set::Basic, Set::Voiced, Set::Combo], true);
        }
        let v = parse_sets(&ans);
        if v.is_empty() {
            println!("  {}", yellow("Enter some of 1, 2, 3, or w / all."));
        } else {
            break (v, false);
        }
    };

    println!();
    let total = loop {
        let ans = prompt(&format!(
            "  {} {} ",
            bold("How many questions?"),
            dim("[30, 0 = endless] ›")
        ))?;
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
    Some(Config {
        scripts,
        sets,
        weak_only,
        total,
    })
}

pub struct Miss {
    typed: Vec<String>,
}

/// Sets named by digits in `input` ("1 3", "2,3"); unknown tokens are ignored.
pub fn parse_sets(input: &str) -> Vec<Set> {
    let mut v = Vec::new();
    for t in input
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
    {
        let set = match t {
            "1" => Set::Basic,
            "2" => Set::Voiced,
            "3" => Set::Combo,
            _ => continue,
        };
        if !v.contains(&set) {
            v.push(set);
        }
    }
    v
}

/// Indices of the cards a session with `cfg` draws from.
pub fn select_cards(cards: &[Card], cfg: &Config, stats: &HashMap<String, Stat>) -> Vec<usize> {
    cards
        .iter()
        .enumerate()
        .filter(|(_, c)| cfg.scripts.contains(&c.script) && cfg.sets.contains(&c.set))
        .filter(|(_, c)| !cfg.weak_only || stats.get(c.kana).is_some_and(|s| s.wrong > 0))
        .map(|(i, _)| i)
        .collect()
}

pub fn pick(
    active: &[usize],
    exclude: &[usize],
    stats: &HashMap<String, Stat>,
    cards: &[Card],
    rng: &mut Rng,
) -> usize {
    let cand: Vec<usize> = active
        .iter()
        .copied()
        .filter(|i| !exclude.contains(i))
        .collect();
    let cand = if cand.is_empty() {
        active.to_vec()
    } else {
        cand
    };
    let w: Vec<f64> = cand
        .iter()
        .map(|&i| stats.get(cards[i].kana).copied().unwrap_or_default().weight)
        .collect();
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
pub fn run_session(cards: &[Card], cfg: &Config, stats: &mut HashMap<String, Stat>) -> bool {
    let active = select_cards(cards, cfg, stats);
    if active.is_empty() {
        println!("  {}", yellow("Nothing to practise with that selection."));
        return false;
    }

    let mut rng = Rng::new();
    let started = Instant::now();
    let mut pending: Vec<(usize, usize)> = Vec::new(); // (due prompt#, card idx)
    let mut misses: HashMap<usize, Miss> = HashMap::new();
    let (mut asked, mut correct, mut streak, mut best, mut recovered, mut prompts) =
        (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let mut last: Option<usize> = None;
    let mut quit = false;

    println!();
    println!(
        "  {}",
        dim("Type the romaji and press Enter.   ? = show answer   q = finish")
    );

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
        println!(
            "  ── {} ─────────────── {}",
            head,
            dim(&format!("streak {}", streak))
        );
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
        if !typed.is_empty() && card.accepts(&typed) {
            if retry {
                recovered += 1;
                st.record_recovered();
                println!(
                    "  {} {}",
                    green("✓ recovered!"),
                    dim("that one is sticking now")
                );
            } else {
                correct += 1;
                streak += 1;
                best = best.max(streak);
                st.record_correct();
                println!("  {} {}", green("✓"), dim(card.primary()));
            }
        } else {
            st.record_wrong(retry);
            streak = 0;
            misses
                .entry(ci)
                .or_insert(Miss { typed: vec![] })
                .typed
                .push(typed.clone());
            show_feedback(cards, card, &typed);
            // Lock it in: retype the right answer (Enter to skip).
            loop {
                match prompt(&format!(
                    "  {} ",
                    yellow("type the correct reading to lock it in ›")
                )) {
                    None => {
                        quit = true;
                        break;
                    }
                    Some(a) if a.is_empty() => break,
                    Some(a) if card.accepts(&a) => {
                        println!(
                            "  {}",
                            green("✓ locked in. You'll see this one again soon.")
                        );
                        break;
                    }
                    Some(_) => println!(
                        "  {} {}",
                        red("✗"),
                        dim(&format!("it's \"{}\"", card.primary()))
                    ),
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
    println!(
        "  {}",
        bold("═══════════════ Session report ═══════════════")
    );
    let pct = (correct * 100).checked_div(asked).unwrap_or(0);
    let secs = started.elapsed().as_secs();
    println!(
        "  Score: {}   Best streak: {}   Recovered: {}   Time: {}m{:02}s",
        bold(&format!("{}/{} ({}%)", correct, asked, pct)),
        best,
        recovered,
        secs / 60,
        secs % 60
    );
    if misses.is_empty() {
        if asked > 0 {
            println!("  {}", green("Flawless, no mistakes this session!"));
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
            println!(
                "    {}  ←  {}   {}",
                pad_right(&plain, &green(&plain), 12),
                said.join(", "),
                dim(&format!("×{}", m.typed.len()))
            );
        }
    }
    let mut weak: Vec<(&String, &Stat)> = stats
        .iter()
        .filter(|(_, s)| s.wrong > 0 && s.weight > 1.2)
        .collect();
    weak.sort_by(|a, b| b.1.weight.partial_cmp(&a.1.weight).unwrap());
    if !weak.is_empty() {
        println!();
        print!("  {} ", bold("Weakest overall:"));
        for (k, s) in weak.iter().take(8) {
            let rom = by_kana(cards, k).map(|c| c.primary()).unwrap_or("?");
            print!(
                "{} {} {}   ",
                k,
                rom,
                dim(&format!("({}/{} missed)", s.wrong, s.seen))
            );
        }
        println!();
        println!(
            "  {}",
            dim("Choose \"w\" at the set prompt to drill only these.")
        );
    }
    println!();
    quit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::build_cards;

    fn cfg(scripts: Vec<Script>, sets: Vec<Set>, weak_only: bool) -> Config {
        Config {
            scripts,
            sets,
            weak_only,
            total: 10,
        }
    }

    #[test]
    fn parse_sets_handles_separators_and_junk() {
        assert_eq!(parse_sets("1 3"), vec![Set::Basic, Set::Combo]);
        assert_eq!(parse_sets("2,3"), vec![Set::Voiced, Set::Combo]);
        assert_eq!(parse_sets("3 3 1"), vec![Set::Combo, Set::Basic]);
        assert!(parse_sets("x 9 hello").is_empty());
        assert!(parse_sets("").is_empty());
    }

    #[test]
    fn select_cards_filters_by_script_and_set() {
        let cards = build_cards();
        let stats = HashMap::new();
        let idx = select_cards(
            &cards,
            &cfg(vec![Script::Hira], vec![Set::Basic], false),
            &stats,
        );
        assert_eq!(idx.len(), 46);
        assert!(idx
            .iter()
            .all(|&i| cards[i].script == Script::Hira && cards[i].set == Set::Basic));

        let idx = select_cards(
            &cards,
            &cfg(vec![Script::Kata], vec![Set::Combo], false),
            &stats,
        );
        assert_eq!(idx.len(), 33);

        let all = select_cards(
            &cards,
            &cfg(
                vec![Script::Hira, Script::Kata],
                vec![Set::Basic, Set::Voiced, Set::Combo],
                false,
            ),
            &stats,
        );
        assert_eq!(all.len(), cards.len());
    }

    #[test]
    fn weak_only_keeps_only_missed_kana() {
        let cards = build_cards();
        let mut stats = HashMap::new();
        stats.insert(
            "し".to_string(),
            Stat {
                seen: 3,
                wrong: 2,
                weight: 3.0,
            },
        );
        stats.insert(
            "あ".to_string(),
            Stat {
                seen: 3,
                wrong: 0,
                weight: 0.5,
            },
        );
        let c = cfg(
            vec![Script::Hira, Script::Kata],
            vec![Set::Basic, Set::Voiced, Set::Combo],
            true,
        );
        let idx = select_cards(&cards, &c, &stats);
        assert_eq!(idx.len(), 1);
        assert_eq!(cards[idx[0]].kana, "し");
    }

    #[test]
    fn pick_respects_exclusions() {
        let cards = build_cards();
        let active: Vec<usize> = (0..5).collect();
        let mut rng = Rng::seeded(1);
        let stats = HashMap::new();
        for _ in 0..200 {
            let i = pick(&active, &[0, 1, 2], &stats, &cards, &mut rng);
            assert!(i == 3 || i == 4);
        }
    }

    #[test]
    fn pick_falls_back_when_everything_excluded() {
        let cards = build_cards();
        let active = vec![0usize, 1];
        let mut rng = Rng::seeded(1);
        let i = pick(&active, &[0, 1], &HashMap::new(), &cards, &mut rng);
        assert!(active.contains(&i));
    }

    #[test]
    fn pick_favours_heavy_cards() {
        let cards = build_cards();
        let active = vec![0usize, 1]; // あ, ア
        let mut stats = HashMap::new();
        stats.insert(
            cards[0].kana.to_string(),
            Stat {
                seen: 9,
                wrong: 9,
                weight: 20.0,
            },
        );
        stats.insert(
            cards[1].kana.to_string(),
            Stat {
                seen: 9,
                wrong: 0,
                weight: 0.4,
            },
        );
        let mut rng = Rng::seeded(99);
        let heavy = (0..2000)
            .filter(|_| pick(&active, &[], &stats, &cards, &mut rng) == 0)
            .count();
        assert!(heavy > 1800, "heavy card drawn only {heavy}/2000");
    }
}
