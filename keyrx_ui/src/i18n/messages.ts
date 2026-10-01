/**
 * UI message catalogs.
 *
 * Only the first-run / safe-save / status surfaces are localized so far (see
 * i18n/index.ts). English is the source of truth: `MessageKey` is derived from
 * it and the Japanese catalog is typed against it, so a missing translation is
 * a compile error rather than a silent English string.
 */

export type Message = string | { one: string; other: string };

export const en = {
  'status.active': 'Active: {profile}',
  'status.noProfile': 'No active profile',
  'status.keyboards': {
    one: '{count} keyboard grabbed',
    other: '{count} keyboards grabbed',
  },
  'status.offline': 'Daemon offline',
  'status.label': 'Daemon status',

  'guide.title': 'What do you want this keyboard to do?',
  'guide.subtitle': 'Pick a path—we’ll set the right scope and tools.',
  'guide.connectFirst': 'Connect a device first',
  'guide.swap.eyebrow': 'Quick fix',
  'guide.swap.title': 'Swap two keys',
  'guide.swap.description':
    'Swap Caps Lock and Ctrl, or any two keys, in one step.',
  'guide.swap.outcome': 'Best for your everyday keyboard',
  'guide.simple.eyebrow': 'Quick fix',
  'guide.simple.title': 'Change a few keys',
  'guide.simple.description':
    'Swap Caps Lock, fix an awkward key, or add a media control.',
  'guide.simple.outcome': 'Best for your everyday keyboard',
  'guide.pad.eyebrow': 'Reuse a gadget',
  'guide.pad.title': 'Make a command pad',
  'guide.pad.description':
    'Turn a cheap numpad or spare keyboard into shortcut buttons.',
  'guide.pad.outcome': 'Best for creators and productivity',
  'guide.advanced.eyebrow': 'Power setup',
  'guide.advanced.title': 'Build a full layout',
  'guide.advanced.description':
    'Use layers, tap/hold behavior, macros, and editable Rhai.',
  'guide.advanced.outcome': 'Best for keyboard enthusiasts',

  'swap.title': 'Swap two keys',
  'swap.first': 'First key',
  'swap.second': 'Second key',
  'swap.apply': 'Swap them',
  'swap.hint': 'Pick two different keys. Each will act like the other.',
  'swap.preset': 'Caps Lock ↔ Ctrl',
  'swap.cancel': 'Cancel',

  'save.title': 'Review changes',
  'save.intro': 'These changes will be saved to this profile:',
  'save.none': 'No key changes.',
  'save.added': '{from} → {to}',
  'save.removed': '{from} is back to normal',
  'save.details': 'Details',
  'save.hideDetails': 'Hide details',
  'save.cancel': 'Cancel',
  'save.confirm': 'Save changes',
} as const satisfies Record<string, Message>;

export type MessageKey = keyof typeof en;

export const ja: Record<MessageKey, Message> = {
  'status.active': '使用中: {profile}',
  'status.noProfile': '有効なプロファイルなし',
  'status.keyboards': {
    one: 'キーボード {count} 台を制御中',
    other: 'キーボード {count} 台を制御中',
  },
  'status.offline': 'デーモンに接続できません',
  'status.label': 'デーモンの状態',

  'guide.title': 'このキーボードで何をしたいですか？',
  'guide.subtitle': '目的を選ぶと、適切な範囲とツールを用意します。',
  'guide.connectFirst': '先にキーボードを接続してください',
  'guide.swap.eyebrow': 'かんたん設定',
  'guide.swap.title': '2つのキーを入れ替える',
  'guide.swap.description':
    'Caps Lock と Ctrl など、2つのキーを一度に入れ替えます。',
  'guide.swap.outcome': '普段使いのキーボード向け',
  'guide.simple.eyebrow': 'かんたん設定',
  'guide.simple.title': 'キーを少しだけ変える',
  'guide.simple.description':
    'Caps Lock の入れ替えや、押しにくいキーの修正、メディアキーの追加ができます。',
  'guide.simple.outcome': '普段使いのキーボード向け',
  'guide.pad.eyebrow': '余ったキーボードの活用',
  'guide.pad.title': 'コマンドパッドを作る',
  'guide.pad.description':
    '安価なテンキーや予備のキーボードを、ショートカットボタンにします。',
  'guide.pad.outcome': 'クリエイティブ作業・仕事の効率化向け',
  'guide.advanced.eyebrow': '上級者向け',
  'guide.advanced.title': 'レイアウトを一から作る',
  'guide.advanced.description':
    'レイヤー、タップ/ホールド、マクロ、Rhai の直接編集が使えます。',
  'guide.advanced.outcome': 'キーボード愛好家向け',

  'swap.title': '2つのキーを入れ替える',
  'swap.first': '1つ目のキー',
  'swap.second': '2つ目のキー',
  'swap.apply': '入れ替える',
  'swap.hint':
    '異なる2つのキーを選んでください。お互いの役割が入れ替わります。',
  'swap.preset': 'Caps Lock ↔ Ctrl',
  'swap.cancel': 'キャンセル',

  'save.title': '変更内容の確認',
  'save.intro': '次の変更をこのプロファイルに保存します。',
  'save.none': 'キーの変更はありません。',
  'save.added': '{from} → {to}',
  'save.removed': '{from} を元に戻す',
  'save.details': '詳細',
  'save.hideDetails': '詳細を隠す',
  'save.cancel': 'キャンセル',
  'save.confirm': '変更を保存',
};
