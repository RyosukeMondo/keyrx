/// <reference types="vitest" />
import type { Reporter, Task, Vitest } from 'vitest';

/**
 * Custom Vitest reporter that logs slow tests exceeding the threshold.
 * Helps identify tests that need performance optimization.
 */
export class SlowTestReporter implements Reporter {
  private ctx!: Vitest;
  private slowThreshold: number;
  private slowTests: Array<{ name: string; duration: number; file: string }> = [];

  constructor(slowThreshold = 1000) {
    this.slowThreshold = slowThreshold;
  }

  onInit(ctx: Vitest) {
    this.ctx = ctx;
  }

  onTaskUpdate(tasks: Task[]) {
    for (const task of tasks) {
      this.checkTask(task);
    }
  }

  private checkTask(task: Task) {
    // Check if this is a test task and it has completed successfully
    if (task.type === 'test' && task.result?.state === 'pass') {
      const duration = task.result.duration || 0;
      if (duration > this.slowThreshold) {
        // Extract file path from task
        const file = task.file?.filepath || 'unknown';
        const testName = task.name;

        // Store slow test info (avoid duplicates)
        const exists = this.slowTests.some(t => t.name === testName && t.file === file);
        if (!exists) {
          this.slowTests.push({
            name: testName,
            duration: Math.round(duration),
            file: file.replace(process.cwd(), '.'),
          });
        }
      }
    }

    // Recursively check child tasks
    if (task.tasks) {
      for (const child of task.tasks) {
        this.checkTask(child);
      }
    }
  }

  async onFinished() {
    if (this.slowTests.length > 0) {
      console.log('\n⚠️  Slow Tests Detected (>' + this.slowThreshold + 'ms):\n');

      // Sort by duration (slowest first)
      this.slowTests.sort((a, b) => b.duration - a.duration);

      // Print slow tests
      for (const test of this.slowTests) {
        console.log(
          `  ${test.duration}ms - ${test.name}`
        );
        console.log(`           ${test.file}`);
      }

      console.log(`\n  Total slow tests: ${this.slowTests.length}\n`);
    }
  }
}
