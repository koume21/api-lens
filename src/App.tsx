import { useState } from "react";

import { TitleBar } from "./components/TitleBar";
import { ToolListBar } from "./components/ToolListBar";
import { SendPage } from "./pages/SendPage";

// Tabのタイプ定義
export type Tab = "observe" | "send";

function App() {
  const [tab, setTab] = useState<Tab>("observe");

  return (
    <main className="flex h-full flex-col bg-base font-sans text-fg">
      <TitleBar tab={tab} onTabChange={setTab} />
      {/*
        2 つの画面は両方とも描画したまま、見えない方を hidden で隠す。
        条件分岐で片方だけ描画すると、タブを切り替えたときに入力中の内容や結果が消えてしまうため。
      */}
      <div className={tab === "observe" ? "min-h-0 flex-1" : "hidden"}>
        <ToolListBar />
      </div>
      <div className={tab === "send" ? "min-h-0 flex-1" : "hidden"}>
        <SendPage />
      </div>
    </main>
  );
}

export default App;
