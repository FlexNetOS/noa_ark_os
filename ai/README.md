# AI Engine

AI models, inference, and llama.cpp integration.

## Components

- **Model Management**: Loading, unloading, lifecycle
- **Llama.cpp Integration**: LLM inference engine
- **Inference Service**: API for model inference
- **Model Storage**: Model weights and configurations

## Structure

```
ai/
├── models/         # Model management
├── llama/          # Llama.cpp integration
├── inference/      # Inference engine
└── serving/        # Model serving API
```

## Self-Contained Approach

All AI models and the llama.cpp library will be bundled with the OS.
No external API calls or dependencies.

## Supported Models

- LLaMA family models
- Custom fine-tuned models
- Embedding models
- Vision models (future)
