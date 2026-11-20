// 観測モードで Rust を呼ぶ窓口と、Rust からのイベント受信（F11・F12・F16・F19・F20）
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { Exchange, ExchangeSummary, ObserveStatus } from "../types";

/** 関所が通信を記録したときのイベント名（Rust の EXCHANGE_OBSERVED_EVENT と同じ） */
const EXCHANGE_OBSERVED_EVENT = "exchange-observed";

/** 観測を開始する。127.0.0.1:<port> で待ち受ける */
export function startObserve(port: number): Promise<ObserveStatus> {
  return invoke<ObserveStatus>("start_observe", { port });
}

export function stopObserve(): Promise<ObserveStatus> {
  return invoke<ObserveStatus>("stop_observe");
}

export function observeStatus(): Promise<ObserveStatus> {
  return invoke<ObserveStatus>("observe_status");
}

/** 観測した通信の一覧（要約・新しい順・最大 1,000 件）。前回起動時の分も入っている */
export function listObserved(): Promise<ExchangeSummary[]> {
  return invoke<ExchangeSummary[]>("list_observed");
}

/** 通信 1 件の詳細（ヘッダー・本文まで） */
export function getExchange(id: number): Promise<Exchange> {
  return invoke<Exchange>("get_exchange", { id });
}

/** 観測した通信の履歴をまとめて削除する */
export function clearObserved(): Promise<void> {
  return invoke<void>("clear_observed");
}

/**
 * 関所が通信を 1 件記録するたびに呼ばれる関数を登録する。
 * 戻り値の Promise が解決すると「登録解除する関数」が手に入るので、画面を閉じるときに呼ぶこと。
 */
export function onExchangeObserved(handler: (summary: ExchangeSummary) => void): Promise<UnlistenFn> {
  return listen<ExchangeSummary>(EXCHANGE_OBSERVED_EVENT, (event) => handler(event.payload));
}
