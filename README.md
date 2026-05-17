# pyrevm-trace

Python bindings for fast EVM simulation and tracing, built on [REVM](https://github.com/bluealloy/revm) via PyO3.

Simulate transactions locally, capture call trees, and profile gas usage — no node required.

## Features

- **Transaction simulation** — execute calls against in-memory EVM state
- **Call tracing** — full recursive call tree with inputs, outputs, and success flags
- **Gas profiling** — opcode-level gas breakdown
- **Async support** — `AsyncSimulator` offloads to a thread so it won't block your event loop
- **Typed API** — Pydantic v2 models for all results

## Installation

Requires Python 3.10+ and Rust (for building from source).

```bash
uv add pyrevm-trace
# or
pip install pyrevm-trace
```

To build from source:

```bash
git clone https://github.com/salva-imm/pyrevm-trace
cd pyrevm-trace
uv run maturin develop --release
```

## Quick Start

```python
from pyrevm_trace import Simulator

sim = Simulator(chain_id=1)

# Set up state
sim.set_balance("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", 10**18)
sim.set_code("0x1234567890123456789012345678901234567890", bytes.fromhex("600160005260206000f3"))

# Simulate a call
result = sim.simulate(
    caller="0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    to="0x1234567890123456789012345678901234567890",
    calldata=b"",
    gas_limit=100_000,
)

print(result.success)    # True
print(result.gas_used)   # e.g. 21318
print(result.return_data.hex())  # 0000...0001
```

## API

### `Simulator`

Synchronous wrapper around `EVMSimulator`.

```python
from pyrevm_trace import Simulator

sim = Simulator(chain_id=1)
sim.set_balance(address: str, balance: int) -> None
sim.set_code(address: str, bytecode: bytes) -> None
sim.set_storage(address: str, slot: int, value: int) -> None

result: SimulationResult = sim.simulate(
    caller: str,
    to: str,
    calldata: bytes = b"",
    value: int = 0,
    gas_limit: int = 30_000_000,
    with_trace: bool = False,
    with_gas_profile: bool = False,
)
```

### `AsyncSimulator`

Drop-in async version — simulation runs in a thread via `asyncio.to_thread`.

```python
import asyncio
from pyrevm_trace import AsyncSimulator

async def main():
    sim = AsyncSimulator(chain_id=1)
    sim.set_code("0x1234...", bytecode)
    result = await sim.simulate(caller="0xabcd...", to="0x1234...", gas_limit=100_000)
    print(result.success)

asyncio.run(main())
```

### Call Tracing

```python
result = sim.simulate(..., with_trace=True)

if result.calls:
    print(result.calls.from_)      # caller address
    print(result.calls.to)         # target address
    print(result.calls.success)    # bool
    for sub in result.calls.calls: # nested calls
        ...
```

### Gas Profiling

```python
result = sim.simulate(..., with_gas_profile=True)

for step in result.gas_steps:
    print(f"pc={step.pc} op=0x{step.op:02x} cost={step.gas_cost}")
```

> `with_trace` takes priority over `with_gas_profile` if both are set.

## Models

```python
class SimulationResult(BaseModel):
    success: bool
    gas_used: int
    return_data: bytes
    logs: list[Log]
    calls: CallFrame | None          # present when with_trace=True
    gas_steps: list[GasStep] | None  # present when with_gas_profile=True

class Log(BaseModel):
    address: str
    topics: list[str]
    data: bytes

class CallFrame(BaseModel):
    from_: str      # "from" in raw dict
    to: str
    calldata: bytes
    value: str      # decimal string (U256)
    gas_limit: int
    success: bool
    output: bytes
    calls: list[CallFrame]

class GasStep(BaseModel):
    pc: int
    op: int
    gas_remaining: int
    gas_cost: int
```

## Development

```bash
# Build Rust extension
uv run maturin develop

# Run tests
uv run pytest

# Run Rust unit tests
cargo test
```

## License

MIT
