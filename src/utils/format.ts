// 表示用の小さな関数（送信画面と観測画面の両方で使う）

import type { HttpHeader } from "../types";

/** バイト数を「1.2 KB」のような読みやすい形にする（F8） */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return `${value.toFixed(1)} ${units[i]}`;
}

/** 時刻を「12:34:56.789」の形にする（観測一覧の時刻列） */
export function formatTime(ms: number): string {
  const d = new Date(ms);
  const pad = (n: number, len = 2) => String(n).padStart(len, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}.${pad(d.getMilliseconds(), 3)}`;
}

/** 成功（2xx / 3xx）かどうか。レスポンスがない（null）なら失敗扱い */
export function isSuccess(status: number | null): boolean {
  return status !== null && status >= 200 && status < 400;
}

/** ステータスの文字色（F5: 成功は緑・失敗は赤） */
export function statusColorClass(status: number | null): string {
  if (status === null) return "text-ng";
  if (status >= 200 && status < 300) return "text-ok";
  if (status >= 300 && status < 400) return "text-warn";
  return "text-ng";
}

/** 名前が一致するヘッダーの値を探す（大文字小文字は区別しない） */
export function findHeader(headers: HttpHeader[], name: string): string | undefined {
  const lower = name.toLowerCase();
  return headers.find((h) => h.name.toLowerCase() === lower)?.value;
}

/**
 * 本文を表示用に整形する（F7: JSON なら見やすく）。
 * Content-Type が JSON でなくても、JSON として読めれば整形する
 * （Content-Type を付け忘れる API も多いため）。読めなければそのまま返す。
 */
export function prettyBody(body: string): { text: string; isJson: boolean } {
  const trimmed = body.trim();
  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
    try {
      return { text: JSON.stringify(JSON.parse(trimmed), null, 2), isJson: true };
    } catch {
      // JSON っぽいが壊れている（途中で省略された本文など）。そのまま出す
    }
  }
  return { text: body, isJson: false };
}

/** メソッドの文字色（一覧で見分けやすくする） */
export function methodColorClass(method: string): string {
  switch (method.toUpperCase()) {
    case "GET":
      return "text-ok";
    case "POST":
      return "text-warn";
    case "PUT":
    case "PATCH":
      return "text-accent";
    case "DELETE":
      return "text-ng";
    default:
      return "text-muted";
  }
}
