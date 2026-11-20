// HTTPS 観測用の認証局証明書で Rust を呼ぶ窓口（F13）
// アプリはキーチェーンに勝手に登録しない。場所を調べる・Finder で開くところまで。
import { invoke } from "@tauri-apps/api/core";

import type { CaInfo } from "../types";

export function caInfo(): Promise<CaInfo> {
  return invoke<CaInfo>("ca_info");
}

/** 証明書のファイルを Finder で表示する */
export function revealCaCert(): Promise<void> {
  return invoke<void>("reveal_ca_cert");
}
