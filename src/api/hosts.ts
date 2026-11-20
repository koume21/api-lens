// 観測対象ホストの登録・削除で Rust を呼ぶ窓口（F21）
import { invoke } from "@tauri-apps/api/core";

import type { ObservedHost } from "../types";

export function listHosts(): Promise<ObservedHost[]> {
  return invoke<ObservedHost[]>("list_hosts");
}

/** URL を渡しても Rust 側で「localhost:3000」の形に整えて登録される */
export function addHost(label: string, host: string): Promise<ObservedHost> {
  return invoke<ObservedHost>("add_host", { label, host });
}

export function deleteHost(id: number): Promise<void> {
  return invoke<void>("delete_host", { id });
}
