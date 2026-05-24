use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ModerationInput {
    content: String,
    author_reputation: f64,
    strictness: f64,
}

#[derive(Debug, Serialize)]
struct ModerationOutput {
    approved: bool,
    flags: Vec<String>,
    confidence: f64,
}

const BANNED_WORDS: &[&str] = &[
    "spam", "scam", "phish", "malware",
];

const SPAM_PATTERNS: &[&str] = &[
    "http://", "free money", "click here", "act now",
    "limited time", "congratulations you won",
];

fn check_content(content: &str) -> Vec<String> {
    let lower = content.to_lowercase();
    let mut flags = Vec::new();

    for word in BANNED_WORDS {
        if lower.contains(word) {
            flags.push(format!("banned_word:{word}"));
        }
    }

    for pattern in SPAM_PATTERNS {
        if lower.contains(pattern) {
            flags.push(format!("spam_pattern:{pattern}"));
        }
    }

    if content.len() > 2000 {
        flags.push("too_long".into());
    }

    if content.len() < 2 {
        flags.push("too_short".into());
    }

    flags
}

#[no_mangle]
pub extern "C" fn run() -> i32 {
    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input).unwrap_or(0);

    let moderation: ModerationInput = match serde_json::from_str(&input) {
        Ok(m) => m,
        Err(e) => {
            let err = format!("{{\"error\": \"{e}\"}}");
            println!("{err}");
            return 1;
        }
    };

    let flags = check_content(&moderation.content);
    let severity = flags.len() as f64;
    let confidence = (1.0 - (severity * 0.2).min(0.95)) * moderation.strictness.min(1.0);

    let output = ModerationOutput {
        approved: flags.is_empty() && moderation.author_reputation > 0.0,
        flags,
        confidence,
    };

    println!("{}", serde_json::to_string(&output).unwrap());
    0
}

#[no_mangle]
pub extern "C" fn _start() {
    std::process::exit(run());
}
