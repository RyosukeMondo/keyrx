import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';
import { test, expect, type Page } from '@playwright/test';

/**
 * "Load layout from file" against a scratch daemon: file picker, drag and
 * drop, every shipped example .krx, collisions, and invalid files.
 */

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const KRX = [
  'examples/user_layout.krx',
  'uat_tests/test1_simple.krx',
  'uat_tests/test2_modifiers.krx',
  'uat_tests/test3_locks.krx',
  'uat_tests/test4_chords.krx',
  'uat_tests/test5_vim.krx',
  'uat_tests/test9_multidevice.krx',
];

async function openDialog(page: Page) {
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  // On a phone the profile list sits behind the "profiles" button.
  const open = page.getByRole('button', { name: /load layout from file/i });
  if (!(await open.isVisible())) {
    await page.getByRole('button', { name: /profiles?/i }).first().click();
  }
  await open.click();
  await expect(page.getByRole('dialog')).toBeVisible();
}

async function pick(page: Page, file: string) {
  await page.getByTestId('import-file-input').setInputFiles(path.join(REPO, file));
}

test.describe('load layout from file', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  for (const [i, file] of KRX.entries()) {
    test(`imports ${file} and the editor shows its mappings`, async ({ page }) => {
      const name = `e2e-${i}-${path.basename(file, '.krx')}`;
      await openDialog(page);
      await pick(page, file);
      await page.getByLabel('New profile name').fill(name);
      await page.getByRole('dialog').getByRole('button', { name: 'Load layout', exact: true }).click();
      await expect(page.getByRole('dialog')).toBeHidden();

      const config = await page.request.get(`/api/profiles/${name}/config`);
      const { source } = await config.json();
      expect(source).toContain('device_start(');
      // The new profile is selected (not bounced back to the first one) and
      // the visual editor is showing it.
      await expect(page).toHaveURL(new RegExp(`/profiles/${name}/config$`));
      await expect(page.getByText('Global Keys')).toBeVisible();
      if (file.endsWith('test5_vim.krx')) {
        // Caps/Space became layer modifiers: their layers appear as tabs.
        await expect(page.getByRole('button', { name: 'MD_00' })).toBeVisible();
        await expect(page.getByRole('button', { name: 'MD_01' })).toBeVisible();
      }
      await page.screenshot({ path: test.info().outputPath(`${name}.png`) });
    });
  }

  test('a taken name keeps the dialog open and suggests a free one', async ({ page }) => {
    await openDialog(page);
    await pick(page, 'uat_tests/test1_simple.krx');
    await page.getByLabel('New profile name').fill('default');
    await page.getByRole('dialog').getByRole('button', { name: 'Load layout', exact: true }).click();
    await expect(page.getByText(/already exists/i)).toBeVisible();
    await expect(page.getByText(/default-2/)).toBeVisible();
    await expect(page.getByRole('dialog')).toBeVisible();
  });

  test('wrong, empty and corrupt files get a clear error and nothing is stored', async ({ page }, info) => {
    await openDialog(page);
    const dir = info.outputPath('bad');
    fs.mkdirSync(dir, { recursive: true });
    const write = (n: string, data: Buffer | string) => {
      const p = path.join(dir, n);
      fs.writeFileSync(p, data);
      return p;
    };

    await page.getByTestId('import-file-input').setInputFiles(write('notes.txt', 'hi'));
    await expect(page.getByRole('alert')).toContainText(/\.krx and \.rhai/);

    await page.getByTestId('import-file-input').setInputFiles(write('empty.krx', ''));
    await expect(page.getByRole('alert')).toContainText(/empty/i);

    await page
      .getByTestId('import-file-input')
      .setInputFiles(write('big.rhai', Buffer.alloc(1024 * 1024 + 1, 0x20)));
    await expect(page.getByRole('alert')).toContainText(/1 MB/);

    await page
      .getByTestId('import-file-input')
      .setInputFiles(write('corrupt-e2e.krx', Buffer.from('not a krx file at all, but long enough to hold a header')));
    await page.getByLabel('New profile name').fill('corrupt-e2e');
    await page.getByRole('dialog').getByRole('button', { name: 'Load layout', exact: true }).click();
    await expect(page.getByRole('alert')).toContainText(/cannot be loaded/i);
    const list = await (await page.request.get('/api/profiles')).json();
    expect(list.profiles.map((p: { name: string }) => p.name)).not.toContain('corrupt-e2e');
  });

  test('drag and drop loads a .rhai file', async ({ page }) => {
    await openDialog(page);
    const source = 'device_start("*");\n  map("A", "VK_B");\ndevice_end();\n';
    const data = await page.evaluateHandle((text) => {
      const dt = new DataTransfer();
      dt.items.add(new File([text], 'dropped.rhai', { type: 'text/plain' }));
      return dt;
    }, source);
    await page.getByTestId('import-dropzone').dispatchEvent('drop', { dataTransfer: data });
    await expect(page.getByLabel('New profile name')).toHaveValue('dropped');
    await page.getByRole('dialog').getByRole('button', { name: 'Load layout', exact: true }).click();
    await expect(page.getByRole('dialog')).toBeHidden();
    const { source: stored } = await (await page.request.get('/api/profiles/dropped/config')).json();
    expect(stored).toBe(source);
  });

  test('works from the keyboard and is labelled in Japanese', async ({ page }) => {
    await page.goto('/?lang=ja');
    await page.waitForLoadState('networkidle');
    const open = page.getByRole('button', { name: 'ファイルからレイアウトを読み込む' });
    await open.focus();
    await page.keyboard.press('Enter');
    await expect(page.getByRole('dialog')).toBeVisible();
    await page.keyboard.press('Tab');
    const focused = await page.evaluate(() => document.activeElement?.textContent ?? '');
    expect(focused.length).toBeGreaterThan(0);
    await page.keyboard.press('Escape');
    await expect(page.getByRole('dialog')).toBeHidden();
  });
});

test.describe('load layout on a phone', () => {
  test.use({ viewport: { width: 390, height: 800 } });

  test('fits the screen and imports', async ({ page }) => {
    await openDialog(page);
    await pick(page, 'uat_tests/test5_vim.krx');
    await page.getByLabel('New profile name').fill('phone-vim');
    const box = await page.getByRole('dialog').boundingBox();
    expect(box && box.x >= 0 && box.x + box.width <= 390).toBeTruthy();
    await page.screenshot({ path: test.info().outputPath('phone-dialog.png') });
    await page.getByRole('dialog').getByRole('button', { name: 'Load layout', exact: true }).click();
    await expect(page.getByRole('dialog')).toBeHidden();
  });
});
