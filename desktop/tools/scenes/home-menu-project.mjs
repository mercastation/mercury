// Home with the box's project menu open: the projects in their order, the one the box is on
// lit, and the row that makes a new one.
//
// `chip` is which of the box's chips to press, by the place the page gives it in the order it
// hides them when the row is too narrow (`dropRank` in `renderer/src/threads/fit-row.ts`): the
// model is 4 and the priority 1. The project is never hidden and is found by its own mark.
export async function openMenu(h, chip) {
  const at = chip === 'project' ? `.hm-start [data-tour="project"] [data-trigger]` : `.hm-start .tc-anchor[data-drop="${chip}"] [data-trigger]`;
  await h.evaluate(`document.querySelector(${JSON.stringify(at)}).click()`);
  await h.wait(500);
}

export async function go(h) {
  await openMenu(h, 'project');
  return { menu: 'project' };
}
