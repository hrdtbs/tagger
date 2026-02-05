# Models Directory

This application requires ONNX models to function. Due to their size, they are not included in the repository.

Please download the following files and place them in this directory:

1. **model.onnx**: The WD14 tagger model (e.g., SwinV2 or ConvNext).
   - Recommended: [SmilingWolf/wd-v1-4-convnext-tagger-v2](https://huggingface.co/SmilingWolf/wd-v1-4-convnext-tagger-v2/blob/main/model.onnx)
   - Rename the downloaded file to `model.onnx`.

2. **tags.csv**: The tags CSV file matching the model.
   - Recommended: [SmilingWolf/wd-v1-4-convnext-tagger-v2](https://huggingface.co/SmilingWolf/wd-v1-4-convnext-tagger-v2/blob/main/selected_tags.csv)
   - Rename the downloaded file to `tags.csv`.

**Note:** If these files are missing, the application will start but tagging functionality will use a fallback/mock response.
