#!/usr/bin/env python3
"""金标命中率评估（需真网关 + 已构建的二进制）。

用法:
  （密钥/端点/模型读 ~/.codesleuth/config.toml）\
    python3 scripts/eval.py tests/golden/fixture-rs.json tests/fixtures/fixture-rs
"""
import json
import os
import subprocess
import sys

BIN = os.path.join(os.path.dirname(__file__), "..", "target", "debug", "codesleuth")


def main() -> int:
    golden_path, fixture = sys.argv[1], sys.argv[2]
    import tomllib
    from pathlib import Path
    cfg_p = Path.home() / ".codesleuth" / "config.toml"
    cfg = tomllib.loads(cfg_p.read_text(encoding="utf-8")).get("llm", {}) if cfg_p.exists() else {}
    if not cfg.get("api_key"):
        print("需要 ~/.codesleuth/config.toml 的 [llm] api_key", file=sys.stderr)
        return 2
    if not os.path.exists(BIN):
        print("先 cargo build", file=sys.stderr)
        return 2
    golden = json.load(open(golden_path))
    total = hit = 0
    for q in golden["questions"]:
        total += 1
        r = subprocess.run(
            [BIN, q["q"], "--repo", fixture, "--json"],
            capture_output=True,
            text=True,
            timeout=180,
        )
        try:
            report = json.loads(r.stdout)
        except Exception:
            print(f"MISS {q['q']}（stdout 非法 JSON）")
            continue
        text = (
            report.get("answer", "")
            + " ".join(f.get("statement", "") for f in report.get("findings", []))
            + " ".join(
                e.get("file", "")
                for f in report.get("findings", [])
                for e in f.get("evidence", [])
            )
        )
        ok = any(x in text for x in q["expect_files"])
        print(("HIT " if ok else "MISS ") + q["q"])
        hit += 1 if ok else 0
    rate = f"{hit}/{total}"
    print(f"=== 命中率: {rate} ===")
    return 0 if hit == total else 1


if __name__ == "__main__":
    sys.exit(main())
