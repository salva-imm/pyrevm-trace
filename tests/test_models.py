import pytest
from pyrevm_trace import EVMSimulator
from pyrevm_trace.models import SimulationResult, Log

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT = "0x1234567890123456789012345678901234567890"


def test_simulation_result_success():
    sim = EVMSimulator(chain_id=1)
    # PUSH1 1, PUSH1 0, MSTORE, PUSH1 32, PUSH1 0, RETURN — returns uint256(1)
    sim.set_code(CONTRACT, bytes.fromhex("600160005260206000f3"))

    raw = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000)
    result = SimulationResult.model_validate(raw)

    assert result.success is True
    assert result.gas_used > 0
    assert result.return_data == (1).to_bytes(32, "big")
    assert result.logs == []


def test_simulation_result_revert():
    sim = EVMSimulator(chain_id=1)
    # PUSH1 0, PUSH1 0, REVERT
    sim.set_code(CONTRACT, bytes.fromhex("60006000fd"))

    raw = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000)
    result = SimulationResult.model_validate(raw)

    assert result.success is False
    assert result.gas_used > 0


def test_log_model():
    # LOG1: emit one topic. Bytecode:
    # PUSH1 0x42 (data), PUSH1 0, MSTORE   → store 0x42 at mem[0]
    # PUSH32 <topic>                         → push topic onto stack
    # PUSH1 0x20 (size=32), PUSH1 0 (off)   → log region
    # LOG1
    topic = "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
    topic_bytes = bytes.fromhex(topic[2:])

    # Simple bytecode: PUSH32 topic, PUSH1 32, PUSH1 0, LOG1
    bytecode = (
        b"\x7f" + topic_bytes   # PUSH32 topic
        + b"\x60\x20"           # PUSH1 32 (size)
        + b"\x60\x00"           # PUSH1 0 (offset)
        + b"\xa1"               # LOG1
        + b"\x60\x00\x60\x00\xf3"  # PUSH1 0, PUSH1 0, RETURN
    )
    sim = EVMSimulator(chain_id=1)
    sim.set_code(CONTRACT, bytecode)

    raw = sim.simulate(caller=SENDER, to=CONTRACT, calldata=b"", value=0, gas_limit=100_000)
    result = SimulationResult.model_validate(raw)

    assert result.success is True
    assert len(result.logs) == 1
    log = result.logs[0]
    assert isinstance(log, Log)
    assert log.address.lower() == CONTRACT.lower()
    assert len(log.topics) == 1
    assert log.topics[0] == topic
