import { useState } from "react";

import { TitleBar } from "./components/TitleBar";
import { ToolListBar } from "./components/ToolListBar";


// Tabのタイプ定義
export type Tab = "observe" | "send";

function App() {
  const [tab, setTab] = useState<Tab>("observe");

  return (
    <main>
      <TitleBar tab={tab} onTabChange={setTab} />
      {tab === "observe" ? <ToolListBar />: <p>a</p> }
    </main>
  );
}

export default App;
