"""默认配置。"""

from dataclasses import dataclass


@dataclass
class Config:
    endpoint: str = "http://localhost:9090"
    timeout_ms: int = 3000


def load_default() -> Config:
    return Config()
