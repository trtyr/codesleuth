import { fetchWithRetry } from "./http";

async function main() {
  const body = await fetchWithRetry({ url: "http://localhost:8080", timeoutMs: 1000 });
  console.log(body);
}

main();
