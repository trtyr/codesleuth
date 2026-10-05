import { retryWithBackoff } from "./retry";

export interface FetchOptions {
  url: string;
  timeoutMs: number;
}

export async function fetchWithRetry(opts: FetchOptions): Promise<string> {
  return retryWithBackoff(async () => {
    const res = await fetch(opts.url);
    if (!res.ok) throw new Error(`http ${res.status}`);
    return res.text();
  });
}
