use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use bacc::{BaccaratRound, BaccaratScoreboard, BaccaratShoe};
use serde::Serialize;
use tokio::sync::RwLock;

const NUM_DECKS: usize = 8;
const PASSES: u8 = 1;
const PENETRATION: f32 = 0.75;

struct AppState {
    shoe: BaccaratShoe,
    scoreboard: BaccaratScoreboard,
    current_round: Option<RoundResponse>,
}

impl AppState {
    fn new() -> Self {
        Self {
            shoe: BaccaratShoe::new(NUM_DECKS, PASSES, PENETRATION),
            scoreboard: BaccaratScoreboard::new(),
            current_round: None,
        }
    }

    /// Advances the shoe by one round, updates the scoreboard, and stores the
    /// round. Recreates the shoe and clears the scoreboard when exhausted.
    fn advance(&mut self) -> &RoundResponse {
        let round = match self.shoe.next() {
            Some(r) => r,
            None => {
                self.shoe = BaccaratShoe::new(NUM_DECKS, PASSES, PENETRATION);
                self.scoreboard.clear();
                self.shoe.next().expect("fresh shoe yielded no round")
            }
        };
        self.scoreboard.update(&round);
        self.current_round = Some(RoundResponse::from_round(&round));
        self.current_round.as_ref().unwrap()
    }
}

#[derive(Serialize, Clone)]
struct RoundResponse {
    encoded: u32,
    is_forced_third: bool,
    cut_card_index: Option<u8>,
    player_cards: Vec<u32>,
    banker_cards: Vec<u32>,
}

impl RoundResponse {
    fn from_round(round: &BaccaratRound) -> Self {
        Self {
            encoded: round.encode(),
            is_forced_third: round.is_forced_third(),
            cut_card_index: round.cut_card_index(),
            player_cards: round.player_cards().iter().map(|c| *c as u32).collect(),
            banker_cards: round.banker_cards().iter().map(|c| *c as u32).collect(),
        }
    }
}

#[derive(Serialize)]
struct ScoreboardResponse {
    bead_plate: String,
    big_road: String,
    derived_roads: [String; 3],
}

async fn post_round_next(State(state): State<Arc<RwLock<AppState>>>) -> Json<RoundResponse> {
    let mut state = state.write().await;
    let round = state.advance().clone();
    Json(round)
}

async fn get_round(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<RoundResponse>, StatusCode> {
    let state = state.read().await;
    state
        .current_round
        .clone()
        .map(Json)
        .ok_or(StatusCode::NO_CONTENT)
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

    let app = Router::new()
        .route("/round/next", post(post_round_next))
        .route("/round", get(get_round))
        .route("/scoreboard", get(get_scoreboard))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server error");
}
