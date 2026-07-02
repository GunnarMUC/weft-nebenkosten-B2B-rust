import type { NodeTemplate } from '$lib/types';
import { ShieldCheck } from '@lucide/svelte';

export const ComplianceCheckerNode: NodeTemplate = {
  type: 'ComplianceChecker',
  label: 'Compliance-Check',
  description: 'Deterministische Prufung gegen BetrKV, HeizkostenV, CO2KostAufG, BGB (Rust Rule Engine).',
  isBase: false,
  icon: ShieldCheck,
  color: '#f59e0b',
  category: 'Legal',
  tags: ['compliance', 'regulatorik', 'pruefung', 'recht'],
  fields: [
    { key: 'checks', label: 'Prufmodule', type: 'multiselect', options: ['fristen', 'verteilschluessel', 'co2', 'kabelanschluss', 'vorwegabzuege', 'plausibilitaet'], default: ['fristen', 'verteilschluessel', 'co2'] },
  ],
  defaultInputs: [
    { name: 'classifiedPositions', portType: 'List[JsonDict]', required: true, description: 'Klassifizierte Kostenpositionen' },
    { name: 'metadata', portType: 'JsonDict', required: false, description: 'Metadaten (Flache, Zeitraum, Vorjahreskosten)' },
  ],
  defaultOutputs: [
    { name: 'findings', portType: 'List[JsonDict]', required: false, description: 'Gefundene Verstoesse' },
    { name: 'severity', portType: 'Number', required: false, description: 'Gesamtschweregrad 1-5' },
    { name: 'routing', portType: 'String', required: false, description: 'auto|review|escalate' },
  ],
  features: {},
};
