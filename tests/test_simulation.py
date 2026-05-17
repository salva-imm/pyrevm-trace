import pytest
from pyrevm_trace import EVMSimulator

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT = "0x1234567890123456789012345678901234567890"


def test_set_balance():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)


def test_set_code():
    sim = EVMSimulator(chain_id=1)
    # PUSH1 1, PUSH1 0, MSTORE, PUSH1 32, PUSH1 0, RETURN — returns uint256(1)
    sim.set_code(CONTRACT, bytes.fromhex("600160005260206000f3"))


def test_set_storage():
    sim = EVMSimulator(chain_id=1)
    sim.set_storage(CONTRACT, slot=0, value=42)


def test_invalid_address_raises():
    sim = EVMSimulator(chain_id=1)
    with pytest.raises(ValueError):
        sim.set_balance("not_an_address", 100)


def test_simulate_returns_value():
    sim = EVMSimulator(chain_id=1)
    # PUSH1 1, PUSH1 0, MSTORE, PUSH1 32, PUSH1 0, RETURN — returns uint256(1)
    sim.set_code(CONTRACT, bytes.fromhex("600160005260206000f3"))

    result = sim.simulate(
        caller=SENDER,
        to=CONTRACT,
        calldata=b"",
        value=0,
        gas_limit=100_000,
    )

    assert result["success"] is True
    assert result["gas_used"] > 0
    assert result["return_data"] == (1).to_bytes(32, "big")
    assert result["logs"] == []


def test_simulate_revert():
    sim = EVMSimulator(chain_id=1)
    # PUSH1 0, PUSH1 0, REVERT — always reverts with empty data
    sim.set_code(CONTRACT, bytes.fromhex("60006000fd"))

    result = sim.simulate(
        caller=SENDER,
        to=CONTRACT,
        calldata=b"",
        value=0,
        gas_limit=100_000,
    )

    assert result["success"] is False
    assert result["gas_used"] > 0


def test_simulate_call_to_eoa_with_value():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)

    result = sim.simulate(
        caller=SENDER,
        to=CONTRACT,
        calldata=b"",
        value=10**15,
        gas_limit=21_000,
    )

    assert result["success"] is True
