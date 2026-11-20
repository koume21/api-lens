// 送信画面の左側：保存したリクエストの一覧（F10）と送信履歴
// クリックで送信画面に読み込む。保存したものはここから削除もできる

import { useState } from "react";

import type { Exchange, SavedRequest } from "../types";
import { formatTime, methodColorClass, statusColorClass } from "../utils/format";
import { Tabs } from "./Tabs";

type SidebarTab = "saved" | "history";

type SavedRequestListProps = {
  saved: SavedRequest[];
  history: Exchange[];
  /** いま読み込んでいる保存リクエストの番号（強調表示する） */
  selectedId: number | null;
  onSelectSaved: (saved: SavedRequest) => void;
  onDeleteSaved: (saved: SavedRequest) => void;
  onSelectHistory: (ex: Exchange) => void;
};

export function SavedRequestList(props: SavedRequestListProps) {
  const [tab, setTab] = useState<SidebarTab>("saved");

  return (
    <aside className="flex w-64 shrink-0 flex-col border-r border-line bg-panel">
      <Tabs
        tabs={[
          { value: "saved", label: `保存済み（${props.saved.length}）` },
          { value: "history", label: "履歴" },
        ]}
        value={tab}
        onChange={setTab}
      />
      <div className="min-h-0 flex-1 overflow-auto">
        {tab === "saved" ? <SavedTab {...props} /> : <HistoryTab {...props} />}
      </div>
    </aside>
  );
}

function SavedTab({ saved, selectedId, onSelectSaved, onDeleteSaved }: SavedRequestListProps) {
  // 削除の確認中の行。WebView では window.confirm が使えない環境があるため、
  // 「×」→「削除」の 2 回押しで確認する
  const [confirmId, setConfirmId] = useState<number | null>(null);

  if (saved.length === 0) {
    return <p className="p-4 text-xs leading-relaxed text-dim">まだありません。リクエストを入力して「保存」を押すとここに並びます</p>;
  }
  return (
    <ul>
      {saved.map((s) => (
        <li
          key={s.id}
          className={`group flex items-center gap-2 border-b border-line/50 px-3 py-2 ${
            s.id === selectedId ? "bg-accent/15" : "hover:bg-line/40"
          }`}
          onMouseLeave={() => setConfirmId(null)}
        >
          <button type="button" className="flex min-w-0 flex-1 flex-col items-start text-left" onClick={() => onSelectSaved(s)}>
            <span className="w-full truncate text-xs text-fg">{s.name}</span>
            <span className="flex w-full gap-1.5 font-mono text-[10px]">
              <span className={methodColorClass(s.request.method)}>{s.request.method}</span>
              <span className="truncate text-dim">{s.request.url}</span>
            </span>
          </button>
          {confirmId === s.id ? (
            <button
              type="button"
              className="shrink-0 rounded bg-ng px-2 py-0.5 text-[10px] text-white"
              onClick={() => {
                onDeleteSaved(s);
                setConfirmId(null);
              }}
            >
              削除
            </button>
          ) : (
            <button
              type="button"
              className="shrink-0 px-1 text-xs text-dim opacity-0 group-hover:opacity-100 hover:text-ng"
              onClick={() => setConfirmId(s.id)}
              aria-label={`${s.name} を削除`}
            >
              ×
            </button>
          )}
        </li>
      ))}
    </ul>
  );
}

function HistoryTab({ history, onSelectHistory }: SavedRequestListProps) {
  if (history.length === 0) {
    return <p className="p-4 text-xs text-dim">まだ送信していません</p>;
  }
  return (
    <ul>
      {history.map((ex) => (
        <li key={ex.id}>
          <button
            type="button"
            className="flex w-full flex-col items-start border-b border-line/50 px-3 py-2 text-left hover:bg-line/40"
            onClick={() => onSelectHistory(ex)}
          >
            <span className="flex w-full gap-1.5 font-mono text-[10px]">
              <span className="text-dim">{formatTime(ex.startedAt)}</span>
              <span className={methodColorClass(ex.method)}>{ex.method}</span>
              <span className={statusColorClass(ex.status)}>{ex.status ?? "ERR"}</span>
            </span>
            <span className="w-full truncate font-mono text-[11px] text-fg">{ex.url}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}
