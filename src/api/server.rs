use actix_web::{post, get, web, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use crate::engine::personal::Profile;
use crate::engine::memorable;

#[derive(Serialize, Deserialize)]
pub struct CheckRequest {
    pub profile: Profile,
    pub password: String,
}

#[derive(Serialize)]
pub struct CheckResponse {
    pub found: bool,
    pub position: Option<usize>,
    pub total_candidates: usize,
    pub time_taken_ms: u128,
}

#[post("/api/personal/generate")]
async fn generate_wordlist(profile: web::Json<Profile>) -> impl Responder {
    let candidates = profile.generate();
    // Convert Vec<Vec<u8>> to Vec<String> for JSON response
    let strings: Vec<String> = candidates.into_iter()
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .collect();
    
    HttpResponse::Ok().json(strings)
}

#[post("/api/check-password")]
async fn check_password(req: web::Json<CheckRequest>) -> impl Responder {
    let start = std::time::Instant::now();
    let found = req.profile.check_password(&req.password);
    let duration = start.elapsed();
    
    HttpResponse::Ok().json(CheckResponse {
        found,
        position: None, // Optimization: we don't track position in early exit
        total_candidates: 0, // We don't know total in early exit
        time_taken_ms: duration.as_millis(),
    })
}

#[get("/api/memorable/generate")]
async fn generate_memorable() -> impl Responder {
    let password = memorable::generate_memorable_password();
    HttpResponse::Ok().json(serde_json::json!({ "password": password }))
}

pub async fn run_server(port: u16) -> std::io::Result<()> {
    println!("Starting JIGSAW API server on 0.0.0.0:{}", port);
    
    HttpServer::new(|| {
        let cors = Cors::permissive(); // Allow all for now, can be restricted later
        
        App::new()
            .wrap(cors)
            .service(generate_wordlist)
            .service(check_password)
            .service(generate_memorable)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
