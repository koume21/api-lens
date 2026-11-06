// Figma: API通信観測 画面デザイン > 観測画面 > タイトルバー
// 左上の赤・黄・緑の丸は Mac の本物を使う（tauri.conf.json の titleBarStyle: "Overlay"）。
// 丸と重ならないよう左に 80px あけ、data-tauri-drag-region でバーをつかんでウィンドウを動かせるようにしている

import type { Tab } from "../App";

type TitleBarProps = {
    tab: Tab;
    // タブが押されたら、押されたタブを親に知らせる
    onTabChange: (tab: Tab) => void;
};

// タブボタンの見た目。共通部分と、選択中／非選択で変わる部分に分ける
const tabBase = "rounded-md px-4 py-[5px] text-[13px]";
const tabActive = "bg-[#2563eb] font-bold text-white";
const tabInactive = "bg-[#101215] text-[#9aa3ad]";

export function TitleBar({ tab, onTabChange }: TitleBarProps) {
    return (
        <div data-tauri-drag-region className="flex h-12 items-center gap-2 border-b border-[#2a2f36] bg-[#101215] pr-[14px] pl-[80px] font-['Noto_Sans_JP',sans-serif]">
            {/* アプリ名 */}
            <p className="text-xs whitespace-nowrap text-[#9aa3ad]">API通信観測</p>

            {/* アプリ名とタブの間のすき間 */}
            <div className="w-3" />

            {/* 画面切り替えタブ（tab の値で選択中のボタンを青くする） */}
            <div className="flex items-center gap-1">
                <button className={`${tabBase} ${tab === "observe" ? tabActive : tabInactive}`} onClick={() => onTabChange("observe")}>
                    観測
                </button>
                <button className={`${tabBase} ${tab === "send" ? tabActive : tabInactive}`} onClick={() => onTabChange("send")}>
                    送信
                </button>
            </div>
        </div>
    );
}
