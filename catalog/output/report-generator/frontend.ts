import type { NodeTemplate } from '$lib/types';
import { FileText } from '@lucide/svelte';

export const ReportGeneratorNode: NodeTemplate = {
  type: 'ReportGenerator',
  label: 'Bericht erstellen',
  description: 'Erstellt einen strukturierten Prufbericht (PDF + JSON) mit allen Findings und Handlungsempfehlungen.',
  isBase: false,
  icon: FileText,
  color: '#8b5cf6',
  category: 'Output',
  tags: ['report', 'pdf', 'bericht', 'pruefbericht'],
  fields: [
    { key: 'format', label: 'Format', type: 'select', options: ['pdf', 'json', 'both'], default: 'pdf' },
    { key: 'template', label: 'Template', type: 'select', options: ['gewerbe', 'standard', 'wohnraum'], default: 'gewerbe' },
  ],
  defaultInputs: [
    { name: 'analysisResult', portType: 'JsonDict', required: true, description: 'Analyse-Ergebnisse' },
    { name: 'findings', portType: 'List[JsonDict]', required: true, description: 'Compliance-Findings' },
  ],
  defaultOutputs: [
    { name: 'reportPdf', portType: 'Binary', required: false, description: 'PDF-Prufbericht' },
    { name: 'reportJson', portType: 'JsonDict', required: false, description: 'JSON-Ausgabe' },
  ],
  features: {},
};
