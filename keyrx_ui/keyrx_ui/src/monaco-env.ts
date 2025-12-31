/**
 * Monaco Editor environment configuration for Vite
 *
 * This file configures Monaco Editor's web workers for proper loading in Vite.
 * Must be imported before any Monaco Editor usage.
 */

import * as monaco from 'monaco-editor';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
import { initializeRhaiSupport } from './utils/monacoConfig';

// Configure Monaco environment for web workers
self.MonacoEnvironment = {
  getWorker(_: string, label: string) {
    if (label === 'json') {
      return new jsonWorker();
    }
    return new editorWorker();
  }
};

// Initialize Rhai language support
initializeRhaiSupport();

export { monaco };
