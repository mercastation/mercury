// A key the run asks for with a reason that holds a link and a command: the title is the key's
// own words, the line under it the whole reason with its link and its code (`withLinks`), and no
// service's page to get it from.
import { needs } from './thread-needs.mjs';

export const go = (h) => needs(h, {
  said: 'AGENTBOX-NEEDS: key TREG_TOKEN | the treg agent token from `treg org agent-new x-replies`, made at treg.dev/settings/agents',
});
