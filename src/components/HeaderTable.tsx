// ヘッダーの一覧表示（F6）。送信結果と観測詳細の両方で使う
//
// Cookie やトークンなどの秘密情報は、肩越しに見られたり画面共有で映ったりしないよう、
// はじめは伏せ字で表示する（マスキング）。「秘密を表示」を押すとそのまま見える。
// DB には元の値のまま残している（再送に必要なため。保存先は PC の中だけ）。

import { useState } from "react";

import type { HttpHeader } from "../types";

/** 伏せ字にするヘッダー名（小文字で比べる） */
const SENSITIVE_HEADERS = new Set([
  "authorization",
  "proxy-authorization",
  "cookie",
  "set-cookie",
  "x-api-key",
  "x-auth-token",
  "x-csrf-token",
]);

export function isSensitiveHeader(name: string): boolean {
  return SENSITIVE_HEADERS.has(name.toLowerCase());
}

/** 値を伏せ字にする。「Bearer」のような種類は残して、何の値かは分かるようにする */
export function maskValue(value: string): string {
  const [scheme, ...rest] = value.split(" ");
  if (rest.length > 0 && /^(bearer|basic|token)$/i.test(scheme)) return `${scheme} ••••••••`;
  return "••••••••";
}

export function HeaderTable({ headers }: { headers: HttpHeader[] }) {
  const [reveal, setReveal] = useState(false);

  if (headers.length === 0) {
    return <p className="p-4 text-xs text-dim">ヘッダーはありません</p>;
  }
  const hasSensitive = headers.some((h) => isSensitiveHeader(h.name));

  return (
    <div>
      {hasSensitive && (
        <div className="flex justify-end px-4 pt-2">
          <button type="button" className="text-[11px] text-muted hover:text-fg" onClick={() => setReveal((r) => !r)}>
            {reveal ? "🙈 秘密を隠す" : "👁 秘密を表示"}
          </button>
        </div>
      )}
      <table className="w-full table-fixed text-xs">
        <tbody>
          {headers.map((h, i) => {
            const masked = !reveal && isSensitiveHeader(h.name);
            return (
              <tr key={i} className="border-b border-line/50 align-top">
                <td className="w-1/3 px-4 py-1.5 font-mono break-all text-muted">{h.name}</td>
                {/* 値は長いことが多い（Cookie など）ので折り返す。select-text でコピーできるようにする */}
                <td className={`px-4 py-1.5 font-mono break-all select-text ${masked ? "text-dim" : "text-fg"}`}>
                  {masked ? maskValue(h.value) : h.value}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
