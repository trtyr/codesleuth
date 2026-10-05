"""命令行入口：抓取默认 endpoint。"""

from .config import load_default
from .http import fetch_with_retry


def main() -> None:
    cfg = load_default()
    body = fetch_with_retry(cfg.endpoint)
    print(f"cfg: {cfg}")
    print(f"body: {body[:120]}")


if __name__ == "__main__":
    main()
