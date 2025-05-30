use feed_rs::parser;

use tokio;
use tokio::time::{timeout, Duration};
use futures::future::join_all;
use serde::Serialize;
use reqwest::Client;
use log::{info, warn};

use unicode_normalization::UnicodeNormalization;


#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub headline:String,
    pub published:i64,
    pub href:String
}

pub struct Fetcher {
    pub rss_urls: Vec<String>
}

impl Fetcher {
    pub fn new(rss_urls: Vec<String>) -> Self {
        Self{
            rss_urls
        }
    }
    
    fn is_english(text: &String) -> bool {
        let common_english_words = vec![" the ", " and ", " is ", " in ", " it ", " of "];
        common_english_words.iter().any(|&word| text.contains(word))
    }
    
    pub async fn run(&self) -> Vec<Event> {
        info!("Spawning extract tasks...");
        let futures: Vec<_> = self.rss_urls.clone().into_iter().map(|url| tokio::spawn(Self::extract_feed(url))).collect();
        let results: Vec<Result<Vec<Event>, tokio::task::JoinError>> = join_all(futures).await;
        
        let mut events: Vec<Event> = Vec::new();
        
        for result in results {
            match result {
                Ok(feed_events) => {
                    events.extend(feed_events);
                },
                Err(_) => {}
            }
        }
        
        events
    }

    pub async fn run_with_timeout(&self) -> Vec<Event> {
        match timeout(Duration::from_secs(10), self.run()).await {
            Ok(events) => {
                info!("Fetcher found {:?} events", events.len());
                events
            }
            Err(_) => {
                warn!("Fetcher critical timeout error! No events were found! (Internet down?)");
                vec![]
            }
        }
    }

    async fn extract_feed(url: String) -> Vec<Event> {
        let mut result = vec![];

        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .build()
            .expect("Failed to build client");
        
        info!("Built client for {}", url);

        let feed = match timeout(Duration::from_secs(5), client.get(url.clone()).send()).await {
            Ok(Ok(response)) => {
                info!("Received response for {}", url);
                match response.text().await {
                    Ok(body) => {
                        info!("Got body for {}, parsing...", url);
                        match tokio::task::spawn_blocking(move || parser::parse(body.as_bytes())).await {
                            Ok(Ok(feed)) => feed.entries,
                            Ok(Err(_e)) => {
                                warn!("Could not parse from {}", url);
                                vec![]
                            }
                            Err(_e) => {
                                warn!("Spawn blocking failed for {}", url);
                                vec![]
                            }
                        }
                    }
                    Err(_e) => {
                        warn!("Invalid response from {}", url);
                        vec![]
                    }
                }
            }
            Ok(Err(_e)) => {
                warn!("Get request failed for {}", url);
                vec![]
            }
            Err(_timeout) => {
                warn!("Reached timeout for {}", url);
                vec![]
            }
        };
        info!("Processed {} entries for {}", feed.len(), url);
    
        for item in feed {
            if let (Some(title), Some(published), Some(href)) = (
                item.title,
                item.published,
                item.links.get(0).map(|link| &link.href),
            ) {
                let headline: String = title.content.nfc().collect();
                if headline.len() > 5 && Self::is_english(&headline) {
                    result.push(Event {
                        headline: headline,
                        published: published.timestamp(),
                        href: href.clone(),
                    });
                }
            }
        }
    
        result
    }
    
}
