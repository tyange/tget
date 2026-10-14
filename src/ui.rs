use std::io::{self, Write};

use anyhow::Result;

use crate::hit::Hit;

/// 번호를 입력받는다. 취소하면 None, 고르면 0부터 시작하는 인덱스.
pub fn prompt_choice(max: usize) -> Result<Option<usize>> {
    loop {
        print!("선택 (1-{max}, 엔터=취소)> ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let line = line.trim();

        if line.is_empty() || line.eq_ignore_ascii_case("q") {
            return Ok(None);
        }
        match line.parse::<usize>() {
            Ok(n) if (1..=max).contains(&n) => return Ok(Some(n - 1)),
            _ => println!("1-{max} 사이 숫자를 입력하세요"),
        }
    }
}

pub fn print_hits(hits: &[Hit]) {
    for (i, h) in hits.iter().enumerate() {
        println!(
            "{:>2}) {}\n     {}  seeders {}  [{}]",
            i + 1,
            h.title,
            human_size(h.size),
            h.seeders(),
            h.indexer
        );
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    format!("{v:.1}{}", UNITS[u])
}
