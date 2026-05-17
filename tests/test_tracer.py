from pyrevm_trace import EVMSimulator

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT_A = "0x1111111111111111111111111111111111111111"
CONTRACT_B = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"

RETURN_ONE = bytes.fromhex("600160005260206000f3")

# Contract A calls Contract B with no calldata/value, then STOPs.
# Stack layout for CALL: gas, addr, value, argsOff, argsLen, retOff, retLen
# PUSH1 0 × 5, PUSH20 <CONTRACT_B>, GAS, CALL, STOP
CALL_B = bytes.fromhex(
    "6000"              # PUSH1 0  (retLen)
    "6000"              # PUSH1 0  (retOff)
    "6000"              # PUSH1 0  (argsLen)
    "6000"              # PUSH1 0  (argsOff)
    "6000"              # PUSH1 0  (value)
    "73" + "bb" * 20   # PUSH20 CONTRACT_B
    + "5a"             # GAS
    + "f1"             # CALL
    + "00"             # STOP
)


def test_trace_top_level_call():
    sim = EVMSimulator(chain_id=1)
    sim.set_code(CONTRACT_A, RETURN_ONE)

    result = sim.simulate(
        caller=SENDER,
        to=CONTRACT_A,
        calldata=b"",
        value=0,
        gas_limit=100_000,
        with_trace=True,
    )

    assert result["success"] is True
    frame = result["calls"]
    assert frame["from"].lower() == SENDER.lower()
    assert frame["to"].lower() == CONTRACT_A.lower()
    assert frame["success"] is True
    assert frame["calls"] == []


def test_trace_nested_call():
    sim = EVMSimulator(chain_id=1)
    sim.set_code(CONTRACT_B, RETURN_ONE)
    sim.set_code(CONTRACT_A, CALL_B)

    result = sim.simulate(
        caller=SENDER,
        to=CONTRACT_A,
        calldata=b"",
        value=0,
        gas_limit=100_000,
        with_trace=True,
    )

    assert result["success"] is True
    root = result["calls"]
    assert root["to"].lower() == CONTRACT_A.lower()
    assert len(root["calls"]) == 1
    sub = root["calls"][0]
    assert sub["from"].lower() == CONTRACT_A.lower()
    assert sub["to"].lower() == CONTRACT_B.lower()
    assert sub["success"] is True


def test_no_trace_by_default():
    sim = EVMSimulator(chain_id=1)
    sim.set_code(CONTRACT_A, RETURN_ONE)

    result = sim.simulate(
        caller=SENDER,
        to=CONTRACT_A,
        calldata=b"",
        value=0,
        gas_limit=100_000,
    )

    assert "calls" not in result
