// 観測画面の左側：通信の一覧（F12・F18）
// 列：時刻・メソッド・URL・ステータス・時間（ms）

import type { ExchangeSummary } from "../types";
import { formatTime, methodColorClass, statusColorClass } from "../utils/format";

type ExchangeListProps = {
  items: ExchangeSummary[];
  selectedId: number | null;
  onSelect: (id: number) => void;
};

export function ExchangeList({ items, selectedId, onSelect }: ExchangeListProps) {
  return (
    <table className="w-full table-fixed text-xs">
      {/* 見出しはスクロールしても上に残す */}
      <thead className="sticky top-0 bg-panel text-left text-[11px] text-muted">
        <tr className="border-b border-line">
          <th className="w-[92px] px-3 py-2 font-normal">時刻</th>
          <th className="w-[64px] px-2 py-2 font-normal">メソッド</th>
          <th className="px-2 py-2 font-normal">URL</th>
          <th className="w-[64px] px-2 py-2 font-normal">ステータス</th>
          <th className="w-[72px] px-3 py-2 text-right font-normal">時間</th>
        </tr>
      </thead>
      <tbody className="font-mono">
        {items.map((ex) => (
          <tr
            key={ex.id}
            onClick={() => onSelect(ex.id)}
            className={`cursor-default border-b border-line/50 ${
              ex.id === selectedId ? "bg-accent/20" : "hover:bg-line/40"
            }`}
          >
            <td className="px-3 py-1.5 text-dim">{formatTime(ex.startedAt)}</td>
            <td className={`px-2 py-1.5 ${methodColorClass(ex.method)}`}>{ex.method}</td>
            {/* URL は長いので省略表示。全体はマウスを乗せると見える */}
            <td className="truncate px-2 py-1.5 text-fg" title={ex.url}>
              {ex.url}
            </td>
            <td className={`px-2 py-1.5 ${statusColorClass(ex.status)}`}>{ex.status ?? "ERR"}</td>
            <td className="px-3 py-1.5 text-right text-muted">{ex.durationMs} ms</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
