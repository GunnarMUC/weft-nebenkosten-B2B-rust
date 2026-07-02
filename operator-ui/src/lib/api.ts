// Operator UI API Client
// Interface to Weft Extension API for listing/ completing human review tasks.

const DEFAULT_API_URL = 'http://localhost:3000';

interface PendingTask {
  executionId: string;
  nodeId: string;
  title: string;
  description?: string;
  data?: Record<string, unknown>;
  createdAt: string;
  taskType?: string;
  actionUrl?: string;
  formSchema?: {
    fields: FormField[];
  };
  metadata: Record<string, unknown>;
}

interface FormField {
  fieldType: string;
  key: string;
  render?: Record<string, unknown>;
  value?: unknown;
  config?: Record<string, unknown>;
}

interface TaskListResponse {
  tasks: PendingTask[];
}

class OperatorApiClient {
  private apiBase: string;
  private token: string;

  constructor(apiBase: string, token: string) {
    this.apiBase = apiBase.replace(/\/$/, '');
    this.token = token;
  }

  async fetchTasks(): Promise<PendingTask[]> {
    const res = await fetch(`${this.apiBase}/extension/token/${this.token}/tasks`);
    if (!res.ok) throw new Error(`API error ${res.status}: ${res.statusText}`);
    const data: TaskListResponse = await res.json();
    return data.tasks || [];
  }

  async completeTask(executionId: string, nodeId: string, input: Record<string, unknown>): Promise<void> {
    const res = await fetch(
      `${this.apiBase}/extension/token/${this.token}/complete/${executionId}`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          nodeId,
          input,
          callbackId: `${executionId}-${nodeId}`,
        }),
      }
    );
    if (!res.ok) throw new Error(`Complete error ${res.status}`);
  }

  async cancelTask(executionId: string): Promise<void> {
    const res = await fetch(
      `${this.apiBase}/extension/token/${this.token}/cancel/${executionId}`,
      { method: 'POST' }
    );
    if (!res.ok) throw new Error(`Cancel error ${res.status}`);
  }

  async validateToken(): Promise<boolean> {
    try {
      const res = await fetch(`${this.apiBase}/extension/token/${this.token}/health`);
      return res.ok;
    } catch {
      return false;
    }
  }
}

export { OperatorApiClient };
export type { PendingTask, FormField, TaskListResponse };
