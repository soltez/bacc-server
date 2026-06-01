use std::sync::Arc;
use std::time::Duration;

use axum::{Json, Router, extract::State, routing::get};
use bacc::{BaccaratScoreboard, BaccaratShoe};
use serde::Serialize;
use tokio::sync::RwLock;

const NUM_DECKS: usize = 8;
const PASSES: u8 = 1;
const PENETRATION: f32 = 0.75;
const ROUND_INTERVAL_SECS: u64 = 30;

struct AppState {
    shoe: BaccaratShoe,
    scoreboard: BaccaratScoreboard,
}

impl AppState {
    fn new() -> Self {
        Self {
            shoe: BaccaratShoe::new(NUM_DECKS, PASSES, PENETRATION),
            scoreboard: BaccaratScoreboard::new(),
        }
    }

    /// Plays one round. If the shoe is exhausted, resets it and the scoreboard,
    /// then plays the first round of the new shoe.
    fn play_round(&mut self) {
        match self.shoe.next() {
            Some(round) => self.scoreboard.update(&round),
            None => {
                self.shoe = BaccaratShoe::new(NUM_DECKS, PASSES, PENETRATION);
                self.scoreboard.clear();
                if let Some(round) = self.shoe.next() {
                    self.scoreboard.update(&round);
                }
            }
        }
    }
}

#[derive(Serialize)]
struct ScoreboardResponse {
    bead_plate: String,
    big_road: String,
    derived_roads: [String; 3],
}

async fn get_scoreboard(State(state): State<Arc<RwLock<AppState>>>) -> Json<ScoreboardResponse> {
    let state = state.read().await;
    let bead_plate = state.scoreboard.bead_plate().to_str_radix(16);
    let big_road = state.scoreboard.big_road().to_str_radix(16);
    let derived_roads = state.scoreboard.derived_roads().map(|r| r.to_str_radix(16));
    Json(ScoreboardResponse {
        bead_plate,
        big_road,
        derived_roads,
    })
}

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(AppState::new()));

    let bg_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(ROUND_INTERVAL_SECS));
        loop {
            interval.tick().await;
            bg_state.write().await.play_round();
        }
    });

    let app = Router::new()
        .route("/scoreboard", get(get_scoreboard))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server error");
}
