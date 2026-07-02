import type { NodeTemplate } from '$lib/types';
import { FolderOpen } from '@lucide/svelte';

export const JsonlIngestionNode: NodeTemplate = {
  type: 'JsonlIngestion',
  label: 'JSONL-Eingang',
  description: 'Uberwacht Verzeichnis auf neue JSONL-Dateien (vom Chunker) und startet Analyse-Pipeline.',
  isBase: false,
  icon: FolderOpen,
  color: '#06b6d4',
  category: 'Triggers',
  tags: ['jsonl', 'ingestion', 'chunker', 'trigger'],
  fields: [
    { key: 'directory', label: 'Verzeichnis', type: 'text', default: '/data/chunks' },
    { key: 'pattern', label: 'Dateimuster', type: 'text', default: '*.jsonl' },
    { key: 'pollInterval', label: 'Poll-Intervall (s)', type: 'number', default: 10 },
  ],
  defaultInputs: [],
  defaultOutputs: [
    { name: 'chunks', portType: 'List[JsonDict]', required: false, description: 'Gelesene Chunks aus JSONL' },
    { name: 'metadata', portType: 'JsonDict', required: false, description: 'Dokument-Metadaten' },
  ],
  features: {},
};
