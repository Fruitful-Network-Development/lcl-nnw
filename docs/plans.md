The biggest local-only wins left are mostly about deleting compatibility and archive weight, not adding more features.

1. Retire the gateway for real. `gateway/` is still a standalone crate with live code and tests, and it still defaults to `remote_frontier` in [gateway/src/config.rs](/home/smohn/LCL/lcl-nnw/gateway/src/config.rs:10). `model_registry/lanes/` is also still present in [model_registry/lanes/remote_frontier.toml](/home/smohn/LCL/lcl-nnw/model_registry/lanes/remote_frontier.toml:1) and [model_registry/lanes/local_cpu16.toml](/home/smohn/LCL/lcl-nnw/model_registry/lanes/local_cpu16.toml:1). If the cutover is real, move these under `archive/` or delete them.

2. Remove the old config/session compatibility if you do not need migration support. The new runtime still loads legacy `.claw.json` paths in [crates/runtime/src/config.rs](/home/smohn/LCL/lcl-nnw/crates/runtime/src/config.rs:267), and session lookup still scans legacy roots in [crates/runtime/src/session_control.rs](/home/smohn/LCL/lcl-nnw/crates/runtime/src/session_control.rs:125). That adds branches, tests, and failure modes you may no longer want.

3. Replace the env-based provider override with explicit client config. The weakest hardening point right now is [with_lcl_provider_env_overrides()](/home/smohn/LCL/lcl-nnw/crates/rusty-claude-cli/src/main.rs:6952), which temporarily mutates `OPENAI_BASE_URL` and `OPENAI_API_KEY` inside the process. It works, but it is still brittle. The cleaner fix is to pass `base_url` and `api_key` directly into the OpenAI-compatible client constructors.

4. Add end-to-end tests for the new `.claw` flow. You already have config parsing coverage, but the important gap is proving the whole CLI path: `.claw/settings.json` -> `doctor` / `status` / `prompt` / resume under `.claw/sessions` against a mock OpenAI-compatible server. That is the highest-value hardening after the provider refactor.

5. Remove Python from local ops. Both [runtimes/llama_cpp/start.sh](/home/smohn/LCL/lcl-nnw/runtimes/llama_cpp/start.sh:1) and [ops/scripts/smoke-test.sh](/home/smohn/LCL/lcl-nnw/ops/scripts/smoke-test.sh:1) require `python3` just to read merged JSON. A tiny `lcl config print --json` or `lcl config get` command would make startup cheaper and less fragile.

6. Delete the stale session directory. The new runtime uses `.claw/sessions`, but [data/sessions/.gitkeep](/home/smohn/LCL/lcl-nnw/data/sessions/.gitkeep:1) is still sitting around. That is small, but it is exactly the kind of repo noise that causes drift later.

7. Finish the rename cleanup. There are still a lot of `claw`-named helpers/messages/tests outside pure provenance contexts, for example in [crates/plugins/src/lib.rs](/home/smohn/LCL/lcl-nnw/crates/plugins/src/lib.rs:1625) and [crates/runtime/src/lib.rs](/home/smohn/LCL/lcl-nnw/crates/runtime/src/lib.rs:1). Keeping upstream provenance is fine; keeping mixed product naming everywhere else is just cognitive waste.

8. Audit lane/orchestrator leftovers. If your target product is truly a local CLI, files like [crates/runtime/src/lane_events.rs](/home/smohn/LCL/lcl-nnw/crates/runtime/src/lane_events.rs:1) and the lane-manifest logic inside [crates/tools/src/lib.rs](/home/smohn/LCL/lcl-nnw/crates/tools/src/lib.rs:3538) are worth reevaluating. They may be intentional, but they look like a likely second-pass trim.

9. Add a local regression guard. A simple test or script can fail the build if active code outside `archive/` reintroduces `remote_frontier`, `/v1/chat/completions`, `GET /health`, lane manifests, or `.env`-driven runtime config.

If I were sequencing this for minimum waste, I would do it in this order: remove `gateway/` + `model_registry/` + `data/sessions` remnants, remove legacy `.claw.json` and legacy session fallback, replace the env override with explicit provider config, then add CLI end-to-end tests and a small config-print command for the scripts. That gives you the biggest surface-area reduction first.