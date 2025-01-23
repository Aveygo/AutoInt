
mod fetcher;
mod community;
use rustpotion::{RustPotion, PotionModel};
use serde::Serialize;

use std::path::Path;
use chrono::prelude::*;
use std::time::Instant;

use log::info;

#[derive(Clone, Debug, Serialize)]
pub struct EventCluster {
    pub sources: Vec<fetcher::Event>,
    pub score: f32
}

pub struct Watchdog {
    pub clusters: Result<Vec<EventCluster>, ()>,

    positive_tag:Vec<f32>,
    negative_tag:Vec<f32>,

    model:RustPotion,
    fetcher:fetcher::Fetcher
}

impl Watchdog {
    pub fn new() -> Self {

        let model = RustPotion::new(PotionModel::BASE2M, Path::new("models"));
        let positive_tag = model.encode("High priority news: geopolitics, war, disaster, collapse, presidents, nations");
        let negative_tag = model.encode("Low priority news: opinions, celebrity, sports, gossip, local crimes, weather, music, puzzles");

        Self {
            clusters: Err(()),
            positive_tag: positive_tag,
            negative_tag: negative_tag,

            model: model,
            fetcher: fetcher::Fetcher::new(vec![
                "https://www.9news.com/feeds/syndication/rss/news".to_string(),
                "https://content.api.nytimes.com/svc/news/v3/all/recent.rss".to_string(),
                "https://www.independent.co.uk/news/rss".to_string(),
                "https://www.theguardian.com/world/rss".to_string(),
                "https://www.sbs.com.au/news/topic/latest/feed".to_string(),
                "https://moxie.foxnews.com/google-publisher/world.xml".to_string(),
                "https://feeds.a.dj.com/rss/RSSWorldNews.xml".to_string(),
                "https://www.forbes.com/innovation/feed2".to_string(),
                "https://www.forbes.com/business/feed".to_string(),
                "http://feeds.bbci.co.uk/news/world/rss.xml".to_string(),
                "https://www.news.com.au/content-feeds/latest-news-world/".to_string(),
                "https://www.smh.com.au/rss/feed.xml".to_string(),
                "https://www.heraldsun.com.au/rss".to_string(),
                "https://www.abc.net.au/news/feed/51120/rss.xml".to_string(),
                "https://www.thesundaily.my/rss/world".to_string(),
                "https://feeds.washingtonpost.com/rss/politics".to_string(),
                "http://rss.cnn.com/rss/edition.rss".to_string(),
                "https://www.reutersagency.com/feed/?taxonomy=best-sectors&post_type=best".to_string(),
                "https://www.yahoo.com/news/rss".to_string(),
                "https://thehill.com/news/feed/".to_string(),
                "https://www.economist.com/the-world-this-week/rss.xml".to_string(),
                "https://www.theatlantic.com/feed/all/".to_string(),
                "http://www.newsweek.com/rss".to_string(),
                "https://www.theverge.com/rss/index.xml".to_string(),
                "https://techcrunch.com/feed/".to_string(),
                "http://feeds2.feedburner.com/businessinsider".to_string(),
                "http://www.chinadaily.com.cn/rss/world_rss.xml".to_string(),
                "https://www.thehindu.com/news/feeder/default.rss".to_string(),
                "http://www.globaltimes.cn/rss/outbrain.xml".to_string(),
                "https://feeds.feedburner.com/CoinDesk".to_string(),
                "http://fortune.com/feed/".to_string(),
                "https://www.reddit.com/r/news/.rss".to_string(),
                "https://www.reddit.com/r/worldnews/.rss".to_string(),
                "https://www.cbc.ca/cmlink/rss-topstories".to_string(),
                "https://www.ctvnews.ca/rss/ctvnews-ca-top-stories-public-rss-1.822009".to_string(),
                "https://www.france24.com/en/rss".to_string(),
                "https://www.rt.com/rss/".to_string(),
                "https://rss.dw.com/rdf/rss-en-all".to_string(),
                "https://globalnews.ca/feed/".to_string(),
            ])
        }
    }

    fn score_cluster(&self, cluster:&Vec<fetcher::Event>, cluster_priorities:Vec<f32>) -> f32 {

        let priority = cluster_priorities.iter().sum::<f32>() / cluster.len() as f32;
        let cluster_size = cluster.len() as f32;
        let avg_age = (Utc::now().timestamp() as f32 - cluster.iter().map(|x| x.published).sum::<i64>() as f32 / cluster.len() as f32) / 60.0 / 60.0;
        let result = (cluster_size.log(2.17) * (priority + 1.0)) / ((avg_age + 1.0).powf(1.8)) * 1000.0;

        info!("Cluster: {:?}", cluster[0].headline);
        info!("Priority: {:?}, Size: {:?}, Age: {:?}, Final Score: {:?}", priority, cluster_size, avg_age, result);
        info!("--------------------------------------------------------------------------");

        return result
    }

    fn sort_clusters(&self, clusters:Vec<Vec<usize>>, events:&Vec<fetcher::Event>, priorities:Vec<f32>) -> Vec<EventCluster> {

        let mut result = vec![]; 
        for cluster_idxs in clusters {

            let mut cluster = vec![];
            let mut cluster_priorities = vec![];
            for index in cluster_idxs {
                cluster.push(events[index].clone());
                cluster_priorities.push(priorities[index].clone());
            }

            result.push(EventCluster{
                score: self.score_cluster(&cluster, cluster_priorities),
                sources: cluster,
            });
        }

        result.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        result

    }

    fn cosine_similarity(&self, vec1: &Vec<f32>, vec2: &Vec<f32>) -> Option<f32> {
        if vec1.len() != vec2.len() {
            return None;
        }
    
        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(x, y)| x * y).sum();
        let magnitude_vec1 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_vec2 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
    
        if magnitude_vec1 == 0.0 || magnitude_vec2 == 0.0 {
            return None;
        }
    
        Some(dot_product / (magnitude_vec1 * magnitude_vec2))
    }


    fn softmax(&self, vector: Vec<f32>) -> Vec<f32> {
        let max_value = vector
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
    
        let exp_values: Vec<f32> = vector.iter().map(|&x| (x - max_value).exp()).collect();
        let sum_exp: f32 = exp_values.iter().sum();
    
        exp_values.iter().map(|&x| x / sum_exp).collect()
    }

    fn calc_priorities(&self, embeddings:&Vec<Vec<f32>>) -> Vec<f32> {
        let mut result = vec![];

        for embedding in embeddings {
            let p = self.cosine_similarity(embedding, &self.positive_tag).unwrap() / 0.1;
            let n = self.cosine_similarity(embedding, &self.negative_tag).unwrap() / 0.1;

            result.push(self.softmax(vec![p, n])[0]);
        }

        result

    }

    pub async fn extract_clusters(&self) -> Vec<EventCluster> {
        info!("WATCHDOG START");
        let start = Instant::now();
        let mut now = Instant::now(); 

        let events = self.fetcher.run_with_timeout().await;
        info!("Fetched events in : {:.2?}", now.elapsed());
        now = Instant::now();

        let headlines:Vec<String> = events.clone().into_iter().map(|event| event.headline).collect();
        let embeddings = self.model.encode_many(headlines.clone());
        info!("Encoded events in : {:.2?}", now.elapsed());
        now = Instant::now();

        let priorities = self.calc_priorities(&embeddings);
        let clusters = community::find_clusters(&embeddings, 0.6, 3);
        info!("Found clusters in : {:.2?}", now.elapsed());

        let result = self.sort_clusters(clusters, &events, priorities);
        info!("Total time: {:.2?}", start.elapsed());
        result
    }

}