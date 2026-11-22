// A tile of the Files place pressed: a script opened big (`OpenFile` in
// renderer/src/files/FilesPlace.tsx, which draws `ArtifactView`). The fixture says which (`fileOpen`).
import { openFiles, openTile } from './files.mjs';

export const go = (h) => openFiles(h, { then: (h) => openTile(h, 'round-tax.ts') });
