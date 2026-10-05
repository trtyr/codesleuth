"""带重试的 HTTP 抓取。"""

import urllib.error
import urllib.request

from .exceptions import HttpError, is_retryable
from .retry import TransientError, retry_with_backoff


@retry_with_backoff
def fetch_with_retry(url: str) -> str:
    """抓取 URL：可重试状态码抛 TransientError 走重试，其余抛 HttpError。"""
    try:
        with urllib.request.urlopen(url) as res:
            return res.read().decode("utf-8")
    except urllib.error.HTTPError as err:
        if is_retryable(err.code):
            raise TransientError(f"http {err.code}") from err
        raise HttpError(f"http {err.code}") from err
