/**
 * Mock Monaco Editor for testing
 */

export const editor = {
  setModelMarkers: () => {},
};

export const MarkerSeverity = {
  Hint: 1,
  Info: 2,
  Warning: 4,
  Error: 8,
};

export default {
  editor,
  MarkerSeverity,
};
