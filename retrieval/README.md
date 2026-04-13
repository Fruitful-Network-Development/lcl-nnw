# retrieval skeleton

Minimal retrieval layout for future indexing work.

## Intended flow
1. `retrieval/pipelines/` pulls raw sources and normalizes text.
2. `retrieval/chunks/` stores chunking strategy definitions and chunk outputs.
3. `retrieval/index/` stores index build scripts/config and metadata.
4. Embeddings are written to `data/vector_store/`.

This is intentionally scaffold-only for now.
