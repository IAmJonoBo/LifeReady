# ARTIFACTS (excluded from the review pack)

This repo uses `.gitattributes` + `export-ignore` so `git archive` exports a small, review-friendly ZIP.

## What is typically excluded

- Dependencies (`node_modules/`, `.venv/`, etc.)
- Build outputs (`dist/`, `build/`, `.next/`, `target/`, etc.)
- Large data / models (`data/`, `datasets/`, `models/`, `*.gguf`, `*.safetensors`, ...)
- Local secrets / env (`.env*`, `*.pem`, `*.key`, ...)

## How to share excluded artefacts (when needed)

Create a separate “artefact pack” and share it only when asked:

- `models-pack.zip` (models/checkpoints)
- `data-pack.zip` (datasets/testdata if large)
- `build-pack.zip` (if reproducing a binary-only issue)

Add links below (Drive/Release/S3), plus checksums.

### Artefact packs

- models-pack.zip: link sha256: hash
- data-pack.zip: link sha256: hash

### Notes

- If you use Git LFS, decide separately whether LFS objects should be included in GitHub-generated archives:
  <https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/managing-git-lfs-objects-in-archives-of-your-repository>
