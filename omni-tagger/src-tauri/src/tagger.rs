use crate::config::PreprocessConfig;
use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView};
use ndarray::Array4;
use ort::session::{builder::GraphOptimizationLevel, Session};
use std::fs::File;

pub struct Tagger {
    session: Session,
    tags: Vec<String>,
    config: PreprocessConfig,
}

impl Tagger {
    pub fn new(
        model_path: &str,
        tags_csv_path: &str,
        config: PreprocessConfig,
    ) -> Result<Self> {
        // Load tags
        let file = File::open(tags_csv_path).context("Failed to open tags file")?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file);

        let mut tags = Vec::new();
        for result in rdr.records() {
            let record = result.context("Failed to read CSV record")?;
            if let Some(tag) = record.get(1) {
                tags.push(tag.to_string());
            }
        }

        // Initialize ORT session
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)
            .context("Failed to load model")?;

        Ok(Self {
            session,
            tags,
            config,
        })
    }

    pub fn infer(&mut self, image: &DynamicImage, threshold: f32) -> Result<Vec<(String, f32)>> {
        let input_tensor = preprocess(image, &self.config);

        // Run inference
        // Explicitly create Value from ndarray
        let input_value = ort::value::Value::from_array(input_tensor)?;
        let outputs = self.session.run(ort::inputs!["input_1" => input_value])?;

        // Get output.
        let (_, data) = outputs[0].try_extract_tensor::<f32>()?;

        let mut results = Vec::new();
        // Skip first 4 tags (ratings)
        for (i, &score) in data.iter().enumerate() {
            if i < 4 {
                continue;
            }
            if score > threshold {
                if let Some(tag) = self.tags.get(i) {
                    results.push((tag.clone(), score));
                }
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }
}

// Preprocessing helper
fn preprocess(image: &DynamicImage, config: &PreprocessConfig) -> Array4<f32> {
    let size = config.input_size;
    let resized = image.resize_exact(size, size, image::imageops::FilterType::CatmullRom);

    let mut input = Array4::<f32>::zeros((1, size as usize, size as usize, 3));
    let normalize_factor = if config.normalize { 255.0 } else { 1.0 };

    for (x, y, pixel) in resized.pixels() {
        let r = pixel[0] as f32 / normalize_factor;
        let g = pixel[1] as f32 / normalize_factor;
        let b = pixel[2] as f32 / normalize_factor;

        if config.format == "bgr" {
            // BGR order
            input[[0, y as usize, x as usize, 0]] = b;
            input[[0, y as usize, x as usize, 1]] = g;
            input[[0, y as usize, x as usize, 2]] = r;
        } else {
            // Assume RGB
            input[[0, y as usize, x as usize, 0]] = r;
            input[[0, y as usize, x as usize, 1]] = g;
            input[[0, y as usize, x as usize, 2]] = b;
        }
    }

    input
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn test_preprocess_default() {
        let mut img = RgbImage::new(100, 100);
        for x in 0..100 {
            for y in 0..100 {
                img.put_pixel(x, y, Rgb([255, 0, 0])); // Red
            }
        }
        let dynamic_img = DynamicImage::ImageRgb8(img);

        // Use default config, which should have normalize: false (after update)
        // or we manually create config to test specific behavior
        let config = PreprocessConfig {
             input_size: 448,
             format: "bgr".to_string(),
             normalize: false,
        };

        let tensor = preprocess(&dynamic_img, &config);

        assert_eq!(tensor.shape(), &[1, 448, 448, 3]);

        // Default is BGR [0, 255]
        assert_eq!(tensor[[0, 0, 0, 0]], 0.0); // B
        assert_eq!(tensor[[0, 0, 0, 1]], 0.0); // G
        assert_eq!(tensor[[0, 0, 0, 2]], 255.0); // R
    }

    #[test]
    fn test_preprocess_custom_normalized() {
        let mut img = RgbImage::new(100, 100);
        for x in 0..100 {
            for y in 0..100 {
                img.put_pixel(x, y, Rgb([255, 0, 0])); // Red
            }
        }
        let dynamic_img = DynamicImage::ImageRgb8(img);
        let config = PreprocessConfig {
            input_size: 224,
            format: "rgb".to_string(),
            normalize: true, // Normalized [0, 1]
        };

        let tensor = preprocess(&dynamic_img, &config);

        assert_eq!(tensor.shape(), &[1, 224, 224, 3]);

        // Custom is RGB [0, 1]
        assert_eq!(tensor[[0, 0, 0, 0]], 1.0); // R
        assert_eq!(tensor[[0, 0, 0, 1]], 0.0); // G
        assert_eq!(tensor[[0, 0, 0, 2]], 0.0); // B
    }

    #[tokio::test]
    #[ignore] // Requires model files and runtime environment
    async fn test_inference_performance() {
        use std::time::Instant;

        // Paths should be adjusted to where models are expected during test
        // This is a placeholder as we don't have models in the repo
        let model_path = "model.onnx";
        let tags_path = "selected_tags.csv";

        if !std::path::Path::new(model_path).exists() || !std::path::Path::new(tags_path).exists() {
            println!("Skipping performance test: Model files not found.");
            return;
        }

        let config = PreprocessConfig {
             input_size: 448,
             format: "bgr".to_string(),
             normalize: false,
        };
        let mut tagger = Tagger::new(model_path, tags_path, config).expect("Failed to load tagger");

        // Create a dummy image
        let img = DynamicImage::ImageRgb8(RgbImage::new(1000, 1000)); // Large image to test resizing too

        // Warmup (optional, but good for accurate inference timing)
        let _ = tagger.infer(&img, 0.5).unwrap();

        let start = Instant::now();
        let _ = tagger.infer(&img, 0.5).unwrap();
        let duration = start.elapsed();

        println!("Inference time: {:?}", duration);

        // Verify performance constraint (e.g. < 1 second)
        assert!(duration.as_secs_f32() < 1.0, "Inference took too long: {:?}", duration);
    }
}
