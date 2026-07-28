/**
 * Structured logger for the JMAP Desktop frontend.
 *
 * Provides namespaced, leveled logging with timestamps.
 * All logs go to the browser console; in Tauri they appear
 * alongside the Rust `tracing` output when `--devtools` is enabled.
 *
 * Usage:
 *   import { logger } from '$lib/logger';
 *   logger.info('actions', 'connect', { serverUrl, username });
 *   logger.warn('actions', 'connect', 'failed', { error });
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  namespace: string;
  action: string;
  details?: unknown;
  durationMs?: number;
}

/** Format a structured entry for console output. */
function format(entry: LogEntry): string {
  let msg = `[${entry.namespace}] ${entry.action}`;
  if (entry.details !== undefined) {
    // Keep details as second arg to console for expandability
    return msg;
  }
  return msg;
}

/** Emit a console log at the appropriate level. */
function emit(level: LogLevel, entry: LogEntry): void {
  const args: unknown[] = [entry.timestamp, format(entry)];
  if (entry.details !== undefined) args.push(entry.details);
  if (entry.durationMs !== undefined) args.push(`(${entry.durationMs}ms)`);

  switch (level) {
    case 'debug':
      console.debug(...args);
      break;
    case 'info':
      console.info(...args);
      break;
    case 'warn':
      console.warn(...args);
      break;
    case 'error':
      console.error(...args);
      break;
  }
}

export const logger = {
  /**
   * Log at DEBUG level.
   * @param namespace  e.g. 'jmap', 'store', 'ui'
   * @param action     e.g. 'connect', 'query_emails'
   * @param details    optional data to attach
   */
  debug(namespace: string, action: string, details?: unknown) {
    emit('debug', {
      timestamp: new Date().toISOString(),
      level: 'debug',
      namespace,
      action,
      details,
    });
  },

  /** Log at INFO level. */
  info(namespace: string, action: string, details?: unknown) {
    emit('info', {
      timestamp: new Date().toISOString(),
      level: 'info',
      namespace,
      action,
      details,
    });
  },

  /** Log at WARN level. */
  warn(namespace: string, action: string, details?: unknown) {
    emit('warn', {
      timestamp: new Date().toISOString(),
      level: 'warn',
      namespace,
      action,
      details,
    });
  },

  /** Log at ERROR level. */
  error(namespace: string, action: string, details?: unknown) {
    emit('error', {
      timestamp: new Date().toISOString(),
      level: 'error',
      namespace,
      action,
      details,
    });
  },

  /**
   * Time an async operation and log the duration.
   * @returns the result of the async function
   */
  async time<T>(
    level: LogLevel,
    namespace: string,
    action: string,
    fn: () => Promise<T>,
  ): Promise<T> {
    const start = performance.now();
    try {
      const result = await fn();
      const elapsed = Math.round(performance.now() - start);
      emit(level, {
        timestamp: new Date().toISOString(),
        level,
        namespace,
        action: `${action} ✓`,
        durationMs: elapsed,
      });
      return result;
    } catch (err) {
      const elapsed = Math.round(performance.now() - start);
      emit('error', {
        timestamp: new Date().toISOString(),
        level: 'error',
        namespace,
        action: `${action} ✗`,
        details: err,
        durationMs: elapsed,
      });
      throw err;
    }
  },
};
