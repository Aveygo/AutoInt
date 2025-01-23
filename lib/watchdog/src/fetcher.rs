use reqwest::get;
use feed_rs::parser;

use tokio;
use tokio::time::{timeout, Duration};
use futures::future::join_all;
use serde::Serialize;

use unicode_normalization::UnicodeNormalization;

use log::warn;

#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub headline:String,
    pub published:i64,
    pub href:String
}

pub struct Fetcher {
    rss_urls: Vec<String>
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
                events
            }
            Err(_) => {
                vec![]
            }
        }
    }

    async fn extract_feed(url: String) -> Vec<Event> {
        let mut result = vec![];
    
        let feed = match timeout(Duration::from_secs(5), get(url.clone())).await {
            Ok(Ok(response)) => match response.text().await {
                Ok(body) => match parser::parse(body.as_bytes()) {
                    Ok(feed) => feed.entries,
                    Err(_e) => {
                        warn!("Could not parse from {:?}", url);
                        vec![]
                    },
                },
                Err(_e) => {
                    warn!("Invalid response from {:?}", url);
                    vec![]
                },
            },
            Ok(Err(_e)) => {
                warn!("Get request failed for {:?}", url);
                vec![]
            },
            Err(_timeout) => {
                warn!("Reached timeout for {:?}", url);
                vec![]
            },
        };
    
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
