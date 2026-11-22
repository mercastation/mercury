// Home with two threads that need the person and none in progress: three rows or fewer, so the
// two cards are only as tall as their rows and have no fade at their foot (`.hm-two.fits` in
// renderer/src/home/home.css), side by side one height, stacked each its own.
export const dress = (snapshot) => ({ ...snapshot, items: snapshot.items.slice(0, 2) });
