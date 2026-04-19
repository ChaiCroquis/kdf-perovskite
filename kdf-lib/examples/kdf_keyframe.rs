//! Video Keyframe Extraction with KDF
//!
//! This example demonstrates how to extract keyframes from video:
//! - Core: Redundant frames (can be skipped)
//! - Edge: Transition frames (optional inclusion)
//! - Rare: Unique keyframes (must include)
//!
//! Key insight: KDF naturally identifies distinctive visual moments.
//!
//! Run: cargo run --example kdf_keyframe

use kdf::{cosine_similarity, Kdf, Layer};

/// Represents a video frame with visual features
#[derive(Clone)]
struct VideoFrame {
    frame_number: u32,
    timestamp_ms: u64,
    /// Feature vector from image embedding (simulated)
    /// In practice: CNN features, histogram, motion vectors
    features: Vec<f64>,
    scene_label: String,
}

/// Keyframe extraction result
struct KeyframeResult {
    keyframe_indices: Vec<usize>,
    transition_indices: Vec<usize>,
    compression_ratio: f64,
    scene_changes: Vec<(u32, u32)>,
}

/// Extract keyframes using KDF layer analysis
fn extract_keyframes(frames: &[VideoFrame], threshold: f64) -> KeyframeResult {
    let kdf = Kdf::with_defaults();

    let result = kdf.process(frames, threshold, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    let mut keyframe_indices = Vec::new();
    let mut transition_indices = Vec::new();

    for (i, &layer) in result.layers.iter().enumerate() {
        match layer {
            Layer::Rare => keyframe_indices.push(i),
            Layer::Edge => transition_indices.push(i),
            Layer::Core => {}  // Skip redundant frames
        }
    }

    // Detect scene changes (consecutive rare frames from different scenes)
    let mut scene_changes = Vec::new();
    for i in 1..frames.len() {
        if result.layers[i] == Layer::Rare && result.layers[i - 1] != Layer::Rare {
            if frames[i].scene_label != frames[i - 1].scene_label {
                scene_changes.push((frames[i - 1].frame_number, frames[i].frame_number));
            }
        }
    }

    let total_frames = frames.len();
    let selected_frames = keyframe_indices.len() + transition_indices.len();
    let compression_ratio = 1.0 - (selected_frames as f64 / total_frames as f64);

    KeyframeResult {
        keyframe_indices,
        transition_indices,
        compression_ratio,
        scene_changes,
    }
}

/// Generate simulated video frames for demonstration
fn generate_video_frames() -> Vec<VideoFrame> {
    let mut frames = Vec::new();
    let mut frame_num = 0u32;

    // Scene 1: Indoor office (frames 0-9)
    for i in 0..10 {
        let variation = (i as f64) * 0.02;
        frames.push(VideoFrame {
            frame_number: frame_num,
            timestamp_ms: (frame_num as u64) * 33,  // ~30fps
            features: vec![0.8 + variation, 0.2, 0.1, 0.3, 0.1],
            scene_label: "office".into(),
        });
        frame_num += 1;
    }

    // Scene transition: office to outdoor (frames 10-12)
    for i in 0..3 {
        let blend = (i as f64 + 1.0) / 4.0;
        frames.push(VideoFrame {
            frame_number: frame_num,
            timestamp_ms: (frame_num as u64) * 33,
            features: vec![
                0.8 * (1.0 - blend) + 0.2 * blend,
                0.2 * (1.0 - blend) + 0.7 * blend,
                0.1 * (1.0 - blend) + 0.8 * blend,
                0.3,
                0.2,
            ],
            scene_label: "transition".into(),
        });
        frame_num += 1;
    }

    // Scene 2: Outdoor nature (frames 13-22)
    for i in 0..10 {
        let variation = (i as f64) * 0.015;
        frames.push(VideoFrame {
            frame_number: frame_num,
            timestamp_ms: (frame_num as u64) * 33,
            features: vec![0.2, 0.7 - variation, 0.8, 0.3, 0.2],
            scene_label: "nature".into(),
        });
        frame_num += 1;
    }

    // Action moment in nature (frame 23) - unique!
    frames.push(VideoFrame {
        frame_number: frame_num,
        timestamp_ms: (frame_num as u64) * 33,
        features: vec![0.5, 0.9, 0.9, 0.8, 0.7],  // High motion
        scene_label: "nature_action".into(),
    });
    frame_num += 1;

    // Scene 2 continues (frames 24-28)
    for i in 0..5 {
        let variation = (i as f64) * 0.01;
        frames.push(VideoFrame {
            frame_number: frame_num,
            timestamp_ms: (frame_num as u64) * 33,
            features: vec![0.2, 0.65 + variation, 0.75, 0.3, 0.2],
            scene_label: "nature".into(),
        });
        frame_num += 1;
    }

    // Scene 3: Close-up face (frames 29-34)
    for i in 0..6 {
        let variation = (i as f64) * 0.02;
        frames.push(VideoFrame {
            frame_number: frame_num,
            timestamp_ms: (frame_num as u64) * 33,
            features: vec![0.6, 0.3, 0.2, 0.9 - variation, 0.8],
            scene_label: "closeup".into(),
        });
        frame_num += 1;
    }

    frames
}

fn main() {
    println!("=== Video Keyframe Extraction with KDF ===\n");

    let frames = generate_video_frames();
    println!("Total frames: {} ({:.1}s at 30fps)\n",
        frames.len(),
        frames.len() as f64 / 30.0);

    // =========================================================================
    // 1. Basic KDF Analysis
    // =========================================================================
    println!("--- KDF Layer Analysis ---\n");

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&frames, 0.98, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    let mut current_scene = String::new();
    for (i, frame) in frames.iter().enumerate() {
        if frame.scene_label != current_scene {
            println!("\n[Scene: {}]", frame.scene_label);
            current_scene = frame.scene_label.clone();
        }

        let layer = result.layers[i];
        let icon = match layer {
            Layer::Rare => "★",
            Layer::Edge => "◇",
            Layer::Core => "·",
        };
        print!("{}", icon);
    }
    println!("\n\nLegend: ★=Rare (keyframe), ◇=Edge (transition), ·=Core (skip)\n");

    // =========================================================================
    // 2. Extract Keyframes
    // =========================================================================
    println!("--- Keyframe Extraction ---\n");

    let extraction = extract_keyframes(&frames, 0.98);

    println!("Keyframes (must include): {:?}", extraction.keyframe_indices);
    println!("Transitions (optional): {:?}", extraction.transition_indices);
    println!("Compression ratio: {:.1}%", extraction.compression_ratio * 100.0);
    println!();

    // =========================================================================
    // 3. Detailed Keyframe Info
    // =========================================================================
    println!("--- Keyframe Details ---\n");

    for &idx in &extraction.keyframe_indices {
        let frame = &frames[idx];
        println!("Frame {}: {}ms - Scene: {}",
            frame.frame_number,
            frame.timestamp_ms,
            frame.scene_label);
    }
    println!();

    // =========================================================================
    // 4. Scene Change Detection
    // =========================================================================
    println!("--- Scene Changes ---\n");

    if extraction.scene_changes.is_empty() {
        println!("No abrupt scene changes detected");
    } else {
        for (from, to) in &extraction.scene_changes {
            println!("Scene change: frame {} -> frame {}", from, to);
        }
    }
    println!();

    // =========================================================================
    // 5. Adaptive Sampling Strategies
    // =========================================================================
    println!("--- Adaptive Sampling ---\n");

    // Strategy 1: Minimal (only Rare)
    let minimal_frames: Vec<u32> = extraction.keyframe_indices.iter()
        .map(|&i| frames[i].frame_number)
        .collect();
    println!("Minimal (Rare only): {} frames -> {:?}",
        minimal_frames.len(), minimal_frames);

    // Strategy 2: Standard (Rare + Edge)
    let standard_frames: Vec<u32> = extraction.keyframe_indices.iter()
        .chain(extraction.transition_indices.iter())
        .map(|&i| frames[i].frame_number)
        .collect();
    println!("Standard (Rare+Edge): {} frames", standard_frames.len());

    // Strategy 3: Balanced (Rare + selected Core representatives)
    let core_indices: Vec<usize> = (0..frames.len())
        .filter(|&i| result.layers[i] == Layer::Core)
        .collect();

    // Sample every 5th core frame
    let sampled_core: Vec<usize> = core_indices.iter()
        .enumerate()
        .filter(|(i, _)| i % 5 == 0)
        .map(|(_, &idx)| idx)
        .collect();

    let balanced_count = extraction.keyframe_indices.len()
        + extraction.transition_indices.len()
        + sampled_core.len();
    println!("Balanced (Rare+Edge+sampled Core): {} frames", balanced_count);
    println!();

    // =========================================================================
    // 6. Thumbnail Generation Guide
    // =========================================================================
    println!("--- Thumbnail Generation ---\n");

    println!("Recommended thumbnails for video preview:");
    let thumbnail_candidates: Vec<_> = extraction.keyframe_indices.iter()
        .take(5)
        .map(|&i| &frames[i])
        .collect();

    for (i, frame) in thumbnail_candidates.iter().enumerate() {
        println!("  {}. Frame {} at {:.1}s ({})",
            i + 1,
            frame.frame_number,
            frame.timestamp_ms as f64 / 1000.0,
            frame.scene_label);
    }
    println!();

    // =========================================================================
    // Summary
    // =========================================================================
    println!("=== Summary ===");
    println!("KDF Keyframe Extraction benefits:");
    println!("1. Automatic identification of unique visual moments");
    println!("2. Scene change detection through layer transitions");
    println!("3. Adjustable compression via threshold tuning");
    println!("4. No need for explicit scene boundary annotation");
    println!("5. Works with any visual feature representation");
}
