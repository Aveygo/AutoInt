
mod fetcher;
mod community;
use rustpotion::{RustPotion, PotionModel};
use std::sync::{Mutex, Arc};
use serde::{Deserialize, Serialize};

use std::path::Path;
use std::{thread, vec};
use std::time::Duration;
use chrono::prelude::*;

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
                "http://www.newsweek.com/rss".to_string(),
                "https://www.9news.com/feeds/syndication/rss/news".to_string(),
                "https://content.api.nytimes.com/svc/news/v3/all/recent.rss".to_string(),
                "https://www.independent.co.uk/news/rss".to_string(),
                "https://www.theguardian.com/world/rss".to_string(),
                "https://www.sbs.com.au/news/topic/latest/feed".to_string(),
                "https://moxie.foxnews.com/google-publisher/world.xml".to_string(),
                "https://feeds.a.dj.com/rss/RSSWorldNews.xml".to_string(),
                "https://www.forbes.com/innovation/feed".to_string(),
                "https://www.forbes.com/business/feed".to_string()
            ])
        }
    }

    fn score_cluster(&self, cluster:&Vec<fetcher::Event>, cluster_priorities:Vec<f32>) -> f32 {

        let priority = cluster_priorities.iter().sum::<f32>() / cluster.len() as f32;
        let cluster_size = cluster.len() as f32;
        let avg_age = (Utc::now().timestamp() as f32 - cluster.iter().map(|x| x.published).sum::<i64>() as f32 / cluster.len() as f32) / 60.0 / 60.0;

        return (cluster_size.log(2.17) * (priority + 1.0)) / ((avg_age + 1.0).powf(1.8)) * 1000.0
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
            let p = self.cosine_similarity(embedding, &self.positive_tag).unwrap();
            let n = self.cosine_similarity(embedding, &self.negative_tag).unwrap();

            result.push(self.softmax(vec![p, n])[0]);
        }

        result

    }

    pub async fn extract_clusters(&self) -> Vec<EventCluster> {
        let events = self.fetcher.run().await.unwrap();
        let headlines:Vec<String> = events.clone().into_iter().map(|event| event.headline).collect();
        let embeddings = self.model.encode_many(headlines.clone());
        let priorities = self.calc_priorities(&embeddings);
        let clusters = community::find_clusters(&embeddings, 0.6, 3);
        let result = self.sort_clusters(clusters, &events, priorities);
        result
    }

}