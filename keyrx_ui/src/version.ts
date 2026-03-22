// Version constants injected by Vite at build time from Cargo.toml (SSOT).
// See vite.config.ts getBuildDefines() for injection logic.
// Do NOT hardcode values here — they are replaced at compile time.

export const VERSION: string = __APP_VERSION__;
export const BUILD_TIME: string = __BUILD_TIME__;
export const GIT_COMMIT: string = __GIT_COMMIT__;
export const GIT_BRANCH: string = __GIT_BRANCH__;

export const BUILD_INFO = {
  version: VERSION,
  buildTime: BUILD_TIME,
  gitCommit: GIT_COMMIT,
  gitBranch: GIT_BRANCH,
  gitDirty: __GIT_DIRTY__,
} as const;
