# 微观 · Fence路径围栏（fence）

> 置信度 high · 8 条发现 · 182307 tokens

Fence 路径围栏链路（src/fence.rs → cli.rs:279/281 装配 Arc<Fence> → ReadTool/FuzzyEngine 消费）总体实现正确：词汇层预检 + dunce::canonicalize 解 symlink 双关卡、CS3003/exit 4 错误码语义、CLI 装配与工具注入契约均一致，tests/adversarial.rs 敌对三断言（symlink/绝对路径/../ 逃逸）逐条心算通过。发现一个真实但当前不可达的前缀判定缺陷：starts_with_root 用 zip 逐组件比较，较短一侧耗尽即空真返回 true，把「root 是 p 的前缀」误当成「p 以 root 为前缀」；由于两处调用点（fence.rs:37/48）的被检路径都由 root.join(rel) 派生、组件数必然 ≥ root，该缺陷在现有链路上不可达，属潜伏缺陷（P3），但它是围栏唯一前缀判定的共享工具函数，任何新调用点直接踩雷，建议改为先比较组件数再 zip。死代码维度：确定死 0 项、疑似死 0 项（4012/4013/5099 预留常量已主动清理并注释登记；Fence::root() 仍被 FuzzyEngine 消费）。最值得先修：src/fence.rs:76 的 zip 截断前缀判定。

1. starts_with_root 用 p.components().zip(root.components()).all(...) 做前缀判定：zip 在任一侧耗尽即停止，较短一侧完全决定结果——当 p 组件数 < root 组件数且前若干组件相同时空真返回 true，把「p 以 root 为前缀」的语义放宽成「root 以 p 为前缀」也放行（P3，当前不可达的潜伏缺陷）（src/fence.rs:75-88）
2. 该缺陷在现有两条调用路径不可达：两处判定（词汇层 fence.rs:37 与 canonicalize 复检 fence.rs:48）的被检路径均由 self.root.join(rel) 派生（fence.rs:31），组件数必然 ≥ root，join 保证了长度关系；但该函数是围栏唯一前缀判定的共享工具函数，任何新调用点直接触发（src/fence.rs:30-55）
3. normalize 的 ParentDir => out.pop() 在 pop 到空时产生空路径，空路径经 starts_with_root 的 zip 空真同样通过——与上一条同根的隐含假设（第一关候选必然 ≥ root）（src/fence.rs:59-71）
4. 错误码契约三方一致：不存在路径 → REPO_NOT_FOUND(CS3001)、逃逸 → FENCE_DENIED(CS3003)、CS3xxx → exit 4，与 fence.rs 内联测试及 adversarial.rs 敌对断言吻合；symlink 逃逸由第二道 canonicalize 复检拦住（fence.rs:48）（src/fence.rs:43-47、src/errors.rs:44-46、tests/adversarial.rs:142-161）
5. CLI 装配契约正确：cli.rs:241 先 canonicalize repo，cli.rs:279-281 建 Arc<Fence> 注入 ReadTool；FuzzyEngine 不再持 Fence，仅用 fence.root() 定界 picker（fuzzy.rs:24-27），且 FileFinderTool/GrepTool 不接受路径参数，无围栏绕过面（src/cli.rs:241-248、src/cli.rs:279-284、src/tools/fuzzy.rs:23-36）
6. ReadTool 全数据流核对干净：path 经 fence.resolve（read.rs:86），is_file/二进制/空文件/offset 越界诚实反馈，end=(offset-1+limit).min(total) 与 skip(offset-1).take(end-offset+1) 一致，边界值（total=0、offset>total、limit clamp 1..2000）逐个代入无误（src/tools/read.rs:66-145）
7. vector_search 接口面（R2）核对一致：k clamp(1,10) 与 schema 描述相符，RecallEngine::recall 契约匹配；与 Fence 无路径耦合，未发现误用（src/tools/vector_search.rs:48-63、src/vector/recall.rs:34-53）
8. 死代码盘点：确定死 0、疑似死 0——errors.rs:50/56 注释明示 4012/4013/5099 预留常量已主动清理登记；Fence::root()（fence.rs:23-25）仍被 FuzzyEngine 消费，非死（src/errors.rs:47-56）
