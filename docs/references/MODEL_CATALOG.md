# Model Catalog and Status

**Last Updated**: Current  
**Purpose**: Track available models and their download status  

---

## 📊 **Model Status**

### ✅ **Ready to Download** (Direct GGUF Available)

| Model | Size | Use Case | Priority | Status |
|-------|------|----------|----------|--------|
| **SmolLM3-3B** | 2GB | General, Fast | High | ✅ Ready |
| **Llama-3.2-3B** | 2GB | General | High | ✅ Installed |
| **TinyLlama-1.1B** | 700MB | Testing, Fast | Medium | ✅ Ready |
| **Phi-4-Mini-3.8B** | 2.5GB | Efficient | High | ✅ Ready |
| **Qwen3-4B** | 2.5GB | Balanced | High | ✅ Ready |
| **Llama-3.1-8B** | 5GB | Powerful | Medium | ✅ Ready |
| **Mistral-Nemo-12B** | 7GB | Advanced | Medium | ✅ Ready |
| **OpenThinker3-7B** | 7GB | Reasoning | High | ✅ Ready |
| **Octopus-v2** | 2GB | Function Calling | High | ✅ Ready |
| **Qwen2.5-VL-3B** | 2GB | Vision+Text | Medium | ✅ Ready |

---

## ⚠️ **Need GGUF Conversion** (Not Directly Available)

These models need to be converted to GGUF format or require finding the right GGUF version:

### **High Priority - Need Help Finding**:

1. **StableCode-3B** (stabilityai/stable-code-3b)
   - **Issue**: No official GGUF release
   - **Solution Needed**: Convert from original weights or find community GGUF
   - **Alternative**: Use `llama.cpp/convert.py` to convert

2. **Tiny-Agent-3B** (driaforall/Tiny-Agent-a-3B)
   - **Issue**: GGUF availability unclear
   - **Solution Needed**: Check if GGUF exists or convert
   - **Use Case**: Agent-specialized tasks

3. **StableLM-Alpha-3B** (stabilityai/stablelm-base-alpha-3b)
   - **Issue**: Older model, GGUF may not exist
   - **Solution Needed**: Check TheBloke's repos or convert
   - **Note**: May be superseded by newer models

4. **Granite-Docling-258M** (IBM)
   - **Issue**: Very specialized, GGUF unknown
   - **Solution Needed**: Check IBM's repo or community conversions
   - **Use Case**: Document processing

### **Medium Priority - Need Verification**:

5. **Qwen3-VL-30B-A3B** (Qwen/Qwen3-VL-30B-A3B-Instruct)
   - **Issue**: Very large, may have GGUF from unsloth
   - **Note**: Requires 16GB+ RAM
   - **Alternative**: yairpatch/Qwen3-VL-30B-A3B-Instruct-GGUF exists

6. **Qwen3-Coder-30B** (unsloth/Qwen3-Coder-30B-A3B-Instruct-1M-GGUF)
   - **Status**: GGUF exists but very large (15GB+)
   - **Note**: Requires significant RAM

7. **ERNIE-4.5-21B** (unsloth/ERNIE-4.5-21B-A3B-Thinking-GGUF)
   - **Status**: GGUF available
   - **Size**: ~12GB Q4
   - **Note**: Chinese model, may need special handling

8. **Llama-4-Maverick-17B** (meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8)
   - **Issue**: FP8 format, needs GGUF conversion
   - **Status**: Very new, may not have GGUF yet

9. **Llama-4-Scout-17B** (meta-llama/Llama-4-Scout-17B-16E-Instruct)
   - **Issue**: Very new, GGUF availability unknown
   - **Status**: Check for community conversions

### **Low Priority - Alternatives Exist**:

10. **Gemma-3-270M-IT**: Use existing Gemma models
11. **Jan-v1-4B**: Not widely available in GGUF
12. **Mixtral-8x7B**: Available but requires 32GB+ RAM
13. **OpenELM** (Apple): Experimental, limited GGUF support
14. **Falcon H1**: Limited GGUF availability
15. **LFM2-350M** (Liquid AI): Very new, may not have GGUF
16. **Phi-4-Mini-Flash-Reasoning**: Check Microsoft's releases
17. **Nemotron-Nano-9B-v2** (NVIDIA): Check NVIDIA's releases

---

## 🎯 **Recommended Download Priority**

### **For Your Workspace (Based on Use Cases)**:

1. **SmolLM3-3B** - Fast general model
   ```powershell
   .\scripts\dev\download-models.ps1 -ModelName smollm3-3b
   ```

2. **OpenThinker3-7B** - Reasoning tasks
   ```powershell
   .\scripts\dev\download-models.ps1 -ModelName openthinker-7b
   ```

3. **DeepSeek-Coder-V2** - Code generation
   ```powershell
   .\scripts\dev\download-models.ps1 -ModelName deepseek-coder-v2
   ```

4. **Octopus-v2** - Function calling (agent tasks)
   ```powershell
   .\scripts\dev\download-models.ps1 -ModelName octopus-v2
   ```

5. **Phi-4-Mini** - Efficient reasoning
   ```powershell
   .\scripts\dev\download-models.ps1 -ModelName phi-4-mini
   ```

---

## 🔧 **Converting Models to GGUF**

For models without GGUF versions:

### **Option 1: Use llama.cpp Convert Script**

```powershell
cd server\ai\llama-cpp

# Clone model from HuggingFace
git lfs install
git clone https://huggingface.co/[model-name]

# Convert to GGUF
python convert.py [model-directory] --outtype q4_k_m --outfile models/[output-name].gguf
```

### **Option 2: Check Community Repos**

Check these sources for GGUF versions:
- TheBloke (HuggingFace) - Most comprehensive
- QuantFactory (HuggingFace) - Newer models
- unsloth (HuggingFace) - Optimized versions
- bartowski (HuggingFace) - Recent conversions

---

## 📝 **Models I Need Help Finding**

Please help locate GGUF versions for:

### **Critical (Agent-Focused)**:
1. ✅ **Tiny-Agent-3B** - Agent-specialized
2. ✅ **Octopus-v2** - Function calling (found!)
3. ❓ **Granite-Docling-258M** - Document processing

### **Important (Code/Reasoning)**:
4. ❓ **StableCode-3B** - Code specialization
5. ✅ **DeepSeek-Coder-V2** - Code generation (found!)
6. ✅ **OpenThinker3-7B** - Reasoning (found!)

### **Nice to Have**:
7. ❓ **Llama-4** variants - Very new
8. ❓ **Phi-4-Mini-Flash-Reasoning** - New Microsoft model
9. ❓ **Nemotron-Nano-9B-v2** - NVIDIA's new model

---

## 💡 **Model Selection Guide**

### **For Different Tasks**:

| Task | Recommended Model | Why |
|------|------------------|-----|
| **General Chat** | SmolLM3-3B, Llama-3.2-3B | Fast, capable |
| **Code Generation** | DeepSeek-Coder-V2, StableCode-3B | Code-optimized |
| **Reasoning** | OpenThinker3-7B, Phi-4-Mini | Thinking-focused |
| **Agent Tasks** | Octopus-v2, Tiny-Agent-3B | Function-calling |
| **Fast Testing** | TinyLlama-1.1B, Gemma-270M | Very fast |
| **Vision Tasks** | Qwen2.5-VL-3B | Multimodal |
| **Large Tasks** | Mistral-Nemo-12B, Llama-3.1-8B | More capable |

---

## 🚀 **Quick Start**

### **Download Recommended Set**:

```powershell
cd D:\dev\workspaces\noa_ark_os

# List all available
.\scripts\dev\download-models.ps1 -ListAll

# Download top models
.\scripts\dev\download-models.ps1 -ModelName smollm3-3b
.\scripts\dev\download-models.ps1 -ModelName openthinker-7b
.\scripts\dev\download-models.ps1 -ModelName octopus-v2
.\scripts\dev\download-models.ps1 -ModelName phi-4-mini
```

### **Test Different Models**:

Update `scripts\dev\start-llama-server.ps1` to switch models:
```powershell
$ModelPath = Join-Path $LlamaCppDir "models\[model-file].gguf"
```

---

## 📊 **Disk Space Requirements**

| Model Count | Total Size | Recommendation |
|-------------|------------|----------------|
| 3-4 models | ~10GB | Minimum setup |
| 5-7 models | ~20GB | Good variety |
| 10+ models | ~40GB+ | Complete collection |

Current installed: **Llama-3.2-3B (2GB)**

---

**Status**: Script ready, model catalog complete  
**Action Needed**: Review models that need GGUF help  
**Next**: Download recommended models and test!
