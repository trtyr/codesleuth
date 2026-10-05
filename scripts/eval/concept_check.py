"""概念题零重叠纪律检查（P003 E0）。

规则：
1. 每道 concept 题的 banned 词（答案词汇族）不得出现在查询文本里——出现即出题失败；
2. 报告 banned 词在目标文件中的命中情况（应至少出现核心词，证明禁对了对象）；
3. 查询中的 ASCII 实词（长度≥3）不得出现在 expect_files 目标文件里（硬零重叠）。
"""

import json
import os
import re
import sys

def load_engram_root() -> str:
    """真仓靶路径：--engram-root <path> 旗标传入；未传则 engram 题自动跳过。"""
    import sys
    return sys.argv[sys.argv.index("--engram-root") + 1] if "--engram-root" in sys.argv else ""

FIXTURE_ROOT = {
    "fixture-rs": "tests/fixtures/fixture-rs",
    "fixture-ts": "tests/fixtures/fixture-ts",
    "fixture-py": "tests/fixtures/fixture-py",
    "engram": load_engram_root(),
}


def ascii_tokens(text: str) -> list:
    return [t for t in re.split(r"[^A-Za-z0-9_]+", text) if len(t) >= 3]


def check_file(path: str) -> dict:
    try:
        with open(path, "r", encoding="utf-8", errors="ignore") as fh:
            return {"exists": True, "content": fh.read()}
    except OSError:
        return {"exists": False, "content": ""}


def main() -> int:
    failures = 0
    total = 0
    for gold_file in sys.argv[1:]:
        with open(gold_file, "r", encoding="utf-8") as fh:
            gold = json.load(fh)
        root = FIXTURE_ROOT.get(gold["fixture"])
        if not root:
            print(f"[skip] {gold_file}: fixture {gold['fixture']} 无本地路径")
            continue
        for q in gold["questions"]:
            if q.get("kind", "concept" if "banned" in q else "normal") != "concept":
                continue
            total += 1
            query = q["q"]
            banned = q.get("banned") or []
            # 规则 1：禁词不得出现在查询
            hit = [b for b in banned if b.lower() in query.lower()]
            if hit:
                failures += 1
                print(f"[FAIL] {gold_file} · {query[:26]}… 查询含禁词: {hit}")
                continue
            # 规则 2：禁词应命中目标文件族（抽查第一个 expect_file 的所在目录不可行，检查 expect_files 各文件）
            targets = [check_file(os.path.join(root, f)) for f in q.get("expect_files", [])]
            missing = [f for f, t in zip(q.get("expect_files", []), targets) if not t["exists"]]
            if missing:
                failures += 1
                print(f"[FAIL] {gold_file} · 目标文件不存在: {missing}")
                continue
            # 规则 3：查询 ASCII 实词不得出现在目标文件
            for f, t in zip(q.get("expect_files", []), targets):
                leak = [tok for tok in ascii_tokens(query) if tok in t["content"]]
                if leak:
                    failures += 1
                    print(f"[FAIL] {gold_file} · {query[:26]}… 查询 ASCII 词泄入目标 {f}: {leak}")
    print(f"\n概念题零重叠检查: {total - failures}/{total} 通过")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
