use actix_web::{post, get, web, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use crate::engine::personal::Profile;
use crate::engine::memorable::{self, MemorableConfig, MemorableStyle, CaseStyle, Position};

// ═══════════════════════════════════════════════════════════════
// REQUEST / RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Deserialize)]
pub struct CheckRequest {
    pub profile: Profile,
    pub password: String,
}

#[derive(Serialize)]
pub struct CheckResponse {
    pub found: bool,
    pub total_candidates: usize,
    pub time_taken_ms: u128,
}

#[derive(Serialize)]
pub struct GenerateResponse {
    pub candidates: Vec<String>,
    pub total: usize,
    pub time_taken_ms: u128,
}

#[derive(Serialize, Deserialize)]
pub struct MemorableRequest {
    #[serde(default = "default_word_count")]
    pub word_count: usize,
    #[serde(default)]
    pub separator: String,
    #[serde(default = "default_case_style")]
    pub case_style: String,       // "title", "lower", "upper", "random", "alternating"
    #[serde(default = "default_true")]
    pub include_number: bool,
    #[serde(default = "default_end")]
    pub number_position: String,  // "start", "end", "between"
    #[serde(default = "default_num_max")]
    pub number_max: u32,
    #[serde(default = "default_true")]
    pub include_special: bool,
    #[serde(default = "default_end")]
    pub special_position: String,
    #[serde(default = "default_classic")]
    pub style: String,            // "classic", "passphrase", "story", "alliterative"
    #[serde(default = "default_count")]
    pub count: usize,
    #[serde(default = "default_min_len")]
    pub min_length: usize,
    #[serde(default = "default_max_len")]
    pub max_length: usize,
}

fn default_word_count() -> usize { 3 }
fn default_case_style() -> String { "title".to_string() }
fn default_true() -> bool { true }
fn default_end() -> String { "end".to_string() }
fn default_num_max() -> u32 { 99 }
fn default_classic() -> String { "classic".to_string() }
fn default_count() -> usize { 1 }
fn default_min_len() -> usize { 12 }
fn default_max_len() -> usize { 32 }

#[derive(Serialize)]
pub struct MemorableResponse {
    pub passwords: Vec<String>,
    pub count: usize,
    pub config_used: MemorableConfigSummary,
    pub time_taken_ms: u128,
}

#[derive(Serialize)]
pub struct MemorableConfigSummary {
    pub style: String,
    pub word_count: usize,
    pub separator: String,
    pub case_style: String,
    pub include_number: bool,
    pub include_special: bool,
}

// ═══════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════

#[post("/api/personal/generate")]
async fn generate_personal(profile: web::Json<Profile>) -> impl Responder {
    let start = std::time::Instant::now();
    let candidates = profile.generate();
    let strings: Vec<String> = candidates.iter()
        .map(|b| String::from_utf8_lossy(b).to_string())
        .collect();
    let total = strings.len();
    HttpResponse::Ok().json(GenerateResponse {
        candidates: strings,
        total,
        time_taken_ms: start.elapsed().as_millis(),
    })
}

#[post("/api/personal/check")]
async fn check_password(data: web::Json<CheckRequest>) -> impl Responder {
    let start = std::time::Instant::now();
    let found = data.profile.check_password(&data.password);
    let candidates_count = data.profile.generate().len();
    HttpResponse::Ok().json(CheckResponse {
        found,
        total_candidates: candidates_count,
        time_taken_ms: start.elapsed().as_millis(),
    })
}

#[post("/api/memorable/generate")]
async fn generate_memorable(data: web::Json<MemorableRequest>) -> impl Responder {
    let start = std::time::Instant::now();

    let config = MemorableConfig {
        word_count: data.word_count.clamp(2, 8),
        separator: data.separator.clone(),
        case_style: parse_case_style(&data.case_style),
        include_number: data.include_number,
        number_position: parse_position(&data.number_position),
        number_max: data.number_max,
        include_special: data.include_special,
        special_position: parse_position(&data.special_position),
        style: parse_style(&data.style),
        count: data.count.clamp(1, 100),
        min_length: data.min_length,
        max_length: data.max_length,
    };

    let passwords = memorable::generate_batch(&config);

    HttpResponse::Ok().json(MemorableResponse {
        count: passwords.len(),
        passwords,
        config_used: MemorableConfigSummary {
            style: data.style.clone(),
            word_count: config.word_count,
            separator: config.separator.clone(),
            case_style: data.case_style.clone(),
            include_number: config.include_number,
            include_special: config.include_special,
        },
        time_taken_ms: start.elapsed().as_millis(),
    })
}

#[get("/api/memorable")]
async fn generate_memorable_get() -> impl Responder {
    let pw = memorable::generate_memorable_password();
    HttpResponse::Ok().json(serde_json::json!({
        "password": pw,
        "length": pw.len(),
    }))
}

#[get("/api/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "engine": "jigsaw",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[get("/api/info")]
async fn info() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "name": "JIGSAW",
        "description": "Intelligent Password Toolkit",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": [
            {"method": "POST", "path": "/api/personal/generate", "description": "Generate wordlist from profile"},
            {"method": "POST", "path": "/api/personal/check", "description": "Check if password exists"},
            {"method": "POST", "path": "/api/memorable/generate", "description": "Generate memorable passwords with config"},
            {"method": "GET",  "path": "/api/memorable", "description": "Quick memorable password (default settings)"},
            {"method": "GET",  "path": "/api/health", "description": "Health check"},
            {"method": "GET",  "path": "/api/info", "description": "API info and available endpoints"},
        ],
    }))
}

// ═══════════════════════════════════════════════════════════════
// SERVER STARTUP
// ═══════════════════════════════════════════════════════════════

pub async fn run_server(port: u16) -> std::io::Result<()> {
    println!();
    println!("  ╔═══════════════════════════════════════════╗");
    println!("  ║     JIGSAW API Server                      ║");
    println!("  ╚═══════════════════════════════════════════╝");
    println!();
    println!("  Listening on: http://0.0.0.0:{}", port);
    println!("  Endpoints:");
    println!("    POST /api/personal/generate");
    println!("    POST /api/personal/check");
    println!("    POST /api/memorable/generate");
    println!("    GET  /api/memorable");
    println!("    GET  /api/health");
    println!("    GET  /api/info");
    println!();

    HttpServer::new(|| {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(generate_personal)
            .service(check_password)
            .service(generate_memorable)
            .service(generate_memorable_get)
            .service(health)
            .service(info)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

// ═══════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════

fn parse_case_style(s: &str) -> CaseStyle {
    match s.to_lowercase().as_str() {
        "lower" => CaseStyle::Lower,
        "upper" => CaseStyle::Upper,
        "random" => CaseStyle::Random,
        "alternating" => CaseStyle::Alternating,
        _ => CaseStyle::Title,
    }
}

fn parse_position(s: &str) -> Position {
    match s.to_lowercase().as_str() {
        "start" => Position::Start,
        "between" => Position::Between,
        _ => Position::End,
    }
}

fn parse_style(s: &str) -> MemorableStyle {
    match s.to_lowercase().as_str() {
        "passphrase" => MemorableStyle::Passphrase,
        "story" => MemorableStyle::Story,
        "alliterative" => MemorableStyle::Alliterative,
        _ => MemorableStyle::Classic,
    }
}
