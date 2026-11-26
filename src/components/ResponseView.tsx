// 送信結果の表示（F5〜F8）：ステータス（色付き）・時間・サイズ、その下にヘッダー / 本文のタブ

import { useState } from "react";

import type { Exchange } from "../types";
import { formatBytes, statusColorClass } from "../utils/format";
import { BodyView } from "./BodyView";
import { HeaderTable } from "./HeaderTable";
import { Tabs } from "./Tabs";

type ResultTab = "body" | "headers";

export function ResponseView({ result }: { result: Exchange | null }) {
  const [tab, setTab] = useState<ResultTab>("body");

  if (!result) {
    return <p className="p-4 text-xs text-dim">URL を入力して「送信」を押すと、ここに結果が出ます</p>;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {/* 結果の要約行 */}
      <div className="flex items-center gap-5 border-b border-line px-4 py-2 text-xs">
        <span className="text-muted">
          ステータス{" "}
          <span className={`font-mono text-[13px] font-bold ${statusColorClass(result.status)}`}>
            {result.status ?? "エラー"}
          </span>
        </span>
        <span className="text-muted">
          時間 <span className="font-mono text-fg">{result.durationMs} ms</span>
        </span>
        <span className="text-muted">
          サイズ <span className="font-mono text-fg">{formatBytes(result.size)}</span>
        </span>
      </div>

      {/* 送信に失敗したときは理由を出す（タイムアウト・接続失敗など） */}
      {result.error && <p className="border-b border-line bg-ng/10 px-4 py-2 text-xs text-ng">{result.error}</p>}

      <Tabs
        tabs={[
          { value: "body", label: "本文" },
          { value: "headers", label: `ヘッダー（${result.responseHeaders.length}）` },
        ]}
        value={tab}
        onChange={setTab}
      />

      <div className="min-h-0 flex-1 overflow-auto">
        {tab === "body" ? (
          <BodyView body={result.responseBody} truncated={result.bodyTruncated} size={result.size} />
        ) : (
          <HeaderTable headers={result.responseHeaders} />
        )}
      </div>
    </div>
  );
}
