import React, { useMemo } from 'react';
import { useLayoutData, type LayoutType } from '../contexts/LayoutPreviewContext';
import { cn } from '../utils/cn';

const BASE_KEY_SIZE = 52; // Base size from KeyboardVisualizer

interface LayoutPreviewProps {
  layout: LayoutType;
  scale?: number;
  showLabels?: boolean;
  className?: string;
}

interface MiniKeyProps {
  label: string;
  scale: number;
  showLabels: boolean;
  gridColumnSpan: number;
}

const MiniKey = React.memo<MiniKeyProps>(({ label, scale, showLabels, gridColumnSpan }) => {
  const keySize = BASE_KEY_SIZE * scale;
  const fontSize = Math.max(6, 10 * scale);

  return (
    <div
      className={cn(
        'flex items-center justify-center',
        'bg-slate-700 border border-slate-600 rounded-sm',
        'text-slate-300 font-medium',
        'overflow-hidden'
      )}
      style={{
        width: `${keySize * gridColumnSpan + (gridColumnSpan - 1) * 2}px`,
        height: `${keySize}px`,
        fontSize: `${fontSize}px`,
        lineHeight: 1,
      }}
      aria-hidden="true"
    >
      {showLabels && (
        <span className="truncate px-0.5">
          {label.length > 3 ? label.slice(0, 2) : label}
        </span>
      )}
    </div>
  );
});

MiniKey.displayName = 'MiniKey';

export const LayoutPreview = React.memo<LayoutPreviewProps>(({
  layout,
  scale = 0.4,
  showLabels = true,
  className = '',
}) => {
  const layoutData = useLayoutData(layout);

  const gridStyle = useMemo(() => {
    const keySize = BASE_KEY_SIZE * scale;
    return {
      display: 'grid',
      gridTemplateRows: `repeat(${layoutData.dimensions.rows}, ${keySize}px)`,
      gridTemplateColumns: `repeat(${layoutData.dimensions.cols}, ${keySize}px)`,
      gap: '2px',
    };
  }, [layoutData.dimensions, scale]);

  return (
    <div
      className={cn(
        'p-2 bg-slate-900 rounded-lg border border-slate-700',
        className
      )}
      role="img"
      aria-label={`${layoutData.name} keyboard layout preview`}
    >
      <div className="text-xs text-slate-400 mb-2 font-medium">
        {layoutData.name} ({layoutData.keys.length} keys)
      </div>
      <div style={gridStyle}>
        {layoutData.keys.map((key) => (
          <div
            key={key.keyCode}
            style={{
              gridRow: key.gridRow,
              gridColumn: `${key.gridColumn} / span ${key.gridColumnSpan}`,
            }}
          >
            <MiniKey
              label={key.label}
              scale={scale}
              showLabels={showLabels}
              gridColumnSpan={key.gridColumnSpan}
            />
          </div>
        ))}
      </div>
    </div>
  );
});

LayoutPreview.displayName = 'LayoutPreview';
