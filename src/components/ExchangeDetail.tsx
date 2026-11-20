// 観測画面の右側：選んだ 1 件の詳細（F16・F18）
// 「送った内容」と「返ってきた内容」をタブで切り替え、それぞれヘッダーと本文を出す

import { useState, type ReactNode } from "react";

import type { Exchange } from "../types";
import { formatBytes, formatTime, statusColorClass } from "../utils/format";
import { BodyView } from "./BodyView";
import { HeaderTable } from "./HeaderTable";
import { Tabs } from "./Tabs";

type DetailTab = "request" | "response";

type ExchangeDetailProps = {
  exchange: Exchange;
  /** 要約の右に並べるボタン（「送信画面にコピー」など） */
  actions?: ReactNode;
};

export function ExchangeDetail({ exchange: ex, actions }: ExchangeDetailProps) {
  const [tab, setTab] = useState<DetailTab>("response");

  return (
    <div className="flex h-full min-h-0 flex-col">
      {/* 要約：メソッドと URL、ステータス・時間・サイズ */}
      <div className="flex flex-col gap-1 border-b border-line px-4 py-3">
        <p className="pr-6 font-mono text-xs break-all text-fg select-text">
          <span className="mr-2 font-bold">{ex.method}</span>
          {ex.url}
        </p>
        <div className="flex items-center gap-4 text-[11px] text-muted">
          <span>
            ステータス <span className={`font-mono font-bold ${statusColorClass(ex.status)}`}>{ex.status ?? "エラー"}</span>
          </span>
          <span>
            時間 <span className="font-mono text-fg">{ex.durationMs} ms</span>
          </span>
          <span>
            サイズ <span className="font-mono text-fg">{formatBytes(ex.size)}</span>
          </span>
          <span className="font-mono text-dim">{formatTime(ex.startedAt)}</span>
          <span className="flex-1" />
          {actions}
        </div>
      </div>

      {ex.error && <p className="border-b border-line bg-ng/10 px-4 py-2 text-xs text-ng">{ex.error}</p>}

      <Tabs
        tabs={[
          { value: "request", label: "送った内容" },
          { value: "response", label: "返ってきた内容" },
        ]}
        value={tab}
        onChange={setTab}
      />

      <div className="min-h-0 flex-1 overflow-auto">
        {tab === "request" ? (
          <>
            <Section title={`ヘッダー（${ex.requestHeaders.length}）`}>
              <HeaderTable headers={ex.requestHeaders} />
            </Section>
            <Section title="本文">
              <BodyView body={ex.requestBody} />
            </Section>
          </>
        ) : (
          <>
            <Section title={`ヘッダー（${ex.responseHeaders.length}）`}>
              <HeaderTable headers={ex.responseHeaders} />
            </Section>
            <Section title="本文">
              <BodyView body={ex.responseBody} truncated={ex.bodyTruncated} size={ex.size} />
            </Section>
          </>
        )}
      </div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="border-b border-line">
      <h3 className="bg-panel px-4 py-1.5 text-[11px] text-muted">{title}</h3>
      {children}
    </section>
  );
}
