// リクエスト入力（F1〜F4）：メソッド選択・URL・送信ボタン、その下にヘッダー / 本文のタブ

import { useState, type ReactNode } from "react";

import { HTTP_METHODS, type HttpMethod, type RequestInput } from "../types";
import { HeaderEditor } from "./HeaderEditor";
import { Tabs } from "./Tabs";

type RequestEditorProps = {
  request: RequestInput;
  onChange: (request: RequestInput) => void;
  onSend: () => void;
  sending: boolean;
  /** 送信ボタンの右に並べる追加のボタン（保存ボタンなど） */
  actions?: ReactNode;
};

type EditorTab = "headers" | "body";

export function RequestEditor({ request, onChange, onSend, sending, actions }: RequestEditorProps) {
  const [tab, setTab] = useState<EditorTab>("headers");

  // 本文が JSON として読めるか。送信は止めないが、間違いに気づけるよう知らせる
  const bodyError = jsonError(request.body);
  // 空行を除いたヘッダーの数（タブに件数を出す）
  const headerCount = request.headers.filter((h) => h.name.trim() !== "").length;

  return (
    <div className="flex min-h-0 flex-col">
      {/* form にして、URL 欄で Enter を押したら送信できるようにする */}
      <form
        className="flex items-center gap-2 border-b border-line p-4"
        onSubmit={(e) => {
          e.preventDefault();
          if (!sending) onSend();
        }}
      >
        <select
          className="h-9 rounded-md border border-line bg-base px-2 font-mono text-[13px] font-bold text-fg"
          value={request.method}
          onChange={(e) => onChange({ ...request, method: e.target.value as HttpMethod })}
        >
          {HTTP_METHODS.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </select>
        <input
          className="h-9 min-w-0 flex-1 rounded-md border border-line bg-base px-3 font-mono text-[13px] text-fg placeholder:text-dim"
          placeholder="http://localhost:3000/api/users"
          value={request.url}
          onChange={(e) => onChange({ ...request, url: e.target.value })}
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck={false}
        />
        <button
          type="submit"
          disabled={sending || request.url.trim() === ""}
          className="h-9 rounded-md bg-accent px-5 text-[13px] font-bold text-white disabled:opacity-50"
        >
          {sending ? "送信中…" : "送信"}
        </button>
        {actions}
      </form>

      <Tabs
        tabs={[
          { value: "headers", label: `ヘッダー${headerCount > 0 ? `（${headerCount}）` : ""}` },
          { value: "body", label: "本文（JSON）" },
        ]}
        value={tab}
        onChange={setTab}
      />

      <div className="max-h-[40vh] min-h-[120px] overflow-auto">
        {tab === "headers" ? (
          <HeaderEditor headers={request.headers} onChange={(headers) => onChange({ ...request, headers })} />
        ) : (
          <div className="flex flex-col gap-2 p-4">
            <textarea
              className="h-48 resize-y rounded-md border border-line bg-base p-3 font-mono text-xs text-fg placeholder:text-dim"
              placeholder={'{\n  "name": "taro"\n}'}
              value={request.body}
              onChange={(e) => onChange({ ...request, body: e.target.value })}
              spellCheck={false}
            />
            {bodyError && <p className="text-xs text-warn">JSON として読めません: {bodyError}</p>}
            <p className="text-[11px] text-dim">
              Content-Type を指定していなければ application/json を自動で付けて送ります
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

/** 本文が JSON として壊れていればエラーメッセージ、問題なければ null（空なら null） */
function jsonError(body: string): string | null {
  if (body.trim() === "") return null;
  try {
    JSON.parse(body);
    return null;
  } catch (e) {
    return e instanceof Error ? e.message : String(e);
  }
}
