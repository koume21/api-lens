// 画面1：観測（DevTools の Network タブに近い形）
// 上：ツールバー（F11・F14・F15・F20〜F22）
// 左：通信の一覧（F12・F18）／右：選んだ 1 件の詳細（F16）

import { useEffect, useMemo, useState } from "react";

import { addHost, deleteHost, listHosts } from "../api/hosts";
import {
  clearObserved,
  getExchange,
  listObserved,
  observeStatus,
  onExchangeObserved,
  startObserve,
  stopObserve,
} from "../api/observe";
import { ExchangeDetail } from "../components/ExchangeDetail";
import { ExchangeList } from "../components/ExchangeList";
import { HostSettingsDialog } from "../components/HostSettingsDialog";
import { ProxySetupGuide } from "../components/ProxySetupGuide";
import { ToolListBar, type HostFilter, type StatusFilter } from "../components/ToolListBar";
import type { Exchange, ExchangeSummary, ObservedHost, ObserveStatus } from "../types";
import { isSuccess } from "../utils/format";
import { hostMatches } from "../utils/host";

/** 一覧に持つ最大件数（Rust 側で DB に残す件数 N2 と同じ） */
const MAX_ITEMS = 1000;

export function ObservePage() {
  const [status, setStatus] = useState<ObserveStatus>({ running: false, addr: null });
  const [port, setPort] = useState("8080");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [hosts, setHosts] = useState<ObservedHost[]>([]);
  const [hostDialogOpen, setHostDialogOpen] = useState(false);

  /** 通信の一覧（要約・新しい順） */
  const [items, setItems] = useState<ExchangeSummary[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  /** 選んだ 1 件の詳細（クリックしたときに取り寄せる） */
  const [detail, setDetail] = useState<Exchange | null>(null);

  // 絞り込み条件
  const [hostFilter, setHostFilter] = useState<HostFilter>("all");
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");

  // 起動時：前回までの履歴（F19）・登録済みホスト・観測の状態を読む
  useEffect(() => {
    Promise.all([listObserved(), listHosts(), observeStatus()])
      .then(([observed, h, s]) => {
        // イベントで先に届いた分と重ならないよう、番号で重複を除いて合わせる
        setItems((current) => mergeById(current, observed));
        setHosts(h);
        setStatus(s);
      })
      .catch((e) => setError(String(e)));
  }, []);

  // 関所からの「新しい通信」イベントを受けて、一覧の先頭に 1 行足す（F12）
  useEffect(() => {
    // listen は非同期に登録されるので、登録が終わる前に画面が閉じた場合にも解除できるようにする
    // （React の StrictMode では開発中に effect が 2 回実行されるため、これがないと 2 重に登録される）
    let disposed = false;
    let unlisten: (() => void) | undefined;
    onExchangeObserved((summary) => {
      setItems((current) => [summary, ...current].slice(0, MAX_ITEMS));
    }).then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  // 行を選んだら詳細を取り寄せる。素早く別の行を押したとき、古い結果で上書きしないようにする
  useEffect(() => {
    if (selectedId === null) {
      setDetail(null);
      return;
    }
    let cancelled = false;
    getExchange(selectedId)
      .then((ex) => !cancelled && setDetail(ex))
      .catch((e) => !cancelled && setError(String(e)));
    return () => {
      cancelled = true;
    };
  }, [selectedId]);

  // 絞り込んだ一覧（F14・F15・F22）。条件か一覧が変わったときだけ計算し直す
  const filtered = useMemo(() => {
    const selectedHost = hostFilter === "all" ? null : hosts.find((h) => h.id === hostFilter);
    const q = query.trim().toLowerCase();
    return items.filter((ex) => {
      if (selectedHost && !hostMatches(selectedHost.host, ex.url)) return false;
      if (q && !ex.url.toLowerCase().includes(q)) return false;
      if (statusFilter === "ok" && !isSuccess(ex.status)) return false;
      if (statusFilter === "ng" && isSuccess(ex.status)) return false;
      return true;
    });
  }, [items, hosts, hostFilter, query, statusFilter]);

  /** 開始・停止など「押したら Rust に頼んで状態を更新する」処理の共通部分 */
  const run = async (task: () => Promise<ObserveStatus>) => {
    setBusy(true);
    setError(null);
    try {
      setStatus(await task());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const start = () => {
    const p = Number(port);
    if (!Number.isInteger(p) || p < 1 || p > 65535) {
      setError("ポートは 1〜65535 の数字で入力してください");
      return;
    }
    void run(() => startObserve(p));
  };

  const clear = async () => {
    setError(null);
    try {
      await clearObserved();
      setItems([]);
      setSelectedId(null);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <ToolListBar
        status={status}
        port={port}
        onPortChange={setPort}
        onStart={start}
        onStop={() => void run(stopObserve)}
        busy={busy}
        hosts={hosts}
        hostFilter={hostFilter}
        onHostFilterChange={setHostFilter}
        onOpenHosts={() => setHostDialogOpen(true)}
        query={query}
        onQueryChange={setQuery}
        statusFilter={statusFilter}
        onStatusFilterChange={setStatusFilter}
        onClear={() => void clear()}
      />

      {error && (
        <p className="flex items-center justify-between border-b border-line bg-ng/10 px-4 py-2 text-xs text-ng">
          {error}
          <button type="button" onClick={() => setError(null)} aria-label="閉じる">
            ×
          </button>
        </p>
      )}

      {items.length === 0 ? (
        <div className="min-h-0 flex-1 overflow-auto">
          <ProxySetupGuide status={status} hosts={hosts} port={port} />
        </div>
      ) : (
        <div className="flex min-h-0 flex-1">
          {/* 左：一覧。詳細を開いているときは幅を半分にする */}
          <div className={`min-h-0 overflow-auto ${detail ? "w-1/2 border-r border-line" : "flex-1"}`}>
            <ExchangeList items={filtered} selectedId={selectedId} onSelect={setSelectedId} />
            <p className="px-3 py-2 text-[11px] text-dim">
              {filtered.length} / {items.length} 件
            </p>
          </div>
          {/* 右：詳細 */}
          {detail && (
            <div className="relative min-h-0 w-1/2">
              <button
                type="button"
                className="absolute top-2 right-3 z-10 text-muted hover:text-fg"
                onClick={() => setSelectedId(null)}
                aria-label="詳細を閉じる"
              >
                ×
              </button>
              <ExchangeDetail exchange={detail} />
            </div>
          )}
        </div>
      )}

      {hostDialogOpen && (
        <HostSettingsDialog
          hosts={hosts}
          onAdd={async (label, host) => {
            const added = await addHost(label, host);
            setHosts((list) => [...list, added]);
          }}
          onDelete={async (h) => {
            await deleteHost(h.id);
            setHosts((list) => list.filter((x) => x.id !== h.id));
            if (hostFilter === h.id) setHostFilter("all");
          }}
          onClose={() => setHostDialogOpen(false)}
        />
      )}
    </div>
  );
}

/** 2 つの一覧を、番号の重複を除いて新しい順（番号の大きい順）に合わせる */
function mergeById(a: ExchangeSummary[], b: ExchangeSummary[]): ExchangeSummary[] {
  const map = new Map<number, ExchangeSummary>();
  for (const ex of [...a, ...b]) map.set(ex.id, ex);
  return [...map.values()].sort((x, y) => y.id - x.id).slice(0, MAX_ITEMS);
}
