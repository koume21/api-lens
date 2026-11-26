// 小さなタブ切り替え（「ヘッダー / 本文」「送った内容 / 返ってきた内容」など）

type TabsProps<T extends string> = {
  tabs: { value: T; label: string }[];
  value: T;
  onChange: (value: T) => void;
};

export function Tabs<T extends string>({ tabs, value, onChange }: TabsProps<T>) {
  return (
    <div className="flex gap-4 border-b border-line px-4">
      {tabs.map((t) => (
        <button
          key={t.value}
          type="button"
          onClick={() => onChange(t.value)}
          // 選択中は下線を青にする。-mb-px で親の下線と重ねる
          className={`-mb-px border-b-2 py-2 text-xs ${
            t.value === value ? "border-accent text-fg" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          {t.label}
        </button>
      ))}
    </div>
  );
}
