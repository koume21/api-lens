// 送信モードで Rust を呼ぶ窓口。
// invoke の呼び出しは src/api/ に集約し、画面のコンポーネントからは直接 invoke しない。
// （コマンド名の文字列や引数の形を 1 か所で管理するため）
import { invoke } from "@tauri-apps/api/core";

import type { Exchange, RequestInput } from "../types";

/** リクエストを送り、結果を受け取る。送信の失敗は Exchange.error に入る */
export function sendRequest(input: RequestInput): Promise<Exchange> {
  return invoke<Exchange>("send_request", { input });
}
