// 通信の記録（Exchange）を送信画面の入力（RequestInput）に直す。
// 送信履歴の読み込みと、観測した通信の「送信画面にコピー」（F17）で使う。

import { HTTP_METHODS, type Exchange, type HttpMethod, type RequestInput } from "../types";

/** 送信画面で選べないメソッド（HEAD / OPTIONS など）は GET に寄せる */
function toMethod(method: string): HttpMethod {
  const upper = method.toUpperCase();
  return (HTTP_METHODS as readonly string[]).includes(upper) ? (upper as HttpMethod) : "GET";
}

export function exchangeToRequest(ex: Exchange): RequestInput {
  return {
    method: toMethod(ex.method),
    url: ex.url,
    // 入力欄が 0 行だと追加しづらいので、空なら空行を 1 つ置く
    headers: ex.requestHeaders.length > 0 ? ex.requestHeaders.map((h) => ({ ...h })) : [{ name: "", value: "" }],
    body: ex.requestBody,
  };
}

/**
 * 観測した通信から送り直すときに外すヘッダー（小文字）。
 * - 接続の管理用（送信役の reqwest が自分で付ける・付けてはいけない）
 * - Accept-Encoding: 送信役は圧縮を解かないので、付けたままだと本文が読めない形で返ってくる
 */
const DROP_ON_RESEND = new Set([
  "host",
  "content-length",
  "connection",
  "proxy-connection",
  "keep-alive",
  "transfer-encoding",
  "te",
  "upgrade",
  "accept-encoding",
]);

/** 観測した通信を送信画面の入力にする（F17: 送信画面にコピーして送り直す） */
export function observedToRequest(ex: Exchange): RequestInput {
  const request = exchangeToRequest(ex);
  const headers = request.headers.filter((h) => !DROP_ON_RESEND.has(h.name.toLowerCase()));
  return { ...request, headers: headers.length > 0 ? headers : [{ name: "", value: "" }] };
}
