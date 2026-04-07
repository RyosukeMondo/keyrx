import React from 'react';
import { DiffEditor } from '@monaco-editor/react';

interface ProfileDiffViewProps {
  original: string;
  modified: string;
}

export const ProfileDiffView: React.FC<ProfileDiffViewProps> = ({
  original,
  modified,
}) => {
  return (
    <div className="border border-slate-600 rounded-md overflow-hidden">
      <DiffEditor
        original={original}
        modified={modified}
        language="rust"
        theme="vs-dark"
        options={{
          readOnly: true,
          renderSideBySide: true,
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          fontSize: 13,
        }}
        height="60vh"
      />
    </div>
  );
};
