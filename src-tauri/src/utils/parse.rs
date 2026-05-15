pub fn parse_uptime(raw: &str) -> String {
    let s = raw
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;
    let (d, h, m, sec) = (s / 86400, (s % 86400) / 3600, (s % 3600) / 60, s % 60);
    if d > 0 {
        format!("{}d {}h {}m {}s", d, h, m, sec)
    } else if h > 0 {
        format!("{}h {}m {}s", h, m, sec)
    } else {
        format!("{}m {}s", m, sec)
    }
}

pub fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub fn parse_df(out: &str) -> (u64, u64) {
    for line in out.lines().skip(1) {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() >= 4 {
            let t = parse_df_num(p[1]);
            let u = parse_df_num(p[2]);
            if t > 0 {
                return (t, u);
            }
        }
    }
    (0, 0)
}

fn parse_df_num(s: &str) -> u64 {
    if s.ends_with('G') {
        (s[..s.len() - 1].parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0) as u64
    } else if s.ends_with('M') {
        (s[..s.len() - 1].parse::<f64>().unwrap_or(0.0) * 1024.0) as u64
    } else if s.ends_with('K') {
        s[..s.len() - 1].parse::<f64>().unwrap_or(0.0) as u64
    } else {
        s.parse::<u64>().unwrap_or(0)
    }
}

pub fn parse_meminfo(out: &str) -> (u64, u64) {
    let (mut total, mut avail) = (0u64, 0u64);
    for line in out.lines() {
        if line.starts_with("MemTotal:") {
            total = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
        if line.starts_with("MemAvailable:") {
            avail = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
    }
    (total, avail)
}

pub fn parse_battery_field(out: &str, field: &str) -> Option<i64> {
    out.lines()
        .find(|l| l.trim().starts_with(field))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|v| v.trim().parse().ok())
}
