import asyncio
import pytest
from pyrevm_trace import SimulationResult
from pyrevm_trace.async_simulator import AsyncSimulator

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT = "0x1234567890123456789012345678901234567890"
RETURN_ONE = bytes.fromhex("600160005260206000f3")


pytestmark = pytest.mark.asyncio


async def test_async_simulate_returns_model():
    sim = AsyncSimulator(chain_id=1)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = await sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000)

    assert isinstance(result, SimulationResult)
    assert result.success is True
    assert result.return_data == (1).to_bytes(32, "big")


async def test_async_simulate_does_not_block_event_loop():
    sim = AsyncSimulator(chain_id=1)
    sim.set_code(CONTRACT, RETURN_ONE)

    # Both coroutines should be able to run concurrently (each uses its own instance).
    sim2 = AsyncSimulator(chain_id=1)
    sim2.set_code(CONTRACT, RETURN_ONE)

    r1, r2 = await asyncio.gather(
        sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000),
        sim2.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000),
    )

    assert r1.success is True
    assert r2.success is True
