// A tile of the Files place pressed: a document with sections opened big (`OpenFile` in
// renderer/src/files/FilesPlace.tsx, which draws `ArtifactView`). The fixture says which (`fileOpen`).
import { openFiles, openTile } from './files.mjs';

export const go = (h) => openFiles(h, { then: (h) => openTile(h, 'receipt-plan.md') });
