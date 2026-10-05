"""带指数退避的重试装饰器——侦察目标：找到这个模块并引用 file:line。"""

from .backoff import delay_ms, sleep_ms

MAX_ATTEMPTS = 3


class TransientError(Exception):
    """可重试的瞬态错误标记：只有它会被重试。"""


class RetryError(Exception):
    """重试耗尽后抛出。"""


def retry_with_backoff(func):
    """装饰器：仅捕获 TransientError，按指数退避重试，最多 MAX_ATTEMPTS 次。"""

    def wrapper(*args, **kwargs):
        attempt = 0
        while True:
            try:
                return func(*args, **kwargs)
            except TransientError as err:
                if attempt >= MAX_ATTEMPTS:
                    raise RetryError(f"重试 {attempt} 次后放弃") from err
                sleep_ms(delay_ms(attempt))
                attempt += 1

    return wrapper
