use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use bacc::BaccShoe;
use bacc_core::BaccScoreboard;
use rand::seq::SliceRandom;
use serde::Serialize;
use shoe::{Card, DECK, Shoe};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

const NUM_DECKS: usize = 8;
const PASSES: u8 = 3;
const PENETRATION: f32 = 0.965;

fn new_shoe() -> BaccShoe {
    let mut rng = rand::thread_rng();
    let mut cards: Vec<Card> = (0..NUM_DECKS).flat_map(|_| DECK).collect();
    for _ in 0..PASSES {
        cards.shuffle(&mut rng);
    }
    let total = cards.len();
    let cut_idx = (total as f32 * (1.0 - PENETRATION)) as usize;
    cards.push(Card::Cut);
    cards.swap(cut_idx, total);
    BaccShoe::from(Shoe::from(cards.as_slice()))
}

struct AppState {
    shoe: BaccShoe,
    scoreboard: BaccScoreboard,
    current_round: Option<RoundResponse>,
}

impl AppState {
    fn new() -> Self {
        Self {
            shoe: new_shoe(),
            scoreboard: BaccScoreboard::new(),
            current_round: None,
        }
    }

    /// Advances the shoe by one round, updates the scoreboard, and stores the
    /// round. Recreates the shoe and clears the scoreboard when exhausted.
    fn advance(&mut self) -> &RoundResponse {
        let round = match self.shoe.next() {
            Some(r) => r,
            None => {
                self.shoe = new_shoe();
                self.scoreboard.clear();
                self.shoe.next().expect("fresh shoe yielded no round")
            }
        };
        self.scoreboard.update(&round);
        self.current_round = Some(RoundResponse {
            encoded_hex: round.encode().to_string(),
        });
        self.current_round.as_ref().unwrap()
    }
}

#[derive(Serialize, Clone)]
struct RoundResponse {
    encoded_hex: String,
}

#[derive(Serialize)]
struct ScoreboardResponse {
    encoded_hex: String,
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
    Json(ScoreboardResponse {
        encoded_hex: state.scoreboard.encode().to_string(),
    })
}

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(AppState::new()));

    let app = Router::new()
        .route("/round/next", post(post_round_next))
        .route("/round", get(get_round))
        .route("/scoreboard", get(get_scoreboard))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server error");
}
