# embeddings runtime scaffold

This folder documents the embedding backend integration point used by retrieval.

## Intent
- Keep embedding service wiring separate from generation runtime.
- Allow swapping between local embedding servers later.
- Store vectors under `data/vector_store/`.

## Startup
```bash
bash runtimes/embeddings/start.sh
```
