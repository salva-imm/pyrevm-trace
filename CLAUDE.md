# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

This is a hybrid Rust/Python project. **Always use `uv run` for Python commands** — the project was created with `uv`.

```bash
# Build the Rust extension (required after any change to src/)
uv run maturin develop

# Run Python tests
uv run pytest

# Run a single test
uv run pytest tests/test_simulation.py::test_set_balance -v

# Run Rust unit tests (no Python involved)
cargo test

# Run Rust tests for a specific module
cargo test types
```

## Architecture

The project has two distinct layers that must be kept in sync.

### Rust layer (`src/`)

PyO3 extension compiled to `python/pyrevm_trace/_pyrevm_trace.cpython-*.so` by maturin.

- **`src/lib.rs`** — PyO3 module entry point. Registers all Python-visible classes. Only touch this to add new `#[pyclass]` types.
- **`src/executor.rs`** — `EVMSimulator` `#[pyclass]`. Owns the `CacheDB<EmptyDB>` (in-memory EVM state) and all `#[pymethods]`. This is where Python ↔ REVM conversion happens: extract Python types → call REVM → convert result to Python dicts.
- **`src/types.rs`** — Stateless conversion helpers (`parse_address`, `address_to_hex`, `py_int_to_u256`). All Rust modules that touch Python-provided addresses or amounts use these.
- **`src/tracer.rs`** — `CallTracer` implementing REVM's `Inspector` trait for call tree capture (planned).
- **`src/gas_profiler.rs`** — `GasProfiler` implementing `Inspector` for opcode-level gas steps (planned).

Data flows one way: Python dict → Rust extraction → REVM execution → result as Python dict. No Pydantic or serde on the Rust side.

### Python layer (`python/pyrevm_trace/`)

Pure Python, depends on the compiled `.so`. Planned modules:
- `models.py` — Pydantic v2 models wrapping the raw dicts from Rust (`SimulationResult`, `CallFrame`, `GasStep`, `Log`)
- `simulator.py` — `Simulator` class: typed sync wrapper
- `async_simulator.py` — `AsyncSimulator`: offloads simulation to `asyncio.to_thread` for non-blocking async use

### Key invariants

- The Rust layer returns **raw Python dicts** from `simulate()`. The Python layer wraps them in Pydantic models. Never add serde/JSON serialization on the Rust side.
- Each `EVMSimulator` instance owns independent state — safe for concurrent use across threads/tasks when each caller has its own instance.
- `with_trace=True` runs a `CallTracer` inspector; `with_gas_profile=True` runs `GasProfiler`. Both flags together: only trace is active (combined inspector is future work).

## Development Setup

`.cargo/config.toml` is gitignored (machine-specific Python paths for `cargo test`). Copy `.cargo/config.toml.example` and set `PYO3_PYTHON` to your Python 3.12 interpreter path, or export it as an env var before running `cargo test`.

## Commit Style

Plain commit messages only — no `Co-Authored-By` trailers.
