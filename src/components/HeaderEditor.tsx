// ヘッダーを名前と値の形で追加・削除する入力欄（F3）
// 認証も専用 UI は作らず、ここに「Authorization: Bearer ...」を手で入れる方針（設計の「認証と証明書」）

import type { HttpHeader } from "../types";

type HeaderEditorProps = {
  headers: HttpHeader[];
  onChange: (headers: HttpHeader[]) => void;
};

const inputClass =
  "h-8 min-w-0 flex-1 rounded-md border border-line bg-base px-2 font-mono text-xs text-fg placeholder:text-dim";

export function HeaderEditor({ headers, onChange }: HeaderEditorProps) {
  // i 番目の行の name か value を書き換えた新しい配列を作る（state は直接書き換えない）
  const update = (i: number, field: keyof HttpHeader, value: string) => {
    onChange(headers.map((h, idx) => (idx === i ? { ...h, [field]: value } : h)));
  };
  const remove = (i: number) => onChange(headers.filter((_, idx) => idx !== i));
  const add = () => onChange([...headers, { name: "", value: "" }]);

  return (
    <div className="flex flex-col gap-2 p-4">
      {headers.map((h, i) => (
        // 並べ替えはしないので key は添字で足りる
        <div key={i} className="flex items-center gap-2">
          <input
            className={inputClass}
            placeholder="名前（例: Authorization）"
            value={h.name}
            onChange={(e) => update(i, "name", e.target.value)}
          />
          <input
            className={`${inputClass} flex-[2]`}
            placeholder="値（例: Bearer xxxxx）"
            value={h.value}
            onChange={(e) => update(i, "value", e.target.value)}
          />
          <button
            type="button"
            onClick={() => remove(i)}
            className="h-8 w-8 shrink-0 rounded-md border border-line text-muted hover:text-ng"
            aria-label="このヘッダーを削除"
          >
            ×
          </button>
        </div>
      ))}
      <button
        type="button"
        onClick={add}
        className="h-8 self-start rounded-md border border-line px-3 text-xs text-muted hover:text-fg"
      >
        ＋ ヘッダーを追加
      </button>
    </div>
  );
}
