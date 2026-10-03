import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../../tests/testUtils';
import { ApiError } from '@/api/client';
import { setLocale } from '@/i18n';
import { ImportLayoutDialog } from './ImportLayoutDialog';

const importProfile = vi.fn();
const activateProfile = vi.fn();
vi.mock('@/api/profiles', async (orig) => ({
  ...(await orig<typeof import('@/api/profiles')>()),
  importProfile: (...args: unknown[]) => importProfile(...args),
  activateProfile: (...args: unknown[]) => activateProfile(...args),
}));

const fileOf = (name: string, content: BlobPart = 'x') =>
  new File([content], name);

function setup(existing: string[] = ['default']) {
  const onImported = vi.fn();
  const onClose = vi.fn();
  renderWithProviders(
    <ImportLayoutDialog
      open
      onClose={onClose}
      existingNames={existing}
      onImported={onImported}
    />
  );
  const input = screen.getByTestId('import-file-input') as HTMLInputElement;
  return { onImported, onClose, input };
}

const nameBox = () => screen.getByLabelText('New profile name') as HTMLInputElement;
const submit = () => screen.getByRole('button', { name: 'Load layout' });

describe('ImportLayoutDialog', () => {
  beforeEach(() => {
    setLocale('en');
    importProfile.mockReset();
    activateProfile.mockReset();
  });

  it('loads a .krx picked with the file chooser under a name derived from the file', async () => {
    importProfile.mockResolvedValue({ name: 'my-layout', converted: true, warnings: [] });
    const { onImported, onClose, input } = setup();

    await userEvent.upload(input, fileOf('my layout.krx', 'KRX!'));
    expect(await screen.findByText(/Selected: my layout\.krx/)).toBeInTheDocument();
    expect(screen.getByText(/no source, so it is converted/)).toBeInTheDocument();
    expect(nameBox().value).toBe('my-layout');

    await userEvent.click(submit());
    await waitFor(() => expect(onImported).toHaveBeenCalledWith('my-layout'));
    expect(importProfile).toHaveBeenCalledWith('my-layout', 'krx', expect.any(Uint8Array));
    expect(onClose).toHaveBeenCalled();
    expect(activateProfile).not.toHaveBeenCalled();
  });

  it('accepts a dropped file and can activate it afterwards', async () => {
    importProfile.mockResolvedValue({ name: 'dropped', converted: false, warnings: [] });
    activateProfile.mockResolvedValue({ success: true, errors: [] });
    const { onImported } = setup();

    fireEvent.drop(screen.getByTestId('import-dropzone'), {
      dataTransfer: { files: [fileOf('dropped.rhai', 'device_start("*");')] },
    });
    await waitFor(() => expect(nameBox().value).toBe('dropped'));
    expect(screen.queryByText(/no source/)).toBeNull(); // only .krx is converted

    await userEvent.click(screen.getByLabelText('Activate it after loading'));
    await userEvent.click(submit());
    await waitFor(() => expect(onImported).toHaveBeenCalledWith('dropped'));
    expect(activateProfile).toHaveBeenCalledWith('dropped');
  });

  it.each([
    ['notes.txt', 'x', /Only \.krx and \.rhai/],
    ['empty.krx', '', /empty/],
  ])('refuses %s with a clear message and nothing to submit', async (name, body, message) => {
    const { input } = setup();
    await userEvent.upload(input, fileOf(name, body), { applyAccept: false });
    expect(await screen.findByRole('alert')).toHaveTextContent(message);
    expect(submit()).toBeDisabled();
    expect(importProfile).not.toHaveBeenCalled();
  });

  it('refuses a file over 1 MB without reading it into the request', async () => {
    const { input } = setup();
    const big = fileOf('big.rhai');
    Object.defineProperty(big, 'size', { value: 1024 * 1024 + 1 });
    await userEvent.upload(input, big);
    expect(await screen.findByRole('alert')).toHaveTextContent(/1 MB/);
    expect(submit()).toBeDisabled();
  });

  it('suggests a free name when the chosen one is taken locally', async () => {
    const { input } = setup(['default', 'mine']);
    await userEvent.upload(input, fileOf('mine.krx'));
    // Pre-filled with a free name already.
    expect(nameBox().value).toBe('mine-2');
    fireEvent.change(nameBox(), { target: { value: 'default' } });
    await userEvent.click(submit());
    expect(await screen.findByText(/already exists.*default-2/)).toBeInTheDocument();
    expect(importProfile).not.toHaveBeenCalled();
  });

  it('keeps the dialog open on a daemon 409 and offers another name', async () => {
    importProfile.mockRejectedValue(new ApiError('Profile already exists: race', 409));
    const { input, onClose, onImported } = setup([]);
    await userEvent.upload(input, fileOf('race.rhai'));
    await userEvent.click(submit());
    expect(await screen.findByText(/already exists.*race-2/)).toBeInTheDocument();
    expect(onClose).not.toHaveBeenCalled();
    expect(onImported).not.toHaveBeenCalled();
  });

  it('shows the daemon reason for a corrupt file', async () => {
    importProfile.mockRejectedValue(
      new ApiError('Invalid layout file: not a valid .krx file: bad magic', 400)
    );
    const { input, onClose } = setup([]);
    await userEvent.upload(input, fileOf('bad.krx'));
    await userEvent.click(submit());
    expect(await screen.findByRole('alert')).toHaveTextContent(/cannot be loaded.*bad magic/);
    expect(onClose).not.toHaveBeenCalled();
  });

  it('rejects an invalid typed name before calling the daemon', async () => {
    const { input } = setup([]);
    await userEvent.upload(input, fileOf('ok.rhai'));
    fireEvent.change(nameBox(), { target: { value: '../evil' } });
    await userEvent.click(submit());
    expect(await screen.findByText(/letters, numbers/i)).toBeInTheDocument();
    expect(importProfile).not.toHaveBeenCalled();
  });

  it('is keyboard reachable: the chooser button is a real button and Escape closes', async () => {
    const { onClose } = setup();
    const choose = screen.getByRole('button', { name: 'Choose file' });
    choose.focus();
    expect(choose).toHaveFocus();
    await userEvent.keyboard('{Escape}');
    expect(onClose).toHaveBeenCalled();
  });

  it('is available in Japanese', () => {
    setLocale('ja');
    setup();
    expect(
      screen.getByRole('button', { name: 'ファイルを選ぶ' })
    ).toBeInTheDocument();
    setLocale('en');
  });
});
