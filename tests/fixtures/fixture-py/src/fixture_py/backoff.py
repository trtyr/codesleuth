"""退避延迟计算。"""

import time

BASE_DELAY_MS = 200


def delay_ms(attempt: int) -> int:
    """第 attempt 次重试前的等待毫秒数：基数 200ms 按指数翻倍。"""
    return BASE_DELAY_MS * (2 ** attempt)


def sleep_ms(ms: int) -> None:
    time.sleep(ms / 1000)
