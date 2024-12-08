use reqwest::get;
use feed_rs::parser;

use tokio;
use std::thread;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

use unicode_normalization::UnicodeNormalization;

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
    
    pub async fn run(&self) -> Result<Vec<Event>, ()> {
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
        
        if !events.is_empty() {
            Ok(events)
        } else {
            Err(())
        }
    }

    async fn extract_feed(url: String) -> Vec<Event> {
        let mut result = vec![];

        let feed = match get(url).await {
            Ok(response) => {
                match response.text().await {
                    Ok(body) => {
                        match parser::parse(body.as_bytes()) {
                            Ok(feed) => {feed.entries}
                            Err(_e) => {vec![]}
                        }
                    }
                    Err(_e)=> {vec![]}

                }

            }
            Err(_e) => {vec![]}
        };

    
        for item in feed {
            if let (Some(title), Some(published), Some(href)) = (
                item.title,
                item.published,
                item.links.get(0).map(|link| &link.href),
            ) {

                let headline:String = title.content.nfc().collect();
                if headline.len() > 5 {
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
