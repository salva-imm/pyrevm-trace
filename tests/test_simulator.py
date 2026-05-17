import pytest
from pyrevm_trace import SimulationResult
from pyrevm_trace.simulator import Simulator

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT = "0x1234567890123456789012345678901234567890"
RETURN_ONE = bytes.fromhex("600160005260206000f3")  # returns uint256(1)
ALWAYS_REVERT = bytes.fromhex("60006000fd")


def test_simulate_returns_model():
    sim = Simulator(chain_id=1)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000)

    assert isinstance(result, SimulationResult)
    assert result.success is True
    assert result.return_data == (1).to_bytes(32, "big")


def test_simulate_revert_returns_model():
    sim = Simulator(chain_id=1)
    sim.set_code(CONTRACT, ALWAYS_REVERT)

    result = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000)

    assert isinstance(result, SimulationResult)
    assert result.success is False


def test_simulate_with_trace_returns_call_frame():
    sim = Simulator(chain_id=1)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000, with_trace=True)

    assert isinstance(result, SimulationResult)
    assert result.calls is not None
    assert result.calls.to.lower() == CONTRACT.lower()
    assert result.calls.success is True


def test_set_balance_preserves_across_calls():
    sim = Simulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)

    r1 = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=10**15, gas_limit=21_000)
    r2 = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=10**15, gas_limit=21_000)

    assert r1.success is True
    assert r2.success is True
