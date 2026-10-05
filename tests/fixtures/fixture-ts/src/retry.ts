/** 默认最大重试次数。 */
export const DEFAULT_MAX_RETRIES = 3;

/** 带指数退避的重试。 */
export async function retryWithBackoff<T>(fn: () => Promise<T>): Promise<T> {
  let attempt = 0;
  for (;;) {
    try {
      return await fn();
    } catch (err) {
      if (attempt >= DEFAULT_MAX_RETRIES) throw err;
      await sleep(200 * 2 ** attempt);
      attempt += 1;
    }
  }
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
