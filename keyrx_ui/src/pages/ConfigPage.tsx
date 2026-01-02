import React, { useState, useCallback, useEffect } from 'react';
import { Card } from '@/components/Card';
import { Dropdown } from '@/components/Dropdown';
import { KeyboardVisualizer } from '@/components/KeyboardVisualizer';
import { KeyMapping } from '@/components/KeyButton';
import { LoadingSkeleton } from '@/components/LoadingSkeleton';
import { MonacoEditor } from '@/components/MonacoEditor';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';
import { RpcClient } from '@/api/rpc';
import { ValidationError } from '@/hooks/useWasm';

interface ConfigPageProps {
  profileName?: string;
}

export const ConfigPage: React.FC<ConfigPageProps> = ({
  profileName = 'Default',
}) => {
  const api = useUnifiedApi();
  const rpcClient = new RpcClient(api);

  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'visual' | 'code'>('visual');
  const [configCode, setConfigCode] = useState<string>('// Loading configuration...\n');
  const [validationErrors, setValidationErrors] = useState<ValidationError[]>([]);
  const [selectedLayout, setSelectedLayout] =
    useState<'ANSI_104' | 'ISO_105' | 'JIS_109' | 'HHKB' | 'NUMPAD'>('ANSI_104');
  const [selectedLayer, setSelectedLayer] = useState('base');
  const [previewMode, setPreviewMode] = useState(false);
  const [keyMappings] = useState<Map<string, KeyMapping>>(new Map());
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState<string>('');
  const [connectionTimeout, setConnectionTimeout] = useState(false);

  // Layout options
  const layoutOptions = [
    { value: 'ANSI_104', label: 'ANSI 104' },
    { value: 'ISO_105', label: 'ISO 105' },
    { value: 'JIS_109', label: 'JIS 109' },
    { value: 'HHKB', label: 'HHKB' },
    { value: 'NUMPAD', label: 'Numpad' },
  ];

  // Layer options (mock data - would come from API)
  const layerOptions = [
    { value: 'base', label: 'Base' },
    { value: 'nav', label: 'Nav' },
    { value: 'num', label: 'Num' },
    { value: 'fn', label: 'Fn' },
    { value: 'gaming', label: 'Gaming' },
  ];

  const handleKeyClick = useCallback((keyCode: string) => {
    console.log('Key clicked:', keyCode);
    // TODO: Open KeyConfigDialog modal (Task 17)
  }, []);

  const handleLayoutChange = useCallback((value: string) => {
    setSelectedLayout(value);
  }, []);

  const handleLayerChange = useCallback((value: string) => {
    setSelectedLayer(value);
  }, []);

  const togglePreviewMode = useCallback(() => {
    setPreviewMode((prev) => !prev);
  }, []);

  // Connection timeout - show error after 10 seconds
  useEffect(() => {
    if (!api.isConnected && !connectionTimeout) {
      const timer = setTimeout(() => {
        setConnectionTimeout(true);
      }, 10000);
      return () => clearTimeout(timer);
    }
  }, [api.isConnected, connectionTimeout]);

  // Load configuration on mount
  useEffect(() => {
    const loadConfig = async () => {
      try {
        setLoading(true);
        const config = await rpcClient.getConfig();
        setConfigCode(config.code);
      } catch (error) {
        console.error('Failed to load configuration:', error);
        setConfigCode('// Failed to load configuration\n// Error: ' + (error instanceof Error ? error.message : String(error)));
      } finally {
        setLoading(false);
      }
    };

    if (api.isConnected) {
      setConnectionTimeout(false); // Reset timeout on successful connection
      loadConfig();
    }
  }, [api.isConnected]);

  // Handle validation callback from Monaco
  const handleValidation = useCallback((errors: ValidationError[]) => {
    setValidationErrors(errors);
  }, []);

  // Handle save
  const handleSave = useCallback(async () => {
    // Prevent save if validation errors exist
    if (validationErrors.length > 0) {
      setErrorMessage('Cannot save: configuration has validation errors');
      setSaveStatus('error');
      setTimeout(() => {
        setSaveStatus('idle');
        setErrorMessage('');
      }, 3000);
      return;
    }

    try {
      setSaveStatus('saving');
      setErrorMessage('');
      await rpcClient.updateConfig(configCode);
      setSaveStatus('success');
      setTimeout(() => setSaveStatus('idle'), 2000);
    } catch (error) {
      console.error('Failed to save configuration:', error);
      setErrorMessage(error instanceof Error ? error.message : String(error));
      setSaveStatus('error');
      setTimeout(() => {
        setSaveStatus('idle');
        setErrorMessage('');
      }, 3000);
    }
  }, [configCode, validationErrors, rpcClient]);

  // Ctrl+S / Cmd+S keyboard shortcut
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        handleSave();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleSave]);

  // Mock modified keys count (would come from configuration store)
  const modifiedKeysCount = 37;

  // Show timeout error if connection takes too long
  if (connectionTimeout && !api.isConnected) {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen gap-4 p-4">
        <div className="text-red-400 text-xl">⚠️ Connection Timeout</div>
        <div className="text-slate-300 text-center max-w-md">
          Failed to connect to the daemon WebSocket. Please ensure the daemon is running and try refreshing the page.
        </div>
        <button
          onClick={() => window.location.reload()}
          className="px-6 py-3 bg-primary-500 text-white rounded-md hover:bg-primary-600 transition-colors"
        >
          Reload Page
        </button>
      </div>
    );
  }

  // Show loading state while connecting or loading config
  if (!api.isConnected || loading) {
    return (
      <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
        <div className="text-center text-slate-400 py-4">
          {!api.isConnected ? '⏳ Connecting to daemon...' : '⏳ Loading configuration...'}
        </div>

        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
          <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4">
            <LoadingSkeleton variant="text" width="250px" height="32px" />
            <LoadingSkeleton variant="text" width="150px" height="20px" />
          </div>
          <LoadingSkeleton variant="rectangular" width="180px" height="44px" />
        </div>

        <Card variant="default" padding="lg">
          <div className="flex flex-col gap-4">
            <div className="flex items-center justify-between pb-4 border-b border-slate-700">
              <LoadingSkeleton variant="text" width="150px" height="24px" />
              <LoadingSkeleton variant="rectangular" width="192px" height="40px" />
            </div>
            <LoadingSkeleton variant="rectangular" height="400px" />
          </div>
        </Card>

        <Card variant="default" padding="lg">
          <div className="flex flex-col gap-4">
            <LoadingSkeleton variant="text" width="100px" height="24px" />
            <div className="flex gap-2">
              <LoadingSkeleton variant="rectangular" width="80px" height="32px" />
              <LoadingSkeleton variant="rectangular" width="80px" height="32px" />
              <LoadingSkeleton variant="rectangular" width="80px" height="32px" />
            </div>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
      {/* Header */}
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
        <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4">
          <h1 className="text-xl md:text-2xl lg:text-3xl font-semibold text-slate-100">
            Configuration Editor
          </h1>
          <span className="hidden sm:inline text-slate-400">—</span>
          <span className="text-sm sm:text-base text-slate-300">
            Profile: {profileName}
          </span>
        </div>
        <button
          onClick={handleSave}
          disabled={saveStatus === 'saving' || validationErrors.length > 0}
          className={`px-6 py-3 md:py-2 rounded-md font-medium transition-colors min-h-[44px] md:min-h-0 w-full md:w-auto ${
            saveStatus === 'saving'
              ? 'bg-slate-600 text-slate-400 cursor-wait'
              : validationErrors.length > 0
              ? 'bg-slate-700 text-slate-500 cursor-not-allowed'
              : saveStatus === 'success'
              ? 'bg-green-600 text-white'
              : saveStatus === 'error'
              ? 'bg-red-600 text-white'
              : 'bg-primary-500 text-white hover:bg-primary-600'
          }`}
          aria-label="Save configuration"
        >
          {saveStatus === 'saving'
            ? '💾 Saving...'
            : saveStatus === 'success'
            ? '✓ Saved!'
            : saveStatus === 'error'
            ? '✗ Error'
            : '💾 Save (Ctrl+S)'}
        </button>
      </div>

      {/* Tab Buttons */}
      <div className="grid grid-cols-2 sm:flex sm:gap-2" role="tablist" aria-label="Editor mode">
        <button
          id="visual-tab"
          role="tab"
          aria-selected={activeTab === 'visual'}
          aria-controls="visual-panel"
          onClick={() => setActiveTab('visual')}
          className={`px-6 py-3 sm:py-2 rounded-md font-medium transition-colors min-h-[44px] sm:min-h-0 ${
            activeTab === 'visual'
              ? 'bg-primary-500 text-white'
              : 'bg-slate-700 text-slate-400 hover:text-slate-300 hover:bg-slate-600'
          }`}
        >
          🎨 Visual Editor
        </button>
        <button
          id="code-tab"
          role="tab"
          aria-selected={activeTab === 'code'}
          aria-controls="code-panel"
          onClick={() => setActiveTab('code')}
          className={`px-6 py-3 sm:py-2 rounded-md font-medium transition-colors min-h-[44px] sm:min-h-0 ${
            activeTab === 'code'
              ? 'bg-primary-500 text-white'
              : 'bg-slate-700 text-slate-400 hover:text-slate-300 hover:bg-slate-600'
          }`}
        >
          {'</>'}Code Editor
        </button>
      </div>

      {/* Validation Status Panel */}
      {(validationErrors.length > 0 || saveStatus === 'error') && (
        <Card variant="danger" padding="md">
          <div className="flex flex-col gap-2">
            {validationErrors.length > 0 && (
              <div className="text-sm md:text-base text-red-300">
                ⚠️ {validationErrors.length} validation {validationErrors.length === 1 ? 'error' : 'errors'}
              </div>
            )}
            {saveStatus === 'error' && errorMessage && (
              <div className="text-sm md:text-base text-red-300">
                ✗ {errorMessage}
              </div>
            )}
          </div>
        </Card>
      )}

      {saveStatus === 'success' && (
        <Card variant="success" padding="md">
          <div className="text-sm md:text-base text-green-300">
            ✓ Configuration saved successfully
          </div>
        </Card>
      )}

      {/* Visual Editor Panel */}
      {activeTab === 'visual' && (
        <div role="tabpanel" id="visual-panel" aria-labelledby="visual-tab">
          {/* Keyboard Visualizer Card */}
          <Card variant="default" padding="lg">
            <div className="flex flex-col gap-4">
              {/* Card Header with Layout Selector */}
              <div className="flex items-center justify-between pb-4 border-b border-slate-700">
                <h2 className="text-lg font-medium text-slate-100">
                  Keyboard Layout
                </h2>
                <div className="w-48">
                  <Dropdown
                    options={layoutOptions}
                    value={selectedLayout}
                    onChange={handleLayoutChange}
                    aria-label="Select keyboard layout"
                    searchable={false}
                  />
                </div>
              </div>

              {/* Keyboard Visualizer - horizontal scroll on mobile */}
              <div className="py-4 overflow-x-auto md:overflow-x-visible">
                <KeyboardVisualizer
                  layout={selectedLayout}
                  keyMappings={keyMappings}
                  onKeyClick={handleKeyClick}
                />
              </div>

              {/* Example Mapping Display */}
              <div className="text-sm text-slate-400 italic pt-4 border-t border-slate-700">
                Example: <span className="font-mono text-slate-300">*Caps*</span> ={' '}
                Tap: Escape, Hold (200ms): Ctrl
              </div>
            </div>
          </Card>

          {/* Layer Selector Card */}
          <Card variant="default" padding="lg" className="mt-4 md:mt-6">
            <div className="flex flex-col gap-4">
              {/* Card Header */}
              <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 sm:gap-4">
                <h2 className="text-base sm:text-lg font-medium text-slate-100">
                  Active Layer: MD_00 ({selectedLayer})
                </h2>
                <button
                  className="text-sm text-primary-500 hover:text-primary-400 transition-colors self-start sm:self-auto min-h-[44px] sm:min-h-0 flex items-center"
                  aria-label="Open layer list"
                >
                  Layer List ▼
                </button>
              </div>

              {/* Layer Buttons - responsive grid on mobile */}
              <div className="grid grid-cols-2 sm:flex sm:flex-wrap gap-2" role="group" aria-label="Layer selection">
                {layerOptions.map((layer) => (
                  <button
                    key={layer.value}
                    onClick={() => handleLayerChange(layer.value)}
                    className={`px-4 py-3 sm:py-2 rounded-md font-medium transition-all min-h-[44px] sm:min-h-0 focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2 ${
                      selectedLayer === layer.value
                        ? 'bg-primary-500 text-white'
                        : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                    }`}
                    aria-label={`Switch to ${layer.label} layer`}
                    aria-pressed={selectedLayer === layer.value}
                  >
                    {layer.label}
                  </button>
                ))}
              </div>

              {/* Modified Keys Count */}
              <div className="text-sm text-slate-400 pt-2 border-t border-slate-700">
                Modified keys in this layer:{' '}
                <span className="font-semibold text-slate-300">
                  {modifiedKeysCount}
                </span>
              </div>
            </div>
          </Card>
        </div>
      )}

      {/* Code Editor Panel */}
      {activeTab === 'code' && (
        <div role="tabpanel" id="code-panel" aria-labelledby="code-tab">
          <Card variant="default" padding="lg">
            <div className="flex flex-col gap-4">
              <div className="flex items-center justify-between pb-4 border-b border-slate-700">
                <h2 className="text-lg font-medium text-slate-100">
                  Rhai Configuration Code
                </h2>
                {validationErrors.length === 0 && (
                  <span className="text-sm text-green-400">
                    ✓ No errors
                  </span>
                )}
              </div>
              <MonacoEditor
                value={configCode}
                onChange={setConfigCode}
                onValidate={handleValidation}
                height="600px"
              />
            </div>
          </Card>
        </div>
      )}
    </div>
  );
};

export default ConfigPage;
