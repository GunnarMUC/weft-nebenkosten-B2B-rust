// Placeholder: BetrkvClassifier Node
// Klassifiziert Kostenpositionen in die 17 BetrKV-Kategorien.
// Implementation folgt in P4.

import type { NodeTemplate } from '$lib/types';
import { Scale } from '@lucide/svelte';

export const BetrkvClassifierNode: NodeTemplate = {
  type: 'BetrkvClassifier',
  label: 'BetrKV-Klassifizierung',
  description: 'Ordnet Kostenpositionen den 17 BetrKV-Kategorien (§ 2 BetrKV) zu.',
  isBase: false,
  icon: Scale,
  color: '#6366f1',
  category: 'Legal',
  tags: ['betrkv', 'klassifizierung', 'nebensatz', 'recht'],
  fields: [],
  defaultInputs: [
    { name: 'positions', portType: 'List[JsonDict]', required: true, description: 'Extrahierte Kostenpositionen' },
  ],
  defaultOutputs: [
    { name: 'classified', portType: 'List[JsonDict]', required: false, description: 'Klassifizierte Positionen mit BetrKV-Kategorie' },
  ],
  features: {},
};
