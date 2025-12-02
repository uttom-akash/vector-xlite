//! # Test Data Generators
//!
//! Utilities for generating test vectors and data.
//!
//! ## Usage
//!
//! ```rust
//! use common::generators::*;
//!
//! let vec = random_vector(128);
//! let normalized = unit_vector(128);
//! let batch = vector_batch(100, 128);
//! ```

use std::f32::consts::PI;

// ============================================================================
// Vector Generators
// ============================================================================

/// Generate a random vector with values in [0, 1)
pub fn random_vector(dim: usize) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);

    (0..dim)
        .map(|i| {
            i.hash(&mut hasher);
            let h = hasher.finish();
            (h as f32) / (u64::MAX as f32)
        })
        .collect()
}

/// Generate a deterministic random vector (same seed = same output)
pub fn seeded_vector(dim: usize, seed: u64) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);

    (0..dim)
        .map(|i| {
            i.hash(&mut hasher);
            let h = hasher.finish();
            (h as f32) / (u64::MAX as f32)
        })
        .collect()
}

/// Generate a zero vector
pub fn zero_vector(dim: usize) -> Vec<f32> {
    vec![0.0; dim]
}

/// Generate a vector of all ones
pub fn ones_vector(dim: usize) -> Vec<f32> {
    vec![1.0; dim]
}

/// Generate a unit vector (normalized to length 1)
pub fn unit_vector(dim: usize) -> Vec<f32> {
    let v = random_vector(dim);
    normalize(&v)
}

/// Generate a unit vector along a specific axis
pub fn axis_vector(dim: usize, axis: usize) -> Vec<f32> {
    let mut v = vec![0.0; dim];
    if axis < dim {
        v[axis] = 1.0;
    }
    v
}

/// Generate a vector with a specific value at all positions
pub fn constant_vector(dim: usize, value: f32) -> Vec<f32> {
    vec![value; dim]
}

/// Generate a sequential vector [0, 1, 2, ..., dim-1]
pub fn sequential_vector(dim: usize) -> Vec<f32> {
    (0..dim).map(|i| i as f32).collect()
}

/// Generate a normalized sequential vector
pub fn normalized_sequential(dim: usize) -> Vec<f32> {
    let v = sequential_vector(dim);
    normalize(&v)
}

// ============================================================================
// Batch Generators
// ============================================================================

/// Generate a batch of random vectors
pub fn vector_batch(count: usize, dim: usize) -> Vec<Vec<f32>> {
    (0..count).map(|_| random_vector(dim)).collect()
}

/// Generate a batch of seeded vectors (reproducible)
pub fn seeded_batch(count: usize, dim: usize, base_seed: u64) -> Vec<Vec<f32>> {
    (0..count)
        .map(|i| seeded_vector(dim, base_seed + i as u64))
        .collect()
}

/// Generate vectors with IDs
pub fn vectors_with_ids(count: usize, dim: usize) -> Vec<(u64, Vec<f32>)> {
    (1..=count as u64)
        .map(|id| (id, seeded_vector(dim, id)))
        .collect()
}

/// Generate vectors with IDs starting from a specific ID
pub fn vectors_with_ids_from(start_id: u64, count: usize, dim: usize) -> Vec<(u64, Vec<f32>)> {
    (0..count as u64)
        .map(|i| {
            let id = start_id + i;
            (id, seeded_vector(dim, id))
        })
        .collect()
}

// ============================================================================
// Special Pattern Generators
// ============================================================================

/// Generate vectors arranged in clusters
pub fn clustered_vectors(
    clusters: usize,
    points_per_cluster: usize,
    dim: usize,
) -> Vec<(u64, Vec<f32>)> {
    let mut result = Vec::new();
    let mut id = 1u64;

    for cluster in 0..clusters {
        // Generate cluster center
        let center = seeded_vector(dim, (cluster * 1000) as u64);

        for point in 0..points_per_cluster {
            // Add small noise to center
            let noise = seeded_vector(dim, (cluster * 1000 + point) as u64);
            let v: Vec<f32> = center
                .iter()
                .zip(noise.iter())
                .map(|(c, n)| c + n * 0.1)
                .collect();

            result.push((id, v));
            id += 1;
        }
    }

    result
}

/// Generate vectors on a line from start to end
pub fn linear_vectors(start: Vec<f32>, end: Vec<f32>, count: usize) -> Vec<Vec<f32>> {
    (0..count)
        .map(|i| {
            let t = i as f32 / (count - 1).max(1) as f32;
            start
                .iter()
                .zip(end.iter())
                .map(|(s, e)| s + (e - s) * t)
                .collect()
        })
        .collect()
}

/// Generate vectors on a circle (2D embedded in higher dimensions)
pub fn circular_vectors(count: usize, dim: usize) -> Vec<Vec<f32>> {
    (0..count)
        .map(|i| {
            let angle = 2.0 * PI * (i as f32) / (count as f32);
            let mut v = vec![0.0; dim];
            if dim >= 2 {
                v[0] = angle.cos();
                v[1] = angle.sin();
            }
            v
        })
        .collect()
}

/// Generate orthogonal vectors (as many as dimension allows)
pub fn orthogonal_vectors(dim: usize) -> Vec<Vec<f32>> {
    (0..dim).map(|i| axis_vector(dim, i)).collect()
}

// ============================================================================
// Vector Operations
// ============================================================================

/// Normalize a vector to unit length
pub fn normalize(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        v.iter().map(|x| x / norm).collect()
    } else {
        v.to_vec()
    }
}

/// Calculate L2 (Euclidean) distance between two vectors
pub fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

/// Calculate inner product (dot product)
pub fn inner_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Add noise to a vector
pub fn add_noise(v: &[f32], magnitude: f32, seed: u64) -> Vec<f32> {
    let noise = seeded_vector(v.len(), seed);
    v.iter()
        .zip(noise.iter())
        .map(|(x, n)| x + (n - 0.5) * magnitude * 2.0)
        .collect()
}

/// Create a perturbation of a vector (for testing near-duplicate detection)
pub fn perturb(v: &[f32], amount: f32) -> Vec<f32> {
    add_noise(v, amount, 42)
}

// ============================================================================
// ID Generators
// ============================================================================

/// Generate sequential IDs
pub fn sequential_ids(count: usize) -> Vec<u64> {
    (1..=count as u64).collect()
}

/// Generate IDs starting from a specific value
pub fn ids_from(start: u64, count: usize) -> Vec<u64> {
    (start..start + count as u64).collect()
}

/// Generate random-looking but deterministic IDs
pub fn scattered_ids(count: usize, seed: u64) -> Vec<u64> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    (0..count)
        .map(|i| {
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            i.hash(&mut hasher);
            (hasher.finish() % 1_000_000) + 1
        })
        .collect()
}
