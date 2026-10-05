"""HTTP 状态码分类。"""

from .retry import TransientError

RETRYABLE_STATUS = {429, 500, 502, 503, 504}


class HttpError(Exception):
    """不可重试的 HTTP 错误。"""


def is_retryable(status: int) -> bool:
    """429 与集合内的 5xx 视为可重试。"""
    return status in RETRYABLE_STATUS
