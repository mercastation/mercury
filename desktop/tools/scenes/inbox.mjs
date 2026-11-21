// From Home to the Inbox page: press the rail's Inbox button, the second in its list.
export async function go(h) {
  await h.evaluate(`(() => {
    const tabs = document.querySelectorAll('nav.workspace-tabs[aria-label="Inbox"] button.workspace-tab');
    const inbox = [...tabs].find((b) => b.dataset.tab === 'inbox') ?? tabs[1];
    inbox.click();
  })()`);
  await h.wait(900);
}

// The Inbox page of a snapshot the scene changed: `change` is handed the one the page was
// fed and gives back the one it is answered from here on. The page is loaded again on it and
// taken to the Inbox, and what is handed back goes into the fixture file, so the native
// window is told the same.
export async function withSnapshot(h, change) {
  const snapshot = change(await h.evaluate('window.__SNAP__'));
  await h.call('Page.addScriptToEvaluateOnNewDocument', { source: `window.__SNAP__ = ${JSON.stringify(snapshot)};` });
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
  await go(h);
  return { snapshot };
}
