import React from 'react';
import { Button } from '../Button';
import { WasmStatusBadge } from '../WasmStatusBadge';

interface SimulatorHeaderProps {
  isLoadingWasm: boolean;
  isWasmReady: boolean;
  error: Error | null;
  eventCount: number;
  onCopyLog: () => void;
  onReset: () => void;
}

/**
 * Header section for the simulator page with title, status badge, and action buttons
 */
export const SimulatorHeader: React.FC<SimulatorHeaderProps> = ({
  isLoadingWasm,
  isWasmReady,
  error,
  eventCount,
  onCopyLog,
  onReset,
}) => {
  return (
    <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
      <div className="flex-1">
        <div className="flex items-center gap-3 flex-wrap">
          <h1 className="text-xl md:text-2xl lg:text-3xl font-bold text-slate-100">
            Keyboard Simulator
          </h1>
          <WasmStatusBadge
            isLoading={isLoadingWasm}
            isReady={isWasmReady}
            error={error}
            className="shrink-0"
          />
        </div>
        <p className="text-sm md:text-base text-slate-400 mt-2">
          Test your configuration by clicking keys or typing. Changes are not
          saved to your keyboard.
        </p>
      </div>
      <div className="flex flex-col sm:flex-row gap-2">
        <Button
          variant="secondary"
          size="md"
          onClick={onCopyLog}
          aria-label="Copy event log to clipboard"
          disabled={eventCount === 0}
          className="w-full sm:w-auto min-h-[44px] sm:min-h-0"
        >
          Copy Event Log
        </Button>
        <Button
          variant="danger"
          size="md"
          onClick={onReset}
          aria-label="Reset simulator state"
          className="w-full sm:w-auto min-h-[44px] sm:min-h-0"
        >
          Reset Simulator
        </Button>
      </div>
    </div>
  );
};
