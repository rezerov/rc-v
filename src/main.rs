mod api;
mod error;
mod models;
mod state;

#[tokio::main]
async fn main() {
    println!("Job Queue Starting");

    let app = api::routes().with_state(state::AppState::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
