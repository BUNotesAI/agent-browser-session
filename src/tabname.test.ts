import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { BrowserManager } from './browser.js';
import { executeCommand } from './actions.js';
import type { Command } from './types.js';

describe('tabname isolation e2e', () => {
  let browser: BrowserManager;

  beforeAll(async () => {
    browser = new BrowserManager();
    await browser.launch({ headless: true, navigationDelay: 0 } as any);
  });

  afterAll(async () => {
    await browser.close();
  });

  it('should navigate two tabs independently', async () => {
    const reddit = await browser.getOrCreateTab('e2e-reddit');
    const hn = await browser.getOrCreateTab('e2e-hn');

    await reddit.page.goto(
      'data:text/html,<title>Reddit</title><h1>Reddit</h1><button>Upvote</button>'
    );
    await hn.page.goto('data:text/html,<title>HN</title><h1>HN</h1><a href="#">Comments</a>');

    expect(reddit.page.url()).toContain('Reddit');
    expect(hn.page.url()).toContain('HN');
    // Verify they didn't affect each other
    expect(await reddit.page.title()).not.toBe(await hn.page.title());

    // Clean up
    await browser.closeNamedTab('e2e-reddit');
    await browser.closeNamedTab('e2e-hn');
  });

  it('should have isolated refMaps per tab', async () => {
    const tabA = await browser.getOrCreateTab('e2e-ref-a');
    const tabB = await browser.getOrCreateTab('e2e-ref-b');

    await tabA.page.goto('data:text/html,<button>ButtonA</button>');
    await tabB.page.goto('data:text/html,<button>ButtonB</button>');

    // Take snapshots and store in bindings
    const snapA = await browser.getSnapshotForPage(tabA.page, {});
    const snapB = await browser.getSnapshotForPage(tabB.page, {});
    tabA.refMap = snapA.refs;
    tabB.refMap = snapB.refs;

    // RefMaps should be independent
    expect(Object.keys(tabA.refMap).length).toBeGreaterThan(0);
    expect(Object.keys(tabB.refMap).length).toBeGreaterThan(0);

    // Verify refs point to correct content
    const firstRefA = Object.values(tabA.refMap)[0];
    const firstRefB = Object.values(tabB.refMap)[0];
    expect(firstRefA.name).toContain('ButtonA');
    expect(firstRefB.name).toContain('ButtonB');

    // Clean up
    await browser.closeNamedTab('e2e-ref-a');
    await browser.closeNamedTab('e2e-ref-b');
  });

  it('should route navigate command via tabName', async () => {
    const result = await executeCommand(
      {
        id: '1',
        action: 'navigate',
        url: 'data:text/html,<title>Routed</title><h1>Routed</h1>',
        tabName: 'e2e-routed',
      } as Command,
      browser
    );
    expect(result.success).toBe(true);

    // Verify the tab was created and navigated
    const tabs = await browser.listNamedTabs();
    const routedTab = tabs.find((t) => t.name === 'e2e-routed');
    expect(routedTab).toBeDefined();
    expect(routedTab!.url).toContain('Routed');

    // Clean up
    await browser.closeNamedTab('e2e-routed');
  });

  it('should route snapshot command via tabName and store refMap', async () => {
    // First navigate
    await executeCommand(
      {
        id: '1',
        action: 'navigate',
        url: 'data:text/html,<button>SnapBtn</button>',
        tabName: 'e2e-snap-route',
      } as Command,
      browser
    );

    // Then snapshot
    const result = await executeCommand(
      { id: '2', action: 'snapshot', tabName: 'e2e-snap-route' } as Command,
      browser
    );
    expect(result.success).toBe(true);
    expect((result as any).data?.snapshot).toContain('SnapBtn');

    // Verify refMap was stored in binding
    const binding = await browser.getOrCreateTab('e2e-snap-route');
    expect(Object.keys(binding.refMap).length).toBeGreaterThan(0);

    // Clean up
    await browser.closeNamedTab('e2e-snap-route');
  });

  it('should block close when tabName is present', async () => {
    const result = await executeCommand(
      { id: '1', action: 'close', tabName: 'e2e-close-test' } as Command,
      browser
    );
    expect(result.success).toBe(false);
    expect((result as any).error).toContain('shared browser instance');
  });

  it('should not affect default (no tabname) behavior', async () => {
    // Navigate without tabName — should use activePageIndex
    const result = await executeCommand(
      {
        id: '1',
        action: 'navigate',
        url: 'data:text/html,<h1>Default</h1>',
      } as Command,
      browser
    );
    expect(result.success).toBe(true);
  });

  it('should isolate activeFrame per tab', async () => {
    // Create two tabs each with an iframe
    const htmlWithIframe = (id: string) =>
      `data:text/html,<h1>${id}</h1><iframe name="inner-${id}" srcdoc="<p>Frame content ${id}</p>"></iframe>`;

    await executeCommand(
      {
        id: '1',
        action: 'navigate',
        url: htmlWithIframe('TabX'),
        tabName: 'e2e-frame-x',
      } as Command,
      browser
    );

    await executeCommand(
      {
        id: '2',
        action: 'navigate',
        url: htmlWithIframe('TabY'),
        tabName: 'e2e-frame-y',
      } as Command,
      browser
    );

    // Switch tab X into its iframe
    const frameResult = await executeCommand(
      { id: '3', action: 'frame', name: 'inner-TabX', tabName: 'e2e-frame-x' } as Command,
      browser
    );
    expect(frameResult.success).toBe(true);

    // Tab X should have activeFrame set, Tab Y should not
    const bindingX = await browser.getOrCreateTab('e2e-frame-x');
    const bindingY = await browser.getOrCreateTab('e2e-frame-y');
    expect(bindingX.activeFrame).not.toBeNull();
    expect(bindingY.activeFrame).toBeNull();

    // Reset Tab X back to main frame
    const mainResult = await executeCommand(
      { id: '4', action: 'mainframe', tabName: 'e2e-frame-x' } as Command,
      browser
    );
    expect(mainResult.success).toBe(true);
    expect(bindingX.activeFrame).toBeNull();

    // Clean up
    await browser.closeNamedTab('e2e-frame-x');
    await browser.closeNamedTab('e2e-frame-y');
  });

  it('should allow evaluating JS on a specific tab', async () => {
    // Navigate two tabs to different content
    await executeCommand(
      {
        id: '1',
        action: 'navigate',
        url: 'data:text/html,<h1>EvalAlpha</h1>',
        tabName: 'e2e-eval-a',
      } as Command,
      browser
    );
    await executeCommand(
      {
        id: '2',
        action: 'navigate',
        url: 'data:text/html,<h1>EvalBeta</h1>',
        tabName: 'e2e-eval-b',
      } as Command,
      browser
    );

    // Evaluate on tab A — should see "EvalAlpha"
    const resultA = await executeCommand(
      {
        id: '3',
        action: 'evaluate',
        script: 'document.querySelector("h1").textContent',
        tabName: 'e2e-eval-a',
      } as Command,
      browser
    );
    expect(resultA.success).toBe(true);
    expect((resultA as any).data?.result).toBe('EvalAlpha');

    // Evaluate on tab B — should see "EvalBeta"
    const resultB = await executeCommand(
      {
        id: '4',
        action: 'evaluate',
        script: 'document.querySelector("h1").textContent',
        tabName: 'e2e-eval-b',
      } as Command,
      browser
    );
    expect(resultB.success).toBe(true);
    expect((resultB as any).data?.result).toBe('EvalBeta');

    // Clean up
    await browser.closeNamedTab('e2e-eval-a');
    await browser.closeNamedTab('e2e-eval-b');
  });

  it('should get URL and title for a specific tab', async () => {
    await executeCommand(
      {
        id: '1',
        action: 'navigate',
        url: 'data:text/html,<title>MyTitle</title><h1>Hello</h1>',
        tabName: 'e2e-url-title',
      } as Command,
      browser
    );

    const urlResult = await executeCommand(
      { id: '2', action: 'url', tabName: 'e2e-url-title' } as Command,
      browser
    );
    expect(urlResult.success).toBe(true);
    expect((urlResult as any).data?.url).toContain('MyTitle');

    const titleResult = await executeCommand(
      { id: '3', action: 'title', tabName: 'e2e-url-title' } as Command,
      browser
    );
    expect(titleResult.success).toBe(true);
    expect((titleResult as any).data?.title).toBe('MyTitle');

    // Clean up
    await browser.closeNamedTab('e2e-url-title');
  });

  it('should list named tabs via listNamedTabs after executeCommand creates them', async () => {
    // Create multiple tabs via executeCommand
    await executeCommand(
      {
        id: '1',
        action: 'navigate',
        url: 'data:text/html,<h1>ListTab1</h1>',
        tabName: 'e2e-list-1',
      } as Command,
      browser
    );
    await executeCommand(
      {
        id: '2',
        action: 'navigate',
        url: 'data:text/html,<h1>ListTab2</h1>',
        tabName: 'e2e-list-2',
      } as Command,
      browser
    );

    const tabs = await browser.listNamedTabs();
    const names = tabs.map((t) => t.name);
    expect(names).toContain('e2e-list-1');
    expect(names).toContain('e2e-list-2');

    // Clean up
    await browser.closeNamedTab('e2e-list-1');
    await browser.closeNamedTab('e2e-list-2');
  });

  it('should reuse existing tab when same tabName is used again', async () => {
    // Navigate with tabName
    await executeCommand(
      {
        id: '1',
        action: 'navigate',
        url: 'data:text/html,<h1>First</h1>',
        tabName: 'e2e-reuse',
      } as Command,
      browser
    );

    const binding1 = await browser.getOrCreateTab('e2e-reuse');
    const page1 = binding1.page;

    // Navigate again with same tabName — should reuse same page
    await executeCommand(
      {
        id: '2',
        action: 'navigate',
        url: 'data:text/html,<h1>Second</h1>',
        tabName: 'e2e-reuse',
      } as Command,
      browser
    );

    const binding2 = await browser.getOrCreateTab('e2e-reuse');
    expect(binding2.page).toBe(page1);
    // URL should be updated to the second navigation
    expect(binding2.page.url()).toContain('Second');

    // Clean up
    await browser.closeNamedTab('e2e-reuse');
  });
});
