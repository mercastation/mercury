// Home while a new version waits for a restart: the rail's foot gains its Restart row, first.
export function dress(snapshot) {
  return {
    ...snapshot,
    update: {
      canMove: false, place: null, moveError: null, phase: 'ready', currentVersion: '0.1.0', newVersion: '0.2.0',
      percent: null, error: null, checkedAt: null, readyAt: null, ready: true, source: true, changes: [], behind: 3, installing: false,
    },
  };
}
