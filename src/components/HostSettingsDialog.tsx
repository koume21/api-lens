// 観測対象ホストの設定ダイアログ（F21）
// 観測画面の「⚙ 対象ホスト」ボタンから開く。表示名とホストを登録・削除する

import { useState } from "react";

import type { ObservedHost } from "../types";

type HostSettingsDialogProps = {
  hosts: ObservedHost[];
  onAdd: (label: string, host: string) => Promise<void>;
  onDelete: (host: ObservedHost) => Promise<void>;
  onClose: () => void;
};

const inputClass = "h-8 rounded-md border border-line bg-base px-2 text-xs text-fg placeholder:text-dim";

export function HostSettingsDialog({ hosts, onAdd, onDelete, onClose }: HostSettingsDialogProps) {
  const [label, setLabel] = useState("");
  const [host, setHost] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const add = async () => {
    setBusy(true);
    setError(null);
    try {
      await onAdd(label, host);
      // 登録できたら入力欄を空にして、続けて登録できるようにする
      setLabel("");
      setHost("");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (h: ObservedHost) => {
    setError(null);
    try {
      await onDelete(h);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    // 背景（半透明の黒）。背景をクリックしたら閉じる
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={onClose}>
      {/* 中身のクリックは背景まで伝えない（閉じないようにする） */}
      <div
        role="dialog"
        aria-label="観測対象ホスト"
        className="flex max-h-[80vh] w-[520px] flex-col rounded-lg border border-line bg-panel shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-line px-4 py-3">
          <h2 className="text-sm font-bold">観測対象ホスト</h2>
          <button type="button" className="text-muted hover:text-fg" onClick={onClose} aria-label="閉じる">
            ×
          </button>
        </div>

        <p className="px-4 pt-3 text-xs leading-relaxed text-muted">
          ここに登録したホストへの通信だけを記録します。それ以外の通信は中身を開かずにそのまま通します。
          <br />
          ポート付き（<code className="font-mono text-fg">localhost:3000</code>）はそのポートだけ、ポートなし（
          <code className="font-mono text-fg">api.example.com</code>）はすべてのポートが対象です。
        </p>

        {/* 登録済みの一覧 */}
        <ul className="min-h-0 flex-1 overflow-auto px-4 py-3">
          {hosts.length === 0 && <li className="py-2 text-xs text-dim">まだ登録されていません</li>}
          {hosts.map((h) => (
            <li key={h.id} className="flex items-center gap-3 border-b border-line/50 py-2">
              <span className="w-28 truncate text-xs text-fg">{h.label}</span>
              <span className="min-w-0 flex-1 truncate font-mono text-xs text-muted">{h.host}</span>
              <button
                type="button"
                className="rounded-md border border-line px-2 py-0.5 text-[11px] text-muted hover:border-ng hover:text-ng"
                onClick={() => void remove(h)}
              >
                削除
              </button>
            </li>
          ))}
        </ul>

        {/* 新しく登録する */}
        <form
          className="flex flex-col gap-2 border-t border-line px-4 py-3"
          onSubmit={(e) => {
            e.preventDefault();
            void add();
          }}
        >
          <div className="flex gap-2">
            <input
              className={`${inputClass} w-28`}
              placeholder="表示名（ローカル）"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
            />
            <input
              className={`${inputClass} min-w-0 flex-1 font-mono`}
              placeholder="localhost:3000 または https://api.example.com"
              value={host}
              onChange={(e) => setHost(e.target.value)}
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
            />
            <button
              type="submit"
              disabled={busy || label.trim() === "" || host.trim() === ""}
              className="h-8 rounded-md bg-accent px-4 text-xs font-bold text-white disabled:opacity-50"
            >
              登録
            </button>
          </div>
          {error && <p className="text-xs text-ng">{error}</p>}
        </form>
      </div>
    </div>
  );
}
