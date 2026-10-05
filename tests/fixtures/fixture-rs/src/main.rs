mod config;
mod retry;

fn main() {
    let cfg = config::load_default();
    println!("cfg: {cfg:?}");
    let result = retry::retry_with_backoff(|| Ok::<_, ()>(42));
    println!("result: {result:?}");
}
