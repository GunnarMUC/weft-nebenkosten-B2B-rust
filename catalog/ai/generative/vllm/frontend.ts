import type { NodeTemplate } from '$lib/types';
import { Cpu } from '@lucide/svelte';

export const VllmInferenceNode: NodeTemplate = {
  type: 'VllmInference',
  label: 'vLLM',
  description: 'Lokale LLM-Inferenz uber vLLM OpenAI-compatible API (on-premises).',
  isBase: false,
  icon: Cpu,
  color: '#10b981',
  category: 'AI',
  tags: ['llm', 'local', 'vllm', 'on-premises', 'qwen', 'mixtral'],
  fields: [
    { key: 'model',        label: 'Modell',           type: 'text',     default: 'qwen2.5:32b' },
    { key: 'baseUrl',      label: 'vLLM URL',         type: 'text',     default: 'http://localhost:8000' },
    { key: 'systemPrompt', label: 'System-Prompt',     type: 'textarea' },
    { key: 'temperature',  label: 'Temperature',       type: 'number',   default: 0.7 },
    { key: 'maxTokens',    label: 'Max Tokens',        type: 'number',   default: 4096 },
    { key: 'topP',         label: 'Top P',            type: 'number',   default: 0.9 },
    { key: 'parseJson',    label: 'JSON-Ausgabe',      type: 'checkbox', default: false },
  ],
  defaultInputs: [
    { name: 'prompt',       portType: 'String', required: true,  description: 'Anfrage an das LLM' },
    { name: 'systemPrompt', portType: 'String', required: false, description: 'System-Kontext fur das LLM' },
    { name: 'config',       portType: 'Dict[String, String | Number | Boolean]', required: false, description: 'Uberschreibt node-eigene Config' },
  ],
  defaultOutputs: [
    { name: 'response', portType: 'String', required: false, description: 'LLM-Antwort (oder geparstes JSON bei parseJson)' },
  ],
  features: {
    canAddOutputPorts: true,
  },
};
