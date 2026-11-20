// 観測画面の「ツールバー」
// Figma: API通信観測 画面デザイン > 観測画面 > ツールバー
// 左から：観測開始/停止（F11）・状態・対象の切り替え（F22）・対象ホスト設定（F21）・
//         URL で絞り込み（F14）・成功/失敗の切り替え（F15）・履歴を削除（F20）

import { useState } from "react";

import type { ObservedHost, ObserveStatus } from "../types";

/** 成功 / 失敗の絞り込み */
export type StatusFilter = "all" | "ok" | "ng";

/** 対象の切り替え：すべて、または登録したホストの番号 */
export type HostFilter = "all" | number;

type ToolListBarProps = {
  status: ObserveStatus;
  /** 止まっているときに入力できる待ち受けポート */
  port: string;
  onPortChange: (port: string) => void;
  onStart: () => void;
  onStop: () => void;
  /** 開始・停止の処理中（ボタンを押せなくする） */
  busy: boolean;

  hosts: ObservedHost[];
  hostFilter: HostFilter;
  onHostFilterChange: (filter: HostFilter) => void;
  /** 「⚙ 対象ホスト」が押されたとき（設定ダイアログを開く） */
  onOpenHosts: () => void;

  query: string;
  onQueryChange: (query: string) => void;
  statusFilter: StatusFilter;
  onStatusFilterChange: (filter: StatusFilter) => void;

  onClear: () => void;
};

const STATUS_OPTIONS: { value: StatusFilter; label: string; color: string }[] = [
  { value: "all", label: "すべて", color: "text-fg" },
  { value: "ok", label: "成功", color: "text-ok" },
  { value: "ng", label: "失敗", color: "text-ng" },
];

export function ToolListBar(props: ToolListBarProps) {
  const { status } = props;
  // 削除は取り消せないので、1 回目で確認表示、2 回目で実行する
  const [confirmClear, setConfirmClear] = useState(false);

  return (
    <div className="flex h-[53px] shrink-0 items-center gap-[10px] border-b border-line bg-panel px-4">
      {/* 観測を開始 / 停止ボタン（観測中は赤、止まっていれば緑） */}
      {status.running ? (
        <button
          type="button"
          disabled={props.busy}
          onClick={props.onStop}
          className="h-8 rounded-md border border-ng px-[15px] text-[13px] whitespace-nowrap text-ng disabled:opacity-50"
        >
          ■ 観測を停止
        </button>
      ) : (
        <button
          type="button"
          disabled={props.busy}
          onClick={props.onStart}
          className="h-8 rounded-md border border-ok px-[15px] text-[13px] whitespace-nowrap text-ok disabled:opacity-50"
        >
          ▶ 観測を開始
        </button>
      )}

      {/* 観測状態（観測中は緑の丸＋待ち受けアドレス、止まっていればポート入力） */}
      {status.running ? (
        <div className="flex items-center gap-[6px]">
          <span className="size-2 rounded-full bg-ok" />
          <p className="text-xs whitespace-nowrap text-ok">
            観測中（<span className="font-mono">{status.addr}</span>）
          </p>
        </div>
      ) : (
        <label className="flex items-center gap-[6px] text-xs whitespace-nowrap text-muted">
          <span className="size-2 rounded-full bg-dim" />
          停止中・ポート
          <input
            className="h-7 w-16 rounded-md border border-line bg-base px-2 font-mono text-xs text-fg"
            inputMode="numeric"
            value={props.port}
            onChange={(e) => props.onPortChange(e.target.value.replace(/\D/g, ""))}
          />
        </label>
      )}

      {/* 対象の切り替え（すべて / 登録したホスト 1 つ）（F22） */}
      <label className="flex h-8 items-center gap-1 rounded-md border border-line bg-base pl-[11px] text-xs whitespace-nowrap">
        <span className="text-muted">対象：</span>
        <select
          className="h-full bg-transparent pr-2 text-fg outline-none"
          value={String(props.hostFilter)}
          onChange={(e) => props.onHostFilterChange(e.target.value === "all" ? "all" : Number(e.target.value))}
        >
          <option value="all">すべて</option>
          {props.hosts.map((h) => (
            <option key={h.id} value={h.id}>
              {h.label}（{h.host}）
            </option>
          ))}
        </select>
      </label>

      {/* 対象ホストの設定ボタン（F21） */}
      <button
        type="button"
        onClick={props.onOpenHosts}
        className="h-8 rounded-md border border-line px-[15px] text-[13px] whitespace-nowrap text-fg hover:bg-line/40"
      >
        ⚙ 対象ホスト
      </button>

      {/* URL で絞り込む入力欄（F14） */}
      <input
        type="text"
        placeholder="🔍 URLで絞り込み（例: /api/users）"
        value={props.query}
        onChange={(e) => props.onQueryChange(e.target.value)}
        className="h-8 w-[240px] min-w-0 rounded-md border border-line bg-base px-[11px] text-[13px] text-fg placeholder:text-dim"
        spellCheck={false}
      />

      {/* すべて／成功／失敗の切り替え（F15） */}
      <div className="flex h-[30px] items-center rounded-md border border-line p-px text-xs whitespace-nowrap">
        {STATUS_OPTIONS.map((o) => (
          <button
            key={o.value}
            type="button"
            onClick={() => props.onStatusFilterChange(o.value)}
            className={`flex h-7 items-center rounded-[5px] px-3 ${o.color} ${
              props.statusFilter === o.value ? "bg-line" : ""
            }`}
          >
            {o.label}
          </button>
        ))}
      </div>

      {/* 右端に寄せるためのすき間 */}
      <div className="flex-1" />

      {/* 履歴を削除ボタン（F20） */}
      <button
        type="button"
        onClick={() => {
          if (confirmClear) {
            props.onClear();
            setConfirmClear(false);
          } else {
            setConfirmClear(true);
          }
        }}
        onBlur={() => setConfirmClear(false)}
        className={`h-8 rounded-md border px-[15px] text-[13px] whitespace-nowrap ${
          confirmClear ? "border-ng bg-ng text-white" : "border-line text-muted"
        }`}
      >
        {confirmClear ? "本当に削除する" : "🗑 履歴を削除"}
      </button>
    </div>
  );
}
