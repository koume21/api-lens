// 本文の表示（F7: JSON なら整形）。送信結果と観測詳細の両方で使う

import { useMemo } from "react";

import { formatBytes, prettyBody } from "../utils/format";

type BodyViewProps = {
  body: string;
  /** 本文が 10MB を超えて先頭だけになっているか（N3） */
  truncated?: boolean;
  /** 元の本文サイズ（省略の案内に使う） */
  size?: number;
};

export function BodyView({ body, truncated = false, size }: BodyViewProps) {
  // JSON.parse / stringify は大きい本文だと重いので、本文が変わったときだけやり直す
  const { text, isJson } = useMemo(() => prettyBody(body), [body]);

  return (
    <div className="flex flex-col">
      {truncated && (
        <p className="border-b border-line bg-warn/10 px-4 py-2 text-xs text-warn">
          本文が大きいため先頭の 10MB だけを表示しています
          {size !== undefined && `（全体: ${formatBytes(size)}）`}
        </p>
      )}
      {body.length === 0 ? (
        <p className="p-4 text-xs text-dim">本文はありません</p>
      ) : (
        <pre className="p-4 font-mono text-xs break-all whitespace-pre-wrap text-fg select-text">
          {isJson && <span className="mb-2 block text-[10px] text-dim">JSON（整形済み）</span>}
          {text}
        </pre>
      )}
    </div>
  );
}
