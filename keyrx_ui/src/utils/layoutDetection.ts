import type { LayoutType } from '@/components/KeyboardVisualizer';

/**
 * Auto-detect keyboard layout from Rhai config source.
 * Checks for JIS-specific key codes to determine if the config
 * is for a Japanese keyboard layout.
 */
export function detectLayoutFromSource(source: string | undefined): LayoutType {
  if (!source) return 'JIS_109';
  const jisKeys = [
    'VK_Zenkaku',
    'VK_全角',
    'VK_無変換',
    'VK_変換',
    'VK_ひらがな',
    'VK_カタカナ',
    'VK_Ro',
    'VK_Yen',
    'VK_Henkan',
    'VK_Muhenkan',
  ];
  const hasJisKeys = jisKeys.some((k) => source.includes(k));
  return hasJisKeys ? 'JIS_109' : 'ANSI_104';
}
