// The change pressed open while the engine has not answered yet: the skeleton that holds a
// document's place (`PreviewLoading`). The fixture says so (`codeChanges` holds `null`).
import { openChange } from './thread-change-open.mjs';
import { changeFor } from './change.mjs';

export const go = (h) => openChange(h, { change: changeFor({ pending: true }) });
