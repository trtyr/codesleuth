# read 强读工具（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
read 强读工具是只读检索工具面中"唯一事实来源"的文件读取器：给调用方（LLM Agent）返回带行号+行哈希锚点的文件内容，支持 offset/limit 分页，并对二进制、空文件、越界、非 UTF-8 等情况做诚实反馈，防止读错内容。

## 证据列表
1. 功能定位：强读工具，MVP 六特性（锚点/分页/二进制探测/编码消毒/诚实反馈/结构化输出头），args 为 path + offset(1-based) + limit(默认200，上限2000)。
   - src/tools/read.rs:1-2, 50-52, 54-64（审计 #2）
   - src/tools/read.rs:25-29, 12-13（审计 #2）
2. 入口与关键文件：src/tools/read.rs（ReadTool 结构体、Tool trait 实现、execute 入口），由 src/cli.rs 启动时注册进 ToolRegistry；tests/adversarial.rs、tests/replay.rs 等测试也注册它。
   - src/tools/read.rs:15-22, 44-48, 66（审计 #2）
   - src/cli.rs:238-244（审计 #8）
   - tests/adversarial.rs:32-36（审计 #13）
   - tests/replay.rs:64-68（审计 #15）
3. 运作方式：CLI/Agent 调用注册表中的 read 工具 → ReadTool::execute 解析参数 → 经 Fence resolve 路径并读文件 → 逐行输出「行号:3位hex行哈希|内容」，截断时提示续读 offset；测试验证锚点+分页行为。
   - src/tools/read.rs:66-146（审计 #2）
   - src/tools/read.rs:86, 93（审计 #2）
   - src/tools/read.rs:158-172（审计 #2）
4. 交互模块：上游依赖 crate::errors（USER_INPUT/REPO_NOT_READABLE 错误码）与 crate::fence::Fence（路径安全解析）；自身实现 tools::Tool trait，与 fuzzy 层 FileFinderTool/GrepTool 同注册表构成检索工具面；内建 sha2 行哈希、二进制启发探测、UTF-8 lossy 消毒。
   - src/tools/read.rs:4-6, 16, 86（审计 #2）
   - src/tools/read.rs:32-42, 96-107（审计 #2）
   - src/cli.rs:240-244（审计 #8）

## 死胡同
- 无；仅做概览，未展开 read.rs 测试细节（201-228 行）

## 置信度
high

## 统计
turns=6 · tool_calls=5 · duration=24463ms · tokens=34394
