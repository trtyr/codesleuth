//! D026（EN-10）：--output-format json 时 rc≠0 走单行 JSON 独立通道，
//! 编排层按行首 `{"code":` 稳定提取 CS 码；人读模式不受影响。
use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_codesleuth"))
        .args(args)
        .env_remove("RUST_BACKTRACE")
        .output()
        .expect("spawn codesleuth")
}

#[test]
fn json_mode_error_is_single_line_parseable_json() {
    // 无任务 → CS1001（run_task_inner 首道检查，LLM 调用之前）
    let out = run(&["--output-format", "json", "--repo", "."]);
    assert_ne!(out.status.code(), Some(0), "缺任务必须非零退出");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let line = stderr
        .lines()
        .find(|l| l.starts_with("{\"code\":"))
        .expect("stderr 应含单行 JSON 错误体（行首 {\"code\":）");
    let v: serde_json::Value = serde_json::from_str(line).expect("单行 JSON 必须可解析");
    assert_eq!(v["code"], "CS1001");
    assert_eq!(v["exit_code"], 1);
    assert!(v["message"].as_str().is_some());
}

#[test]
fn human_mode_error_stays_human_readable() {
    let out = run(&["--repo", "."]);
    assert_ne!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("CS1001"),
        "人读模式保留 CS 码主行: {stderr}"
    );
    assert!(
        !stderr.lines().any(|l| l.starts_with("{\"code\":")),
        "人读模式不输出 JSON 错误体"
    );
}
