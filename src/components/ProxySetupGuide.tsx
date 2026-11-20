// 一覧が空のときに出す「観測の始め方」
// プロキシ設定はブラウザごとに違い、localhost の扱いでつまずきやすいので、手順を画面に出しておく

import type { ObservedHost, ObserveStatus } from "../types";

type ProxySetupGuideProps = {
  status: ObserveStatus;
  hosts: ObservedHost[];
  port: string;
};

export function ProxySetupGuide({ status, hosts, port }: ProxySetupGuideProps) {
  const addr = status.addr ?? `127.0.0.1:${port || "8080"}`;
  const [host, portNum] = addr.split(":");

  return (
    <div className="mx-auto flex max-w-[560px] flex-col gap-4 p-8 text-xs leading-relaxed text-muted">
      <h2 className="text-sm font-bold text-fg">観測の始め方</h2>
      <ol className="flex list-decimal flex-col gap-3 pl-5">
        <li className={hosts.length > 0 ? "text-dim line-through" : ""}>
          「⚙ 対象ホスト」で、見たい API のホスト（例: <code className="font-mono text-fg">localhost:3000</code>）を登録する。
          登録していないホストの通信は記録しません。
        </li>
        <li className={status.running ? "text-dim line-through" : ""}>「▶ 観測を開始」を押す。</li>
        <li>
          ブラウザのプロキシを <code className="font-mono text-fg">{addr}</code>（HTTP）に設定する。
          <ul className="mt-1 flex list-disc flex-col gap-1 pl-5">
            <li>
              <b className="text-fg">Firefox</b>: 設定 → ネットワーク設定 → 手動でプロキシーを設定 → HTTP プロキシー{" "}
              <code className="font-mono text-fg">{host}</code> ポート <code className="font-mono text-fg">{portNum}</code>
              （「このプロキシーを HTTPS でも使用する」にチェック）。
              <br />
              localhost を観測するときは <code className="font-mono text-fg">about:config</code> で{" "}
              <code className="font-mono text-fg">network.proxy.allow_hijacking_localhost</code> を true にする。
            </li>
            <li>
              <b className="text-fg">Chrome</b>: 観測用に別のプロファイルで起動するのがおすすめ。
              <pre className="mt-1 rounded bg-base p-2 font-mono text-[11px] whitespace-pre-wrap text-fg select-text">
                {`open -na "Google Chrome" --args --user-data-dir=/tmp/api-lens-chrome --proxy-server="http://${addr}" --proxy-bypass-list="<-loopback>"`}
              </pre>
              <code className="font-mono">{"<-loopback>"}</code> は「localhost もプロキシを通す」指定（ないと localhost は素通りされる）。
            </li>
          </ul>
        </li>
        <li>ブラウザで自分のアプリを操作すると、ここに通信が 1 行ずつ増えていきます。</li>
      </ol>
      <p className="text-[11px] text-dim">
        記録した通信はこの PC の中だけに保存し、外には送りません。プロキシは 127.0.0.1 だけで待ち受けるので、ほかの PC からは使えません。
      </p>
    </div>
  );
}
