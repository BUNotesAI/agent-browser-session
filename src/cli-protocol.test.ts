import { afterEach, beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { execFile, execFileSync } from 'node:child_process';
import { once } from 'node:events';
import { mkdtempSync, mkdirSync, rmSync } from 'node:fs';
import { createServer, type Server } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseCommand } from './protocol.js';

const root = fileURLToPath(new URL('..', import.meta.url));
const binary = join(root, 'cli/target/debug/agent-browser-session');

// Only the socket transport is replaced. The native CLI calls its production
// parse_command, and requests pass through the daemon's actual Zod parser.
describe.skipIf(process.platform === 'win32')('native CLI to daemon protocol regressions', () => {
  let directory: string;
  let server: Server;
  let received: Record<string, unknown> | undefined;

  beforeAll(() => {
    execFileSync(
      'cargo',
      ['build', '--manifest-path', 'cli/Cargo.toml', '--target-dir', 'cli/target'],
      {
        cwd: root,
        timeout: 120000,
        stdio: 'pipe',
      }
    );
  }, 120000);

  beforeEach(async () => {
    directory = mkdtempSync(join(tmpdir(), 'ab-'));
    mkdirSync(join(directory, 'sys'));
    received = undefined;
    server = createServer((socket) => {
      let buffer = '';
      socket.on('data', (chunk) => {
        buffer += chunk.toString();
        if (!buffer.includes('\n')) return;
        received = JSON.parse(buffer.trim());
        const parsed = parseCommand(buffer.trim());
        socket.end(
          JSON.stringify(
            parsed.success ? { success: true, data: {} } : { success: false, error: parsed.error }
          ) + '\n'
        );
      });
    });
    server.listen(join(directory, 'sys/main.sock'));
    await once(server, 'listening');
  });

  afterEach(async () => {
    await new Promise<void>((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()));
    });
    rmSync(directory, { recursive: true, force: true });
  });

  function run(args: string[]) {
    return new Promise<{ code: number | string; stdout: string; stderr: string }>((resolve) => {
      execFile(
        binary,
        ['--json', ...args],
        {
          env: {
            ...process.env,
            AGENT_BROWSER_SOCKET_DIR: directory,
            AGENT_BROWSER_HEADED: 'false',
            AGENT_BROWSER_TABNAME: 'schema-regression',
          },
          timeout: 10000,
        },
        (error, stdout, stderr) =>
          resolve({ code: error?.code ?? 0, stdout: String(stdout), stderr: String(stderr) })
      );
    });
  }

  const accepted: {
    name: string;
    args: string[];
    expected: Record<string, unknown>;
    absent?: string[];
  }[] = [
    {
      name: 'screenshot without a path',
      args: ['screenshot'],
      expected: { action: 'screenshot', fullPage: false },
      absent: ['path'],
    },
    {
      name: 'full screenshot without a path',
      args: ['screenshot', '--full'],
      expected: { action: 'screenshot', fullPage: true },
      absent: ['path'],
    },
    {
      name: 'screenshot with a path',
      args: ['screenshot', '/fixture/page.png'],
      expected: { path: '/fixture/page.png' },
    },
    {
      name: 'requests without a filter',
      args: ['network', 'requests'],
      expected: { action: 'requests', clear: false },
      absent: ['filter'],
    },
    {
      name: 'requests clear without a filter',
      args: ['network', 'requests', '--clear'],
      expected: { action: 'requests', clear: true },
      absent: ['filter'],
    },
    {
      name: 'requests with a filter',
      args: ['network', 'requests', '--filter', 'api'],
      expected: { filter: 'api' },
    },
    {
      name: 'unroute without a URL',
      args: ['network', 'unroute'],
      expected: { action: 'unroute' },
      absent: ['url'],
    },
    {
      name: 'unroute with a URL',
      args: ['network', 'unroute', '**/api'],
      expected: { url: '**/api' },
    },
    {
      name: 'route body reaches response',
      args: ['network', 'route', '**/api', '--body', '{"ok":true}'],
      expected: { action: 'route', response: { body: '{"ok":true}' } },
      absent: ['body'],
    },
    {
      name: 'route abort has no response',
      args: ['network', 'route', '**/api', '--abort'],
      expected: { abort: true },
      absent: ['body', 'response'],
    },
    {
      name: 'trace start has no output path',
      args: ['trace', 'start'],
      expected: { action: 'trace_start' },
      absent: ['path'],
    },
    {
      name: 'trace stop preserves output path',
      args: ['trace', 'stop', '/fixture/trace.zip'],
      expected: { action: 'trace_stop', path: '/fixture/trace.zip' },
    },
    {
      name: 'open preserves scoped headers',
      args: ['open', 'https://example.test', '--headers', '{"X-Probe":"schema"}'],
      expected: { action: 'navigate', headers: { 'X-Probe': 'schema' } },
    },
    {
      name: 'open preserves a data fixture URL',
      args: ['open', 'data:text/html,<title>Fixture</title>'],
      expected: { url: 'data:text/html,<title>Fixture</title>' },
    },
    {
      name: 'tab new omits the unused URL',
      args: ['tab', 'new'],
      expected: { action: 'tab_new' },
      absent: ['url'],
    },
    {
      name: 'tab close without an index',
      args: ['tab', 'close'],
      expected: { action: 'tab_close' },
      absent: ['index'],
    },
    { name: 'tab close with an index', args: ['tab', 'close', '2'], expected: { index: 2 } },
    {
      name: 'frame main maps to mainframe',
      args: ['frame', 'main'],
      expected: { action: 'mainframe' },
    },
    {
      name: 'mouse wheel maps signed deltas',
      args: ['mouse', 'wheel', '120', '-40'],
      expected: { action: 'wheel', deltaX: -40, deltaY: 120 },
    },
    {
      name: 'media maps reduced motion enum',
      args: ['set', 'media', 'dark', 'reduced-motion'],
      expected: { action: 'emulatemedia', colorScheme: 'dark', reducedMotion: 'reduce' },
    },
    {
      name: 'media maps default motion enum',
      args: ['set', 'media', 'light'],
      expected: { action: 'emulatemedia', colorScheme: 'light', reducedMotion: 'no-preference' },
    },
    {
      name: 'dialog accept without text',
      args: ['dialog', 'accept'],
      expected: { action: 'dialog', response: 'accept' },
      absent: ['promptText'],
    },
    {
      name: 'dialog accept with text',
      args: ['dialog', 'accept', 'fixture text'],
      expected: { promptText: 'fixture text' },
    },
    {
      name: 'find role omits optional fields',
      args: ['find', 'role', 'button'],
      expected: { action: 'getbyrole', subaction: 'click' },
      absent: ['name', 'value', 'exact'],
    },
    {
      name: 'find role separates name from action',
      args: ['find', 'role', 'button', '--name', 'Submit'],
      expected: { name: 'Submit', subaction: 'click' },
      absent: ['value', 'exact'],
    },
    {
      name: 'find role separates flags from fill text',
      args: ['find', 'role', 'textbox', 'fill', 'fixture text', '--name', 'Email'],
      expected: { value: 'fixture text', name: 'Email' },
    },
    {
      name: 'find text exact defaults to click',
      args: ['find', 'text', 'Hello', '--exact'],
      expected: { action: 'getbytext', exact: true, subaction: 'click' },
    },
    {
      name: 'find label omits optional value',
      args: ['find', 'label', 'Email'],
      expected: { action: 'getbylabel' },
      absent: ['value', 'exact'],
    },
    {
      name: 'find placeholder omits optional value',
      args: ['find', 'placeholder', 'Email'],
      expected: { action: 'getbyplaceholder' },
      absent: ['value', 'exact'],
    },
    {
      name: 'find alt supports exact hover',
      args: ['find', 'alt', 'Image', 'hover', '--exact'],
      expected: { action: 'getbyalttext', subaction: 'hover', exact: true },
    },
    {
      name: 'find title supports exact click',
      args: ['find', 'title', 'Hint', '--exact'],
      expected: { action: 'getbytitle', subaction: 'click', exact: true },
    },
    {
      name: 'find testid omits optional value',
      args: ['find', 'testid', 'submit'],
      expected: { action: 'getbytestid' },
      absent: ['value'],
    },
    {
      name: 'find first omits optional value',
      args: ['find', 'first', 'button'],
      expected: { action: 'nth', index: 0 },
      absent: ['value'],
    },
    {
      name: 'find last omits optional value',
      args: ['find', 'last', 'button'],
      expected: { action: 'nth', index: -1 },
      absent: ['value'],
    },
    {
      name: 'find nth omits optional value',
      args: ['find', 'nth', '2', 'button'],
      expected: { action: 'nth', index: 2 },
      absent: ['value'],
    },
    {
      name: 'find nth preserves text action',
      args: ['find', 'nth', '0', 'p', 'text'],
      expected: { action: 'nth', subaction: 'text' },
      absent: ['value'],
    },
  ];

  it.each(accepted)('$name', async ({ args, expected, absent }) => {
    const result = await run(args);
    expect(result.code, result.stdout + result.stderr).toBe(0);
    expect(received).toBeDefined();
    const parsed = parseCommand(JSON.stringify(received));
    expect(parsed.success).toBe(true);
    if (!parsed.success) return;
    expect(parsed.command).toEqual(received);
    expect(parsed.command).toMatchObject(expected);
    for (const field of absent ?? []) expect(received).not.toHaveProperty(field);
  });

  const rejected = [
    { name: 'trace stop requires a path', args: ['trace', 'stop'], type: 'missing_arguments' },
    {
      name: 'trace start does not silently discard a path',
      args: ['trace', 'start', '/fixture/trace.zip'],
      type: 'missing_arguments',
    },
    {
      name: 'tab new does not silently discard a URL',
      args: ['tab', 'new', 'https://example.test'],
      type: 'missing_arguments',
    },
    {
      name: 'find role rejects unsupported exact semantics',
      args: ['find', 'role', 'button', '--exact'],
      type: 'unknown_subcommand',
    },
    {
      name: 'find label rejects unsupported exact semantics',
      args: ['find', 'label', 'Email', '--exact'],
      type: 'unknown_subcommand',
    },
    {
      name: 'find placeholder rejects unsupported exact semantics',
      args: ['find', 'placeholder', 'Email', '--exact'],
      type: 'unknown_subcommand',
    },
    {
      name: 'find role rejects unsupported type action',
      args: ['find', 'role', 'textbox', 'type', 'value'],
      type: 'unknown_subcommand',
    },
    {
      name: 'find text rejects unsupported fill action',
      args: ['find', 'text', 'Hello', 'fill', 'value'],
      type: 'unknown_subcommand',
    },
  ];

  it.each(rejected)('$name', async ({ args, type }) => {
    const result = await run(args);
    expect(result.code).toBe(1);
    expect(JSON.parse(result.stdout).type).toBe(type);
    expect(received).toBeUndefined();
  });
});
