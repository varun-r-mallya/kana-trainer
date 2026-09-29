//! kanatrain: a mistake-driven hiragana / katakana trainer for the terminal.

use std::fs;
use std::sync::atomic::Ordering;

use kanatrain::data::build_cards;
use kanatrain::session::{menu, run_session};
use kanatrain::stats::{load_stats, save_stats, stats_path};
use kanatrain::term::{bold, dim, prompt, COLOR};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("kanatrain: mistake-driven hiragana/katakana practice\n\n  --reset     forget saved statistics\n  --no-color  disable colours (NO_COLOR is also honoured)");
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
    println!("  {}", dim("What you miss comes back more often."));

    while let Some(cfg) = menu(&stats) {
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
