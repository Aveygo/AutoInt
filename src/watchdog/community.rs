use ndarray;
use std::ops::Mul;
use std::cmp::{max, min};
use std::collections::HashSet;


#[derive(Debug, Clone, Copy)]
struct Element {
    idx: usize,
    value: f32
}

fn transpose(matrix: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    if matrix.is_empty() || matrix[0].is_empty() {
        return Vec::new();
    }
    let rows = matrix.len();
    let cols = matrix[0].len();

    let mut transposed = vec![vec![matrix[0][0].clone(); rows]; cols];

    for i in 0..rows {
        for j in 0..cols {
            transposed[j][i] = matrix[i][j].clone();
        }
    }
    transposed
}

fn matrix_multiply(a: Vec<Vec<f32>>, b: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let rows_a = a.len();
    let cols_a = a[0].len();
    let rows_b = b.len();
    let cols_b = b[0].len();

    assert_eq!(cols_a, rows_b, "Number of columns in A must equal number of rows in B");

    let mut result = vec![vec![0.0; cols_b]; rows_a];

    for i in 0..rows_a {
        for j in 0..cols_b {
            for k in 0..cols_a {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }

    result
}

fn top_k_1d(row: &Vec<f32>, k: usize, largest: bool) -> Vec<Element> {
    let mut row: Vec<Element> = row.clone()
        .into_iter()
        .enumerate()
        .map(|(idx, value)| Element { idx, value })
        .collect();
        
    if largest {
        row.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
    } else {
        row.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }

    row.into_iter().take(k).clone().collect()

}

fn top_k_2d(matrix: &Vec<Vec<f32>>, k: usize, largest: bool) -> Vec<Vec<Element>> {
    let mut result = vec![];
    for i in 0..matrix.len() {
        result.push(top_k_1d(&matrix[i], k, largest));

    }

    result
}


pub fn find_clusters(embeddings:Vec<Vec<f32>>, threshold:f32, min_cluster_size:usize) -> Vec<Vec<usize>> {

    let emb_len = embeddings.len();
    let mut extracted_communities:Vec<Vec<usize>> = vec![];
    let mut sort_max_size = min(max(2 * min_cluster_size, 50), emb_len);

    let cos_scores = matrix_multiply(embeddings.clone(), transpose(embeddings));
    let top_k_elements = top_k_2d(&cos_scores, min_cluster_size, true);

    for i in 0..top_k_elements.len() {
        if top_k_elements[i][top_k_elements[i].len()-1].value > threshold {

            let row = cos_scores[i].clone();
            let mut top_val_elements = top_k_1d(&row, sort_max_size, true);

            while top_val_elements[top_val_elements.len()-1].value > threshold && sort_max_size < emb_len {
                sort_max_size = min(2 * sort_max_size, emb_len);
                top_val_elements = top_k_1d(&row, sort_max_size, true);
            }

            let community:Vec<usize> =  top_val_elements.into_iter()
                .filter(|e| e.value > threshold) 
                .map(|e| e.idx) 
                .collect();
            
            extracted_communities.push(
               community
            );
        }
    }

    extracted_communities.sort_by(|a, b| b.len().partial_cmp(&a.len()).unwrap());
    let mut unique_communities = vec![];
    let mut extracted_ids = HashSet::new();

    for (cluser_idx, community) in extracted_communities.into_iter().enumerate() {
        let mut non_overlapped_community = vec![];
        for idx in community {
            if !(extracted_ids.contains(&idx)) {
                non_overlapped_community.push(idx)
            }
        }

        if non_overlapped_community.len() >= min_cluster_size {
            unique_communities.push(non_overlapped_community.clone());
            extracted_ids.extend(non_overlapped_community);
        }
    }

    unique_communities.sort_by(|a, b| b.len().partial_cmp(&a.len()).unwrap());


    unique_communities 
}