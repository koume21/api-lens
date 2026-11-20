// ヘッダーの一覧表示（F6）。送信結果と観測詳細の両方で使う

import type { HttpHeader } from "../types";

export function HeaderTable({ headers }: { headers: HttpHeader[] }) {
  if (headers.length === 0) {
    return <p className="p-4 text-xs text-dim">ヘッダーはありません</p>;
  }
  return (
    <table className="w-full table-fixed text-xs">
      <tbody>
        {headers.map((h, i) => (
          <tr key={i} className="border-b border-line/50 align-top">
            <td className="w-1/3 px-4 py-1.5 font-mono break-all text-muted">{h.name}</td>
            {/* 値は長いことが多い（Cookie など）ので折り返す。select-text でコピーできるようにする */}
            <td className="px-4 py-1.5 font-mono break-all text-fg select-text">{h.value}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
