// 画面2：送信（API クライアントに近い形）
// 左：保存したリクエストの一覧・送信履歴（F10）
// 右上：リクエスト入力（F1〜F4）と保存（F9）／右下：結果（F5〜F8）

import { useEffect, useState } from "react";

import {
  deleteSavedRequest,
  listSavedRequests,
  listSendHistory,
  saveRequest,
  sendRequest,
  updateSavedRequest,
} from "../api/client";
import { RequestEditor } from "../components/RequestEditor";
import { ResponseView } from "../components/ResponseView";
import { SavedRequestList } from "../components/SavedRequestList";
import { SaveRequestButton } from "../components/SaveRequestButton";
import type { Exchange, RequestInput, SavedRequest } from "../types";
import { exchangeToRequest } from "../utils/request";

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

  const [saved, setSaved] = useState<SavedRequest[]>([]);
  const [history, setHistory] = useState<Exchange[]>([]);
  /** 保存済みのものを読み込んでいるとき、その保存リクエスト（上書き保存の相手） */
  const [current, setCurrent] = useState<SavedRequest | null>(null);
  /** 画面上部に出すエラー（DB の失敗など）。WebView では alert が使えないことがあるため画面に出す */
  const [error, setError] = useState<string | null>(null);

  // 起動時に、前回までに保存したもの・送信履歴を DB から読む（F10: 再起動しても残る）
  useEffect(() => {
    Promise.all([listSavedRequests(), listSendHistory()])
      .then(([s, h]) => {
        setSaved(s);
        setHistory(h);
      })
      .catch((e) => setError(String(e)));
  }, []);

  const send = async () => {
    setSending(true);
    setError(null);
    try {
      const ex = await sendRequest(request);
      setResult(ex);
      // 履歴の先頭に足す（DB から読み直すより速い。件数は Rust 側の上限と同じ 100 に合わせる）
      setHistory((h) => [ex, ...h].slice(0, 100));
    } catch (e) {
      // 送信自体の失敗は result.error に入るので、ここに来るのは記録（DB）の失敗など
      setError(String(e));
    } finally {
      setSending(false);
    }
  };

  const save = async (name: string, overwriteId: number | null) => {
    setError(null);
    try {
      const s = overwriteId === null ? await saveRequest(name, request) : await updateSavedRequest(overwriteId, name, request);
      setCurrent(s);
      setSaved(await listSavedRequests());
    } catch (e) {
      setError(String(e));
      throw e; // 入力欄を閉じないように、呼び出し元にも失敗を伝える
    }
  };

  const remove = async (s: SavedRequest) => {
    setError(null);
    try {
      await deleteSavedRequest(s.id);
      setSaved((list) => list.filter((x) => x.id !== s.id));
      if (current?.id === s.id) setCurrent(null);
    } catch (e) {
      setError(String(e));
    }
  };

  // 保存したものを読み込む。入力欄が 0 行だと追加しづらいので空行を足す
  const loadSaved = (s: SavedRequest) => {
    setCurrent(s);
    setRequest({
      ...s.request,
      headers: s.request.headers.length > 0 ? s.request.headers : [{ name: "", value: "" }],
    });
  };

  // 履歴から読み込むときは、どの保存リクエストとも結びつけない（上書きの事故を防ぐ）
  const loadHistory = (ex: Exchange) => {
    setCurrent(null);
    setRequest(exchangeToRequest(ex));
    setResult(ex);
  };

  return (
    <div className="flex h-full min-h-0">
      <SavedRequestList
        saved={saved}
        history={history}
        selectedId={current?.id ?? null}
        onSelectSaved={loadSaved}
        onDeleteSaved={remove}
        onSelectHistory={loadHistory}
      />
      <div className="flex min-w-0 flex-1 flex-col">
        {error && (
          <p className="flex items-center justify-between border-b border-line bg-ng/10 px-4 py-2 text-xs text-ng">
            {error}
            <button type="button" onClick={() => setError(null)} aria-label="閉じる">
              ×
            </button>
          </p>
        )}
        {current && (
          <p className="border-b border-line px-4 pt-3 text-xs text-muted">
            保存済み: <span className="text-fg">{current.name}</span>
          </p>
        )}
        <RequestEditor
          request={request}
          onChange={setRequest}
          onSend={send}
          sending={sending}
          actions={<SaveRequestButton current={current} onSave={save} />}
        />
        <ResponseView result={result} />
      </div>
    </div>
  );
}
