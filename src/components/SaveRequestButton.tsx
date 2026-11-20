// 「保存」ボタン（F9）。押すと名前の入力欄が開く
// 保存済みのリクエストを読み込んでいるときは「上書き」と「新しく保存」を選べる

import { useState } from "react";

import type { SavedRequest } from "../types";

type SaveRequestButtonProps = {
  /** いま読み込んでいる保存リクエスト（なければ null） */
  current: SavedRequest | null;
  /** id があれば上書き、null なら新しく保存 */
  onSave: (name: string, overwriteId: number | null) => Promise<void>;
};

export function SaveRequestButton({ current, onSave }: SaveRequestButtonProps) {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [saving, setSaving] = useState(false);

  const start = () => {
    setName(current?.name ?? "");
    setOpen(true);
  };

  const submit = async (overwriteId: number | null) => {
    if (name.trim() === "") return;
    setSaving(true);
    try {
      await onSave(name, overwriteId);
      setOpen(false);
    } catch {
      // 失敗の表示は親（SendPage）が行う。入力欄は開いたままにして、やり直せるようにする
    } finally {
      setSaving(false);
    }
  };

  return (
    // relative: 入力欄（absolute）の位置の基準にする
    <div className="relative">
      <button
        type="button"
        onClick={() => (open ? setOpen(false) : start())}
        className="h-9 rounded-md border border-line px-4 text-[13px] text-fg hover:bg-line/40"
      >
        保存
      </button>

      {open && (
        <div className="absolute top-11 right-0 z-10 flex w-72 flex-col gap-2 rounded-md border border-line bg-panel p-3 shadow-lg">
          <input
            autoFocus
            className="h-8 rounded-md border border-line bg-base px-2 text-xs text-fg placeholder:text-dim"
            placeholder="名前（例: ユーザー一覧を取得）"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => {
              // form の中にいる（URL 欄と同じ form）ので、Enter が送信にならないよう止める
              if (e.key === "Enter") {
                e.preventDefault();
                void submit(current?.id ?? null);
              }
              if (e.key === "Escape") setOpen(false);
            }}
          />
          <div className="flex justify-end gap-2">
            <button type="button" className="h-7 px-2 text-xs text-muted" onClick={() => setOpen(false)}>
              キャンセル
            </button>
            {current && (
              <button
                type="button"
                disabled={saving}
                className="h-7 rounded-md border border-line px-3 text-xs text-fg"
                onClick={() => void submit(null)}
              >
                新しく保存
              </button>
            )}
            <button
              type="button"
              disabled={saving || name.trim() === ""}
              className="h-7 rounded-md bg-accent px-3 text-xs font-bold text-white disabled:opacity-50"
              onClick={() => void submit(current?.id ?? null)}
            >
              {current ? "上書き保存" : "保存"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
