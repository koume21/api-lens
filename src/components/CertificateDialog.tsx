// HTTPS 観測用の証明書の登録手順ダイアログ（F13）
//
// 設計の方針：アプリは証明書をキーチェーンに勝手に登録しない。
// ここでは「どこにあるか」「どう登録するか」を見せ、Finder で開くところまでを手伝う。

import { useEffect, useState } from "react";

import { caInfo, revealCaCert } from "../api/cert";
import type { CaInfo } from "../types";

export function CertificateDialog({ onClose }: { onClose: () => void }) {
  const [info, setInfo] = useState<CaInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [checking, setChecking] = useState(false);

  // 登録状態を調べ直す（手順どおり登録したあとに押してもらう）
  const refresh = async () => {
    setChecking(true);
    try {
      setInfo(await caInfo());
    } catch (e) {
      setError(String(e));
    } finally {
      setChecking(false);
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const addCommand = info
    ? `security add-trusted-cert -r trustRoot -k ~/Library/Keychains/login.keychain-db "${info.certPath}"`
    : "";
  const removeCommand = info ? `security delete-certificate -c "${info.commonName}" ~/Library/Keychains/login.keychain-db` : "";

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={onClose}>
      <div
        role="dialog"
        aria-label="HTTPS 証明書"
        className="flex max-h-[85vh] w-[620px] flex-col rounded-lg border border-line bg-panel shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-line px-4 py-3">
          <h2 className="text-sm font-bold">HTTPS の中身を見るための証明書</h2>
          <button type="button" className="text-muted hover:text-fg" onClick={onClose} aria-label="閉じる">
            ×
          </button>
        </div>

        <div className="flex min-h-0 flex-col gap-4 overflow-auto px-4 py-4 text-xs leading-relaxed text-muted">
          <p>
            本番などの HTTPS 通信は暗号化されているため、そのままでは中身が見えません。 このアプリは自分専用の「認証局」を作ってあり、
            <b className="text-fg">登録した観測対象ホストの HTTPS だけ</b>
            、この認証局で署名した証明書を使って中を開きます。 ブラウザにこの認証局を信頼してもらうため、Mac のキーチェーンへの登録が 1 回だけ必要です。
          </p>

          {/* 登録状態 */}
          <div className="flex items-center gap-3 rounded-md border border-line bg-base px-3 py-2">
            {info?.trusted === true && <span className="text-ok">● 信頼済みとして登録されています</span>}
            {info?.trusted === false && <span className="text-warn">● まだ信頼されていません</span>}
            {info && info.trusted === null && <span className="text-dim">● 登録状態を調べられませんでした</span>}
            {!info && !error && <span className="text-dim">調べています…</span>}
            <span className="flex-1" />
            <button type="button" disabled={checking} className="text-[11px] text-muted hover:text-fg" onClick={() => void refresh()}>
              ↻ 調べ直す
            </button>
          </div>

          {info && (
            <>
              <section className="flex flex-col gap-2">
                <h3 className="text-[13px] font-bold text-fg">登録の手順（キーチェーンアクセス）</h3>
                <ol className="flex list-decimal flex-col gap-1 pl-5">
                  <li>
                    <button
                      type="button"
                      className="rounded-md border border-line px-2 py-0.5 text-fg hover:bg-line/40"
                      onClick={() => revealCaCert().catch((e) => setError(String(e)))}
                    >
                      Finder で証明書を表示
                    </button>{" "}
                    し、<code className="font-mono text-fg">api-lens-ca.pem</code> をダブルクリックする（「ログイン」キーチェーンに追加）
                  </li>
                  <li>
                    キーチェーンアクセスで「<span className="text-fg">{info.commonName}</span>」をダブルクリックする
                  </li>
                  <li>
                    「信頼」を開き、「この証明書を使用するとき」を<b className="text-fg">「常に信頼」</b>にして閉じる（パスワードを聞かれます）
                  </li>
                  <li>ブラウザを再起動し、上の「調べ直す」で確認する</li>
                </ol>
              </section>

              <section className="flex flex-col gap-1">
                <h3 className="text-[13px] font-bold text-fg">ターミナルで登録する場合</h3>
                <p>アプリは実行しません。内容を確認してから、自分でターミナルに貼り付けてください。</p>
                <pre className="rounded bg-base p-2 font-mono text-[11px] break-all whitespace-pre-wrap text-fg select-text">{addCommand}</pre>
              </section>

              <section className="flex flex-col gap-1">
                <h3 className="text-[13px] font-bold text-fg">Firefox を使う場合</h3>
                <p>
                  Firefox は Mac のキーチェーンを見ません。設定 → プライバシーとセキュリティ → 証明書を表示 → 認証局証明書 →
                  「読み込む」で <code className="font-mono text-fg">api-lens-ca.pem</code> を選び、「ウェブサイトの識別に使用する」にチェックします。
                </p>
              </section>

              <section className="flex flex-col gap-1 rounded-md border border-warn/40 bg-warn/5 p-3">
                <h3 className="text-[13px] font-bold text-warn">注意</h3>
                <ul className="flex list-disc flex-col gap-1 pl-5">
                  <li>
                    同じフォルダの <code className="font-mono text-fg">api-lens-ca.key</code>（秘密鍵）は
                    <b className="text-fg">絶対に人に渡さない・アップロードしない</b>
                    でください。漏れると、この Mac のブラウザに対してどんなサイトにもなりすませてしまいます。
                  </li>
                  <li>証明書ピンニングをするアプリ・サイトは観測できません（通信が失敗します）。</li>
                  <li>
                    使わなくなったらキーチェーンアクセスで「{info.commonName}」を削除してください。ターミナルなら：
                    <pre className="mt-1 rounded bg-base p-2 font-mono text-[11px] break-all whitespace-pre-wrap text-fg select-text">
                      {removeCommand}
                    </pre>
                  </li>
                </ul>
              </section>

              <p className="font-mono text-[11px] break-all text-dim select-text">証明書の場所: {info.certPath}</p>
            </>
          )}

          {error && <p className="text-ng">{error}</p>}
        </div>
      </div>
    </div>
  );
}
