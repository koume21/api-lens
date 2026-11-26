// 観測対象ホストとの当てはめ（F22: 一覧を登録したホスト 1 つに絞る）
// Rust 側の models::host_matches と同じ規則。変えるときは両方直すこと。
//   - 登録にポートがある（localhost:3000）→ ホストとポートが両方一致
//   - 登録にポートがない（api.example.com）→ ポートは問わない

/** 登録したホストが「ポート付き」か（IPv6 の [::1]:3000 も考える） */
function hasPort(host: string): boolean {
  return /^(\[[^\]]+\]|[^:]+):\d+$/.test(host);
}

/** URL の宛先が登録したホストに当たるか。URL として読めなければ false */
export function hostMatches(registered: string, url: string): boolean {
  let u: URL;
  try {
    u = new URL(url);
  } catch {
    return false;
  }
  if (!hasPort(registered)) {
    return u.hostname.toLowerCase() === registered;
  }
  // URL.port は既定のポート（http:80 / https:443）だと空になるので補う
  const port = u.port || (u.protocol === "https:" ? "443" : "80");
  return `${u.hostname.toLowerCase()}:${port}` === registered;
}
