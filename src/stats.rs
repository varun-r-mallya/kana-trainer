//! Per-kana statistics, persisted as tab-separated text.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const MIN_WEIGHT: f64 = 0.4;
pub const MAX_WEIGHT: f64 = 25.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stat {
    pub seen: u32,
    pub wrong: u32,
    pub weight: f64,
}

impl Default for Stat {
    fn default() -> Self {
        Stat {
            seen: 0,
            wrong: 0,
            weight: 1.0,
        }
    }
}

impl Stat {
    /// First-try correct answer: the kana becomes rarer.
    pub fn record_correct(&mut self) {
        self.seen += 1;
        self.weight = (self.weight * 0.7).max(MIN_WEIGHT);
    }

    /// Wrong answer. A miss on a second-chance retry raises the weight less.
    pub fn record_wrong(&mut self, retry: bool) {
        self.seen += 1;
        self.wrong += 1;
        self.weight = if retry {
            self.weight + 0.5
        } else {
            self.weight * 1.8 + 1.0
        }
        .min(MAX_WEIGHT);
    }

    /// Correct answer on a second-chance retry: counted as seen, weight untouched.
    pub fn record_recovered(&mut self) {
        self.seen += 1;
    }
}

pub fn stats_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("kanatrain").join("stats.tsv"))
}

pub fn parse_stats(text: &str) -> HashMap<String, Stat> {
    let mut m = HashMap::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() == 4 {
            if let (Ok(seen), Ok(wrong), Ok(weight)) = (f[1].parse(), f[2].parse(), f[3].parse()) {
                m.insert(
                    f[0].to_string(),
                    Stat {
                        seen,
                        wrong,
                        weight,
                    },
                );
            }
        }
    }
    m
}

pub fn serialize_stats(m: &HashMap<String, Stat>) -> String {
    let mut keys: Vec<&String> = m.keys().collect();
    keys.sort();
    let mut out = String::new();
    for k in keys {
        let s = &m[k];
        out.push_str(&format!(
            "{}\t{}\t{}\t{:.3}\n",
            k, s.seen, s.wrong, s.weight
        ));
    }
    out
}

pub fn load_stats() -> HashMap<String, Stat> {
    stats_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .map(|t| parse_stats(&t))
        .unwrap_or_default()
}

pub fn save_stats(m: &HashMap<String, Stat>) {
    let Some(p) = stats_path() else { return };
    if let Some(dir) = p.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(p, serialize_stats(m));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_lowers_weight_with_floor() {
        let mut s = Stat::default();
        for _ in 0..50 {
            s.record_correct();
        }
        assert_eq!(s.weight, MIN_WEIGHT);
        assert_eq!(s.seen, 50);
        assert_eq!(s.wrong, 0);
    }

    #[test]
    fn wrong_raises_weight_with_cap() {
        let mut s = Stat::default();
        s.record_wrong(false);
        assert!((s.weight - 2.8).abs() < 1e-9);
        for _ in 0..50 {
            s.record_wrong(false);
        }
        assert_eq!(s.weight, MAX_WEIGHT);
        assert_eq!(s.wrong, 51);
    }

    #[test]
    fn retry_miss_is_gentler_than_first_miss() {
        let (mut a, mut b) = (Stat::default(), Stat::default());
        a.record_wrong(false);
        b.record_wrong(true);
        assert!(b.weight < a.weight);
    }

    #[test]
    fn recovered_keeps_weight() {
        let mut s = Stat::default();
        s.record_wrong(false);
        let w = s.weight;
        s.record_recovered();
        assert_eq!(s.weight, w);
        assert_eq!(s.seen, 2);
    }

    #[test]
    fn roundtrip() {
        let mut m = HashMap::new();
        m.insert(
            "し".to_string(),
            Stat {
                seen: 7,
                wrong: 3,
                weight: 4.25,
            },
        );
        m.insert("キャ".to_string(), Stat::default());
        assert_eq!(parse_stats(&serialize_stats(&m)), m);
    }

    #[test]
    fn parse_skips_malformed_lines() {
        let m = parse_stats("あ\t1\t0\t1.0\nbroken line\nい\tx\t0\t1.0\nう\t1\t1\t2.5\n");
        assert_eq!(m.len(), 2);
        assert!(m.contains_key("あ") && m.contains_key("う"));
    }

    #[test]
    fn serialization_is_deterministic() {
        let mut m = HashMap::new();
        for k in ["あ", "い", "う", "え"] {
            m.insert(k.to_string(), Stat::default());
        }
        assert_eq!(serialize_stats(&m), serialize_stats(&m.clone()));
    }
}
