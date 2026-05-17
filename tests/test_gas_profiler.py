import pytest
from pyrevm_trace import SimulationResult
from pyrevm_trace.simulator import Simulator

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT = "0x1234567890123456789012345678901234567890"
RETURN_ONE = bytes.fromhex("600160005260206000f3")  # returns uint256(1)


def test_gas_profile_returns_steps():
    sim = Simulator(chain_id=1)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000, with_gas_profile=True)

    assert isinstance(result, SimulationResult)
    assert result.gas_steps is not None
    assert len(result.gas_steps) > 0


def test_gas_step_fields():
    sim = Simulator(chain_id=1)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000, with_gas_profile=True)

    step = result.gas_steps[0]
    assert isinstance(step.pc, int)
    assert isinstance(step.op, int)
    assert isinstance(step.gas_remaining, int)
    assert isinstance(step.gas_cost, int)


def test_gas_steps_absent_by_default():
    sim = Simulator(chain_id=1)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000)

    assert result.gas_steps is None
