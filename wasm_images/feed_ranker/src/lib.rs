use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct FeedInput {
    posts: Vec<Post>,
    weights: RankWeights,
}

#[derive(Debug, Deserialize)]
struct Post {
    id: String,
    author_reputation: f64,
    like_count: u64,
    reply_count: u64,
    age_hours: f64,
    has_media: bool,
}

#[derive(Debug, Deserialize)]
struct RankWeights {
    reputation: f64,
    likes: f64,
    replies: f64,
    freshness: f64,
    media_bonus: f64,
}

#[derive(Debug, Serialize)]
struct RankedPost {
    id: String,
    score: f64,
    rank: usize,
}

#[derive(Debug, Serialize)]
struct FeedOutput {
    ranked: Vec<RankedPost>,
}

fn rank_post(post: &Post, weights: &RankWeights) -> f64 {
    let rep_score = post.author_reputation * weights.reputation;
    let like_score = (post.like_count as f64).ln_1p() * weights.likes;
    let reply_score = (post.reply_count as f64).ln_1p() * weights.replies;
    let freshness_score = (1.0 / (post.age_hours + 1.0)) * weights.freshness;
    let media_score = if post.has_media { weights.media_bonus } else { 0.0 };
    rep_score + like_score + reply_score + freshness_score + media_score
}

#[no_mangle]
pub extern "C" fn run() -> i32 {
    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input).unwrap_or(0);

    let feed: FeedInput = match serde_json::from_str(&input) {
        Ok(f) => f,
        Err(e) => {
            let err = format!("{{\"error\": \"{e}\"}}");
            println!("{err}");
            return 1;
        }
    };

    let mut scored: Vec<(usize, f64)> = feed
        .posts
        .iter()
        .enumerate()
        .map(|(i, p)| (i, rank_post(p, &feed.weights)))
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let output = FeedOutput {
        ranked: scored
            .into_iter()
            .enumerate()
            .map(|(rank, (idx, score))| RankedPost {
                id: feed.posts[idx].id.clone(),
                score,
                rank: rank + 1,
            })
            .collect(),
    };

    println!("{}", serde_json::to_string(&output).unwrap());
    0
}

#[no_mangle]
pub extern "C" fn _start() {
    std::process::exit(run());
}
