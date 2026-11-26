import { useState } from "react";

import { TitleBar } from "./components/TitleBar";
import { ObservePage } from "./pages/ObservePage";
import { SendPage, type IncomingRequest } from "./pages/SendPage";
import type { Exchange } from "./types";
import { observedToRequest } from "./utils/request";

// Tabのタイプ定義
export type Tab = "observe" | "send";

function App() {
  const [tab, setTab] = useState<Tab>("observe");
  // 観測画面から送信画面へ渡すリクエスト（F17）。
  // 同じ通信を 2 回コピーしても読み込み直せるよう、毎回変わる key を付ける
  const [incoming, setIncoming] = useState<IncomingRequest | null>(null);

  const copyToSend = (ex: Exchange) => {
    setIncoming({ request: observedToRequest(ex), key: Date.now() });
    setTab("send");
  };

  return (
    <main className="flex h-full flex-col bg-base font-sans text-fg">
      <TitleBar tab={tab} onTabChange={setTab} />
      {/*
        2 つの画面は両方とも描画したまま、見えない方を hidden で隠す。
        条件分岐で片方だけ描画すると、タブを切り替えたときに入力中の内容や結果が消えてしまうため。
      */}
      <div className={tab === "observe" ? "min-h-0 flex-1" : "hidden"}>
        <ObservePage onCopyToSend={copyToSend} />
      </div>
      <div className={tab === "send" ? "min-h-0 flex-1" : "hidden"}>
        <SendPage incoming={incoming} />
      </div>
    </main>
  );
}

export default App;
