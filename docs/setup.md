# Set up (Populate `lcl-nnw` correctly)

Your repo is already the scaffold. The root `README.md` says `third_party/claw-code/` is the shell surface, `gateway/` is the Rust orchestration layer, `runtimes/llama_cpp/` is the initial backend target, and `data/` is for runtime-generated state.  The repo also already has a bootstrap script in `ops/scripts/bootstrap.sh` and `.gitignore` is already set to ignore `data/models/*`, `*.gguf`, `*.safetensors`, and other large artifacts, so model downloads should stay local and out of Git.  

So the correct way to “populate” `lcl-nnw` is:

1. keep `lcl-nnw` as the root,
2. add the upstream code repos as submodules,
3. commit those submodule pointers to `main`,
4. download models only into `data/models/` locally, not into Git.

Use this exact path.

First, from the `lcl-nnw` repo root, remove any placeholder files in the target folders so submodules can be added cleanly.

```bash
# run from the lcl-nnw repo root
cd ~/LCL/lcl-nnw

find third_party runtimes -type f \( -name '.gitkeep' -o -name 'README.placeholder' \) -print -delete || true
```

Now add the three upstream repos as submodules. This is the correct way to make the repo “contain” them without mangling your parent Git history with raw nested clones.

```bash
# run from the lcl-nnw repo root
cd ~/LCL/lcl-nnw

git submodule add git@github.com:ultraworkers/claw-code.git third_party/claw-code
git submodule add git@github.com:ggml-org/llama.cpp.git runtimes/llama_cpp/llama.cpp
git submodule add git@github.com:huggingface/sentence-transformers.git runtimes/embeddings/sentence-transformers
```

Then verify what changed.

```bash
# run from the lcl-nnw repo root
cd ~/LCL/lcl-nnw

git submodule status
git status
```

Commit and push those repo-populating changes to `main`.

```bash
# run from the lcl-nnw repo root
cd ~/LCL/lcl-nnw

git add .gitmodules third_party/claw-code runtimes/llama_cpp/llama.cpp runtimes/embeddings/sentence-transformers
git commit -m "Add v1 upstream submodules"
git push origin main
```

After that, bootstrap the local runtime directories that are meant to stay local.

```bash
# run from the lcl-nnw repo root
cd ~/LCL/lcl-nnw

bash ops/scripts/bootstrap.sh
cp runtimes/llama_cpp/.env.example runtimes/llama_cpp/.env 2>/dev/null || true
```

Install the local tools needed for model download and build.

```bash
# run from your home directory or anywhere
cd ~

sudo apt update
sudo apt install -y build-essential cmake pkg-config python3 python3-pip
python3 -m pip install -U "huggingface_hub[cli]"
```

Now download the models into the already-ignored `data/models/` tree. These files will remain local and will not be pushed.

```bash
# run from the lcl-nnw repo root
cd ~/LCL/lcl-nnw

huggingface-cli download bartowski/Qwen2.5-Coder-7B-Instruct-GGUF \
  --include "Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf" \
  --local-dir data/models/qwen2.5-coder-7b-instruct-q4_k_m

huggingface-cli download bartowski/DeepSeek-R1-Distill-Qwen-7B-GGUF \
  --include "DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf" \
  --local-dir data/models/deepseek-r1-distill-qwen-7b-q4_k_m

huggingface-cli download BAAI/bge-small-en-v1.5 \
  --local-dir data/models/bge-small-en-v1.5
```

Then build `llama.cpp` from the submodule location that now lives inside your repo.

```bash
# run from the llama.cpp submodule directory
cd ~/LCL/lcl-nnw/runtimes/llama_cpp/llama.cpp

cmake -B build
cmake --build build -j"$(nproc)"
```

Then start the server against your lead model.

```bash
# run from the llama.cpp submodule directory
cd ~/LCL/lcl-nnw/runtimes/llama_cpp/llama.cpp

./build/bin/llama-server \
  -m ~/LCL/lcl-nnw/data/models/qwen2.5-coder-7b-instruct-q4_k_m/Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf \
  --host 127.0.0.1 \
  --port 8080
```

Then validate the Rust gateway still compiles from the repo scaffold.

```bash
# run from the lcl-nnw repo root
cd ~/LCL/lcl-nnw/gateway

cargo check
```

The important rule is this: commit the submodules, do not commit the model files. Your repo is already structured for that pattern. The only thing you should push to `main` from this step is the `.gitmodules` file plus the three submodule entries.

For any future clone of `lcl-nnw`, initialize everything with:

```bash
# run from the parent directory where you want the repo
cd ~/LCL

git clone git@github.com:Fruitful-Network-Development/lcl-nnw.git
cd lcl-nnw
git submodule update --init --recursive
```

Run this next and paste the output if anything fails:

```bash
# run from the lcl-nnw repo root
cd ~/LCL/lcl-nnw

git submodule add git@github.com:ultraworkers/claw-code.git third_party/claw-code
```

