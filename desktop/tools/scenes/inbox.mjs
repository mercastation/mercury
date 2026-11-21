// From Home to the Inbox page: press the rail's Inbox button, the second in its list.
export async function go(h) {
  await h.evaluate(`(() => {
    const tabs = document.querySelectorAll('nav.workspace-tabs[aria-label="Inbox"] button.workspace-tab');
    const inbox = [...tabs].find((b) => b.dataset.tab === 'inbox') ?? tabs[1];
    inbox.click();
  })()`);
  await h.wait(900);
}
