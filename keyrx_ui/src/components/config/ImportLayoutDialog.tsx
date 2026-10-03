import React, { useRef, useState } from 'react';
import { FileUp } from 'lucide-react';
import { Modal } from '@/components/Modal';
import { Input } from '@/components/Input';
import { Button } from '@/components/Button';
import { useImportProfile, useActivateProfile } from '@/hooks/useProfiles';
import { useToast } from '@/hooks/useToast';
import { ApiError } from '@/api/client';
import { t, type MessageKey } from '@/i18n';
import {
  detectLayoutFormat,
  isValidProfileName,
  layoutFileProblem,
  profileNameFromFile,
  readFileBytes,
  uniqueProfileName,
  type LayoutFormat,
  type LayoutFileProblem,
} from '@/utils/layoutFile';

export interface ImportLayoutDialogProps {
  open: boolean;
  onClose: () => void;
  /** Names already taken, to pre-empt a collision. */
  existingNames: readonly string[];
  /** Called with the new profile's name once it is stored. */
  onImported: (name: string) => void;
}

const PROBLEM_MESSAGE: Record<LayoutFileProblem, MessageKey> = {
  type: 'import.err.type',
  empty: 'import.err.empty',
  tooLarge: 'import.err.tooLarge',
};

interface Picked {
  fileName: string;
  size: number;
  format: LayoutFormat;
  bytes: Uint8Array;
}

function formatSize(bytes: number): string {
  return bytes < 1024 ? `${bytes} B` : `${(bytes / 1024).toFixed(1)} KB`;
}

/**
 * "Load layout from file": pick (button) or drop a `.krx` / `.rhai`, name the
 * new profile, optionally activate it. The daemon validates and converts; a
 * taken name keeps the dialog open with a suggested free name.
 * Mount it only while open: closing unmounts it, which resets every field.
 */
export const ImportLayoutDialog: React.FC<ImportLayoutDialogProps> = ({
  open,
  onClose,
  existingNames,
  onImported,
}) => {
  const importMutation = useImportProfile();
  const activateMutation = useActivateProfile();
  const toast = useToast();
  const inputRef = useRef<HTMLInputElement>(null);

  const [picked, setPicked] = useState<Picked | null>(null);
  const [name, setName] = useState('');
  const [activate, setActivate] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [fileError, setFileError] = useState('');
  const [nameError, setNameError] = useState('');

  const acceptFile = async (file: File | undefined) => {
    if (!file) return;
    setNameError('');
    const problem = layoutFileProblem(file);
    const format = detectLayoutFormat(file.name);
    if (problem || !format) {
      setPicked(null);
      setFileError(t(PROBLEM_MESSAGE[problem ?? 'type']));
      return;
    }
    try {
      const bytes = await readFileBytes(file);
      setPicked({ fileName: file.name, size: file.size, format, bytes });
      setName(uniqueProfileName(profileNameFromFile(file.name), existingNames));
      setFileError('');
    } catch {
      setPicked(null);
      setFileError(t('import.err.read'));
    }
  };

  const submit = async () => {
    if (!picked || importMutation.isPending) return;
    if (!isValidProfileName(name)) {
      setNameError(t('import.err.name'));
      return;
    }
    if (existingNames.includes(name)) {
      setNameError(
        t('import.err.taken', {
          name,
          suggestion: uniqueProfileName(name, existingNames),
        })
      );
      return;
    }
    try {
      const result = await importMutation.mutateAsync({
        name,
        format: picked.format,
        bytes: picked.bytes,
      });
      if (activate) await activateMutation.mutateAsync(result.name);
      toast.success(
        t(result.converted ? 'import.doneConverted' : 'import.done', {
          name: result.name,
        })
      );
      onImported(result.name);
      onClose();
    } catch (err) {
      if (err instanceof ApiError && err.statusCode === 409) {
        setNameError(
          t('import.err.taken', {
            name,
            suggestion: uniqueProfileName(name, [...existingNames, name]),
          })
        );
      } else {
        const reason = err instanceof Error ? err.message : String(err);
        setFileError(t('import.err.rejected', { reason }));
      }
    }
  };

  const busy = importMutation.isPending || activateMutation.isPending;

  return (
    <Modal open={open} onClose={onClose} title={t('import.title')}>
      <div className="flex flex-col gap-md">
        <p className="text-sm text-slate-300">{t('import.help')}</p>

        <div
          data-testid="import-dropzone"
          onDragOver={(e) => {
            e.preventDefault();
            setDragging(true);
          }}
          onDragLeave={() => setDragging(false)}
          onDrop={(e) => {
            e.preventDefault();
            setDragging(false);
            void acceptFile(e.dataTransfer.files[0]);
          }}
          className={`flex flex-col items-center gap-3 rounded-md border-2 border-dashed p-4 text-center transition-colors ${
            dragging
              ? 'border-primary-500 bg-primary-500/10'
              : 'border-slate-600 bg-slate-800/50'
          }`}
        >
          <FileUp size={28} className="text-slate-400" aria-hidden="true" />
          <p className="text-sm text-slate-300">
            {dragging ? t('import.dropActive') : t('import.drop')}
          </p>
          <input
            ref={inputRef}
            type="file"
            accept=".krx,.rhai"
            className="sr-only"
            tabIndex={-1}
            aria-hidden="true"
            data-testid="import-file-input"
            onChange={(e) => {
              void acceptFile(e.target.files?.[0]);
              e.target.value = ''; // picking the same file again must re-fire
            }}
          />
          <Button
            variant="secondary"
            size="md"
            aria-label={picked ? t('import.change') : t('import.choose')}
            onClick={() => inputRef.current?.click()}
          >
            {picked ? t('import.change') : t('import.choose')}
          </Button>
          {picked && (
            <p className="text-sm text-slate-100 break-all">
              {t('import.selected', {
                name: picked.fileName,
                size: formatSize(picked.size),
              })}
            </p>
          )}
        </div>

        {fileError && (
          <p role="alert" className="text-sm text-red-400 break-words">
            {fileError}
          </p>
        )}

        {picked && (
          <>
            {picked.format === 'krx' && (
              <p className="text-xs text-slate-400">{t('import.krxNote')}</p>
            )}
            <Input
              type="text"
              value={name}
              onChange={(value) => {
                setName(value);
                setNameError('');
              }}
              label={t('import.name')}
              aria-label={t('import.name')}
              helpText={t('import.nameHelp')}
              error={nameError}
              maxLength={64}
            />
            <label className="flex items-center gap-2 text-sm text-slate-300">
              <input
                type="checkbox"
                checked={activate}
                onChange={(e) => setActivate(e.target.checked)}
              />
              {t('import.activate')}
            </label>
          </>
        )}

        <div className="flex gap-2 justify-end mt-2">
          <Button
            variant="secondary"
            size="md"
            onClick={onClose}
            aria-label={t('import.cancel')}
          >
            {t('import.cancel')}
          </Button>
          <Button
            variant="primary"
            size="md"
            onClick={() => void submit()}
            disabled={!picked || busy}
            loading={busy}
            aria-label={t('import.submit')}
          >
            {busy ? t('import.working') : t('import.submit')}
          </Button>
        </div>
      </div>
    </Modal>
  );
};

ImportLayoutDialog.displayName = 'ImportLayoutDialog';
