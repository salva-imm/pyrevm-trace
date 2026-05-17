from pyrevm_trace._pyrevm_trace import EVMSimulator
from pyrevm_trace.models import SimulationResult


class Simulator:
    def __init__(self, chain_id: int = 1) -> None:
        self._evm = EVMSimulator(chain_id=chain_id)

    def set_balance(self, address: str, balance: int) -> None:
        self._evm.set_balance(address, balance)

    def set_code(self, address: str, bytecode: bytes) -> None:
        self._evm.set_code(address, bytecode)

    def set_storage(self, address: str, slot: int, value: int) -> None:
        self._evm.set_storage(address, slot, value)

    def simulate(
        self,
        *,
        caller: str,
        to: str,
        calldata: bytes = b"",
        value: int = 0,
        gas_limit: int = 30_000_000,
        with_trace: bool = False,
        with_gas_profile: bool = False,
    ) -> SimulationResult:
        raw = self._evm.simulate(
            caller=caller,
            to=to,
            calldata=calldata,
            value=value,
            gas_limit=gas_limit,
            with_trace=with_trace,
            with_gas_profile=with_gas_profile,
        )
        return SimulationResult.model_validate(raw)
