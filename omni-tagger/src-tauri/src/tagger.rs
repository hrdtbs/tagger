use image::{DynamicImage, GenericImageView};
use ndarray::Array4;
use ort::session::{builder::GraphOptimizationLevel, Session};
use std::error::Error;
use std::fs::File;

pub struct Tagger {
    session: Session,
    tags: Vec<String>,
}

impl Tagger {
    pub fn new(model_path: &str, tags_csv_path: &str) -> Result<Self, Box<dyn Error>> {
        // Load tags
        let file = File::open(tags_csv_path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file);

        let mut tags = Vec::new();
        for result in rdr.records() {
            let record = result?;
            if let Some(tag) = record.get(1) {
                tags.push(tag.to_string());
            }
        }

        // Initialize ORT session
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;

        Ok(Self { session, tags })
    }

    pub fn infer(
        &mut self,
        image: &DynamicImage,
        threshold: f32,
    ) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
        let input_tensor = preprocess(image);

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
fn preprocess(image: &DynamicImage) -> Array4<f32> {
    let resized = image.resize_exact(448, 448, image::imageops::FilterType::CatmullRom);

    let mut input = Array4::<f32>::zeros((1, 448, 448, 3));

    for (x, y, pixel) in resized.pixels() {
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;

        // BGR order
        input[[0, y as usize, x as usize, 0]] = b;
        input[[0, y as usize, x as usize, 1]] = g;
        input[[0, y as usize, x as usize, 2]] = r;
    }

    input
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn test_preprocess() {
        let mut img = RgbImage::new(100, 100);
        for x in 0..100 {
            for y in 0..100 {
                img.put_pixel(x, y, Rgb([255, 0, 0])); // Red
            }
        }
        let dynamic_img = DynamicImage::ImageRgb8(img);

        let tensor = preprocess(&dynamic_img);

        assert_eq!(tensor.shape(), &[1, 448, 448, 3]);

        assert_eq!(tensor[[0, 0, 0, 0]], 0.0);
        assert_eq!(tensor[[0, 0, 0, 1]], 0.0);
        assert_eq!(tensor[[0, 0, 0, 2]], 255.0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_inference_performance() {
        use std::time::Instant;

        let model_url = "https://huggingface.co/SmilingWolf/wd-v1-4-swinv2-tagger-v2/resolve/main/model.onnx";
        let tags_url = "https://huggingface.co/SmilingWolf/wd-v1-4-swinv2-tagger-v2/resolve/main/selected_tags.csv";

        let cache_dir = std::env::temp_dir().join("omni-tagger-test-cache");
        std::fs::create_dir_all(&cache_dir).unwrap();

        let model_path = cache_dir.join("model.onnx");
        let tags_path = cache_dir.join("selected_tags.csv");

        // Download if missing
        if !model_path.exists() {
            println!("Downloading model...");
            let client = reqwest::Client::new();
            let resp = client.get(model_url).send().await.expect("Failed to download model");
            let bytes = resp.bytes().await.expect("Failed to get bytes");
            std::fs::write(&model_path, bytes).expect("Failed to write model file");
        }

        if !tags_path.exists() {
            println!("Downloading tags...");
            let client = reqwest::Client::new();
            let resp = client.get(tags_url).send().await.expect("Failed to download tags");
            let bytes = resp.bytes().await.expect("Failed to get bytes");
            std::fs::write(&tags_path, bytes).expect("Failed to write tags file");
        }

        // Load Tagger
        let start_load = Instant::now();
        let mut tagger = Tagger::new(model_path.to_str().unwrap(), tags_path.to_str().unwrap())
            .expect("Failed to load tagger");
        println!("Model loaded in {:?}", start_load.elapsed());

        // Generate dummy image
        let mut img = RgbImage::new(512, 512);
        for x in 0..512 {
            for y in 0..512 {
                img.put_pixel(x, y, Rgb([(x % 255) as u8, (y % 255) as u8, 128]));
            }
        }
        let dynamic_img = DynamicImage::ImageRgb8(img);

        // Run inference
        let start_infer = Instant::now();
        let results = tagger.infer(&dynamic_img, 0.35).expect("Inference failed");
        let duration = start_infer.elapsed();

        println!("Inference took {:?}", duration);
        println!("Top tags: {:?}", results.iter().take(5).collect::<Vec<_>>());

        assert!(duration.as_secs_f32() < 1.0, "Inference took longer than 1 second: {:?}", duration);
    }
}
