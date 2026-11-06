// 観測画面の「ツールバー」（見た目だけ）
// Figma: API通信観測 画面デザイン > 観測画面 > ツールバー
// クリックしたときの処理や状態はまだ入れていない
export function ToolListBar() {
    return (
        <div className="flex h-[53px] items-center gap-[10px] border-b border-[#2a2f36] bg-[#16191d] px-4 font-['Noto_Sans_JP',sans-serif]">
            {/* 観測を停止ボタン（観測中なので赤） */}
            <button className="h-8 rounded-md border border-[#ef4444] px-[15px] text-[13px] whitespace-nowrap text-[#ef4444]">
                ■ 観測を停止
            </button>

            {/* 観測状態（緑の丸＋待ち受けアドレス） */}
            <div className="flex items-center gap-[6px]">
                <span className="size-2 rounded-full bg-[#22c55e]" />
                <p className="text-xs whitespace-nowrap text-[#22c55e]">
                    観測中（<span className="font-['JetBrains_Mono',monospace]">127.0.0.1:8080</span>）
                </p>
            </div>

            {/* 対象の切り替え（見た目だけのプルダウン） */}
            <div className="flex h-8 items-center gap-2 rounded-md border border-[#2a2f36] bg-[#101215] px-[11px] text-xs whitespace-nowrap">
                <span className="text-[#9aa3ad]">対象：</span>
                <span className="text-[#e6e8eb]">すべて</span>
                <span className="text-[#9aa3ad]">▾</span>
            </div>

            {/* 対象ホストの設定ボタン */}
            <button className="h-8 rounded-md border border-[#2a2f36] px-[15px] text-[13px] whitespace-nowrap text-[#e6e8eb]">
                ⚙ 対象ホスト
            </button>

            {/* URL で絞り込む入力欄 */}
            <input
                type="text"
                placeholder="🔍 URLで絞り込み（例: /api/users）"
                className="h-8 w-[280px] rounded-md border border-[#2a2f36] bg-[#101215] px-[11px] text-[13px] text-[#e6e8eb] placeholder:text-[#6b7480]"
            />

            {/* すべて／成功／失敗の切り替え（いまは「すべて」が選ばれている見た目） */}
            <div className="flex h-[30px] items-center rounded-md border border-[#2a2f36] p-px text-xs whitespace-nowrap">
                <span className="flex h-7 items-center rounded-[5px] bg-[#2a2f36] px-3 text-[#e6e8eb]">すべて</span>
                <span className="flex h-7 items-center px-3 text-[#22c55e]">成功</span>
                <span className="flex h-7 items-center px-3 text-[#ef4444]">失敗</span>
            </div>

            {/* 右端に寄せるためのすき間 */}
            <div className="flex-1" />

            {/* 履歴を削除ボタン */}
            <button className="h-8 rounded-md border border-[#2a2f36] px-[15px] text-[13px] whitespace-nowrap text-[#9aa3ad]">
                🗑 履歴を削除
            </button>
        </div>
    );
}
