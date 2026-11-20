// 画面1：観測（DevTools の Network タブに近い形）
// いまはツールバーと観測対象ホストの設定（F21）まで。一覧・詳細は観測の実装で足す

import { useEffect, useState } from "react";

import { addHost, deleteHost, listHosts } from "../api/hosts";
import { HostSettingsDialog } from "../components/HostSettingsDialog";
import { ToolListBar } from "../components/ToolListBar";
import type { ObservedHost } from "../types";

export function ObservePage() {
  const [hosts, setHosts] = useState<ObservedHost[]>([]);
  const [hostDialogOpen, setHostDialogOpen] = useState(false);

  // 起動時に登録済みのホストを読む
  useEffect(() => {
    listHosts()
      .then(setHosts)
      .catch((e) => console.error("観測対象ホストを読めませんでした", e));
  }, []);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <ToolListBar onOpenHosts={() => setHostDialogOpen(true)} />

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
          }}
          onClose={() => setHostDialogOpen(false)}
        />
      )}
    </div>
  );
}
