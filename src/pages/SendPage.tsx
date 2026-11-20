// 画面2：送信（API クライアントに近い形）
// 上：リクエスト入力（F1〜F4）／下：結果（F5〜F8）

import { useState } from "react";

import { sendRequest } from "../api/client";
import { RequestEditor } from "../components/RequestEditor";
import { ResponseView } from "../components/ResponseView";
import type { Exchange, RequestInput } from "../types";

/** 送信画面を開いたときの初期値 */
export const EMPTY_REQUEST: RequestInput = {
  method: "GET",
  url: "",
  headers: [{ name: "", value: "" }],
  body: "",
};

export function SendPage() {
  const [request, setRequest] = useState<RequestInput>(EMPTY_REQUEST);
  const [result, setResult] = useState<Exchange | null>(null);
  const [sending, setSending] = useState(false);

  const send = async () => {
    setSending(true);
    try {
      setResult(await sendRequest(request));
    } catch (e) {
      // 送信自体の失敗は result.error に入るので、ここに来るのは invoke が失敗したとき（Rust 側の想定外）
      alert(`送信できませんでした: ${String(e)}`);
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <RequestEditor request={request} onChange={setRequest} onSend={send} sending={sending} />
      <ResponseView result={result} />
    </div>
  );
}
