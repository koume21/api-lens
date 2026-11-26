// データの形。Rust 側の src-tauri/src/models/mod.rs と 1 対 1 で対応させる。
// 片方を変えたらもう片方も直すこと（serde の camelCase 変換後の名前に合わせている）。

/** ヘッダー 1 つ。同じ名前が複数あってよいので、オブジェクトではなく配列で持つ */
export type HttpHeader = {
  name: string;
  value: string;
};

/** 送信画面で選べるメソッド（F2） */
export const HTTP_METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE"] as const;
export type HttpMethod = (typeof HTTP_METHODS)[number];

/** 送信画面で入力するリクエスト（F1〜F4）。Rust の RequestInput */
export type RequestInput = {
  method: HttpMethod;
  url: string;
  headers: HttpHeader[];
  /** JSON 本文。空文字なら本文なし */
  body: string;
};

/** 通信の種類：観測（ブラウザ → 関所）／送信（送信画面 → 送信役） */
export type ExchangeKind = "observe" | "send";

/** 通信 1 件。Rust の Exchange（DB の表1） */
export type Exchange = {
  /** DB の番号。保存前は 0 */
  id: number;
  /** 通信を始めた日時（UNIX 時刻・ミリ秒） */
  startedAt: number;
  kind: ExchangeKind;
  method: string;
  url: string;
  requestHeaders: HttpHeader[];
  requestBody: string;
  /** レスポンスがない（接続失敗など）ときは null */
  status: number | null;
  responseHeaders: HttpHeader[];
  responseBody: string;
  durationMs: number;
  /** レスポンス本文の実際のサイズ（バイト） */
  size: number;
  /** 本文が 10MB を超えて先頭だけになっているか（N3） */
  bodyTruncated: boolean;
  error: string | null;
};

/** 名前を付けて保存したリクエスト（F9・F10）。Rust の SavedRequest（DB の表2） */
export type SavedRequest = {
  id: number;
  name: string;
  request: RequestInput;
  createdAt: number;
};
