// 送信モードで Rust を呼ぶ窓口。
// invoke の呼び出しは src/api/ に集約し、画面のコンポーネントからは直接 invoke しない。
// （コマンド名の文字列や引数の形を 1 か所で管理するため）
// 失敗すると Promise が reject され、エラーには Rust 側で作った日本語の文字列が入る。
import { invoke } from "@tauri-apps/api/core";

import type { Exchange, RequestInput, SavedRequest } from "../types";

/** リクエストを送り、結果を受け取る。送信の失敗は Exchange.error に入る */
export function sendRequest(input: RequestInput): Promise<Exchange> {
  return invoke<Exchange>("send_request", { input });
}

/** 送信履歴（新しい順） */
export function listSendHistory(): Promise<Exchange[]> {
  return invoke<Exchange[]>("list_send_history");
}

/** 名前を付けて新しく保存する（F9） */
export function saveRequest(name: string, request: RequestInput): Promise<SavedRequest> {
  return invoke<SavedRequest>("save_request", { name, request });
}

/** 保存済みのものを上書きする */
export function updateSavedRequest(id: number, name: string, request: RequestInput): Promise<SavedRequest> {
  return invoke<SavedRequest>("update_saved_request", { id, name, request });
}

/** 保存したリクエストの一覧（名前順）（F10） */
export function listSavedRequests(): Promise<SavedRequest[]> {
  return invoke<SavedRequest[]>("list_saved_requests");
}

export function deleteSavedRequest(id: number): Promise<void> {
  return invoke<void>("delete_saved_request", { id });
}
