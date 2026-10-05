use std::time::Duration;

/// 最大重试次数。
pub const MAX_RETRIES: u32 = 3;

/// 带指数退避的重试逻辑——侦察目标：找到这个函数并引用 file:line。
pub fn retry_with_backoff<F, T, E>(mut f: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempt = 0u32;
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                if attempt >= MAX_RETRIES {
                    return Err(e);
                }
                let backoff = Duration::from_millis(200u64 << attempt);
                std::thread::sleep(backoff);
                attempt += 1;
            }
        }
    }
}

/// 判断错误是否可重试。
pub fn is_retryable(status: u16) -> bool {
    status == 429 || status >= 500
}
