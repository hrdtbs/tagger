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
}
