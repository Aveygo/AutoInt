use safetensors::SafeTensors;
use serde::Deserialize;
use std::collections::HashMap;
use regex::Regex;


#[derive(Deserialize)]
struct Tokenizer {
    model: ModelVocab,
}

#[derive(Deserialize)]
struct ModelVocab {
    vocab: HashMap<String, i32>, 
}

fn word2tok(word: String, vocab: &HashMap<String, i32>) -> Result<Vec<i32>, ()> {
    let mut tokens = Vec::new();
    let mut word = word;
    let mut i = word.len();
    let mut continuing = false;

    while !word.is_empty() {
        let target = if continuing {
            format!("##{}", &word[..i])
        } else {
            word[..i].to_string()
        };

        match vocab.get(&target) {
            Some(&candidate) => {
                tokens.push(candidate);
                word = word[i..].to_string();
                i = word.len();
                continuing = true;
            },
            None => {
                if i == 0 {
                    return Err(());
                }
                i -= 1;
            }
        }
    }

    Ok(tokens)
}

fn setencen2tok(sentence:&str, vocab:&HashMap<String, i32>) -> Result<Vec<i32>, ()> {
    let re = Regex::new(r"[\w'-]+|[.,!?;]").unwrap();

    let mut all_tokens = vec![];
    let lower = sentence.to_lowercase();
    let words:Vec<&str> = re.find_iter(&lower).map(|mat| mat.as_str()).collect();
    
    for word in words {
        all_tokens.extend(word2tok(word.to_string(), vocab)?)
    }
    return Ok(all_tokens)

}

fn u8_to_f32_vec(v: &[u8]) -> Vec<f32> {
    v.chunks_exact(4)
        .map(TryInto::try_into)
        .map(Result::unwrap)
        .map(f32::from_le_bytes)
        .collect()
}

fn norm(v: &Vec<f32>) -> f32 {
    let sum_of_squares: f32 = v.iter().map(|&x| x * x).sum();
    sum_of_squares.sqrt()
}


pub struct RustPotion {
    embeddings: Vec<f32>, // flattened 2d array: (vocab, dimensions)
    dimensions: usize,
    vocab: HashMap<String, i32>
}

impl RustPotion {
    pub fn new() -> Self {
        
        let (embeddings, dimensions) = Self::load_embeddings();
        let vocab = Self::load_vocab();

        Self {
            embeddings,
            dimensions,
            vocab
        }
    }

    fn load_embeddings() -> (Vec<f32>, usize) {
        let emb_tensors = SafeTensors::deserialize(include_bytes!("model.safetensors")).unwrap();
        let embeddings = emb_tensors.tensor("embeddings").unwrap();
        (u8_to_f32_vec(embeddings.data()), embeddings.shape()[1])
    }

    fn load_vocab() -> HashMap<String, i32> {
        let tokenizer: Tokenizer = serde_json::from_str(&include_str!("tokenizer.json")).unwrap();
        tokenizer.model.vocab
    }

    pub fn encode(self:Self, sentence: &str) -> Vec<f32> {

        let tokens = setencen2tok(sentence, &self.vocab).unwrap();
        let mut out_array = vec![0.0f32; self.dimensions];

        for token in tokens.iter() {
            let tmp_arr = self.embeddings[*token as usize * self.dimensions .. (*token as usize * self.dimensions) + self.dimensions].to_vec();
            for i in 0..tmp_arr.len() {
                out_array[i] += tmp_arr[i]
            }
        }

        for i in 0..out_array.len() {
            out_array[i] /= tokens.len() as f32;
        }
    
        let n = norm(&out_array);

        for i in 0..out_array.len() {
            out_array[i] /=  n;
        }

        out_array

    }
}
