use tokio_postgres::NoTls;
use tokio::net::TcpListener;
use tokio::time;

use serde::Serialize;

use std::{thread, time::Duration};
use sha2::{Sha256, Digest};
use chrono::Utc;
use watchdog::EventCluster;

use std::{env, panic};

use log::{info, warn};
use std::sync::{Arc, Mutex};
use include_directory::{include_directory, Dir};

use axum::{
    routing::get,
    http::StatusCode,
    Json, Router,
    extract::State,
    response::{Html, IntoResponse, AppendHeaders},
    extract::Path

};

use tokio_util::io::ReaderStream;
use reqwest::header;

fn score_to_rating(score: u32) -> String {
    if score > 2000 {
        return format!("AA")                    // World altering event
    }

    let thresholds = [
        (1000, "AB".to_string()),               // Critical news
        (500, "BB".to_string()),
        (200, "BC".to_string()),                // Lives at immediate danger
        (100, "CC".to_string()),
        (80, "CD".to_string()),
        (60, "DD".to_string()),
        (40, "DE".to_string()),                 // Interesting story
        (20, "EE".to_string()),
        (10, "EF".to_string()),
        (0, "FF".to_string()),                  // Updates / Opinions
        
    ];
    thresholds
        .iter()
        .find(|&&(threshold, _)| score >= threshold)
        .map(|(_, rating)| rating)
        .unwrap_or(&"ER".to_string()).into()
}

#[derive(Clone, Debug, Serialize, Hash)]
struct CleanedCluster {
    headline: String,
    href: String,
    score: u32,
    rating: String,
    num_sources:usize
}

#[derive(Clone, Debug, Serialize)]
struct Report {
    do_not_abuse: String,
    id: String,
    published: u64,
    clusters: Vec<CleanedCluster>,
    avg_rating: String
}

impl CleanedCluster {
    pub fn new(from: &EventCluster) -> Self {
        CleanedCluster {
            headline : from.sources[0].headline.clone(),
            href: from.sources[0].href.clone(),
            score: from.score as u32,
            rating: score_to_rating(from.score as u32),
            num_sources: from.sources.len()

        }
    }
}

fn hash_headlines(strings: Vec<String>) -> String {
    let combined = strings.clone().join("");
    let mut hasher = Sha256::new();
    hasher.update(combined);
    let result = hasher.finalize();
    format!("{:X}", result)
}

impl Report {
    pub fn new(from: &Vec<EventCluster>) -> Self {
        
        let cleaned_clusters:Vec<CleanedCluster> = from.into_iter().map(|x| CleanedCluster::new(x)).collect();
        let avg_score = cleaned_clusters.iter().map(|x| x.score).sum::<u32>() as f32 / cleaned_clusters.len() as f32;
        let avg_rating = score_to_rating(avg_score as u32);
        let hash = hash_headlines(cleaned_clusters.clone().into_iter().map(|x| x.headline).collect())[0..6].to_string();

        Report {
            do_not_abuse: format!("Please see repo for self hosting."),
            id: hash,
            published:Utc::now().timestamp() as u64, 
            clusters: cleaned_clusters,
            avg_rating: avg_rating,
        }

    }
}

#[derive(Clone)]
struct AppState {
    report: Arc<Mutex<Option<Report>>>
}

pub async fn primary_logic() -> Result<(), Box<dyn std::error::Error>> {
    let watchdog = watchdog::Watchdog::new();
    let supabase_url = env::var("SUPABASE");

    if supabase_url.is_ok() {
        log::info!("Running in Supabase setup");
        
        let connection_url: String = supabase_url?;
        let (client, connection) = tokio_postgres::connect(&connection_url, NoTls).await?;

        // Spawn the connection handler
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        loop {
            let clusters = watchdog.extract_clusters().await;

            let report = Report::new(&clusters);
            let report_string: String = serde_json::to_string(&report)?;

            let query = "UPDATE data SET value = $1 WHERE id = 1";
            let rows_affected = client.execute(query, &[&report_string]).await?;

            log::info!("{:?} Uploaded report, {:?} row update", report.id, rows_affected);

            time::sleep(Duration::from_secs(60 * 5)).await;
            log::info!("Sleeping for 5 minutes");
        }
    } else {
        log::info!("Running in self-hosting setup");

        let report = Arc::new(Mutex::new(None));
        let state = AppState {
            report: Arc::clone(&report),
        };

        let app = Router::new()
            .route("/", get(root))
            .route("/get_report", get(get_report))
            .route("/static/*path", get(static_files))
            .with_state(state);

        let listener = TcpListener::bind("0.0.0.0:8000").await?;
        log::info!("Running on http://0.0.0.0:8000");

        // Spawn the server
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("Server error: {}", e);
            }
        });

        loop {
            let clusters = watchdog.extract_clusters().await;

            let new_report = Report::new(&clusters);

            {
                let mut locked = report.lock().map_err(|_| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Mutex lock poisoned",
                    )) as Box<dyn std::error::Error>
                })?;
                *locked = Some(new_report);
            }

            time::sleep(Duration::from_secs(60 * 5)).await;
            log::info!("Sleeping for 5 minutes before building next report...");
        }
    }
}


#[tokio::main]
async fn main() {

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();


    loop {
        info!("Starting up...");
        let result = primary_logic().await;

        match result {
            Ok(_) => {
                info!("uh? primary logic finished without error?");
            },
            Err(e) => {
                warn!("Primary logic error: {:?}", e);
                
            }
        }

        warn!("Program loop stopped, restarting in 1 minute...");
        thread::sleep(Duration::from_secs(60 * 1));
    }

}

async fn root() -> impl IntoResponse {
    let index_html = include_str!("../index.html");
    Html(index_html)
}

async fn static_files(Path(path): Path<String>) -> impl IntoResponse {
    static STATIC_FOLDER: Dir<'_> = include_directory!("static/");
    let data = STATIC_FOLDER.get_file(path.clone());
    
    match data {
        Some(data) => {
            
            let mimetype = data.mimetype();
            let body = data.contents();

            let stream = ReaderStream::new(body);
            let body = axum::body::Body::from_stream(stream);

            let headers = AppendHeaders([
                (header::CONTENT_TYPE, format!("{}; charset=utf-8", mimetype)),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", path),
                ),
            ]);
            
            return Ok((headers, body))
        },
        None => {
            warn!("Client requested invalid static file: {}", path);
            return Err((StatusCode::NOT_FOUND, format!("File not found: {}", path)))
        }
    }

    
}

async fn get_report(State(state): State<AppState>,) -> (StatusCode, Result<Json<Report>, String>) {
    info!("Client requested report");

    let report;
    {
        let locked = state.report.lock().unwrap();
        report = locked.clone();
    }

    match report {
        Some(report) => {
            return (StatusCode::CREATED, Ok(Json(report)))
        },
        None => {
            return (StatusCode::NOT_FOUND, Err(format!("Report still building")))
        }
    }

    
}
