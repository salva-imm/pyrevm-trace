from __future__ import annotations
from pydantic import BaseModel, Field


class GasStep(BaseModel):
    pc: int
    op: int
    gas_remaining: int
    gas_cost: int


class Log(BaseModel):
    address: str
    topics: list[str]
    data: bytes

    model_config = {"arbitrary_types_allowed": True}


class CallFrame(BaseModel):
    from_: str = Field(alias="from")
    to: str
    calldata: bytes
    value: str
    gas_limit: int
    success: bool
    output: bytes
    calls: list[CallFrame] = []

    model_config = {"arbitrary_types_allowed": True, "populate_by_name": True}


class SimulationResult(BaseModel):
    success: bool
    gas_used: int
    return_data: bytes
    logs: list[Log]
    calls: CallFrame | None = None
    gas_steps: list[GasStep] | None = None

    model_config = {"arbitrary_types_allowed": True}
