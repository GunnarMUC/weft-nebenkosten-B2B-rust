<script>
  import { onMount } from 'svelte';

  let token = $state('');
  let connected = $state(false);
  let tasks = $state([]);
  let selectedTask = $state(null);
  let pollInterval = null;
  let decision = $state('');
  let notes = $state('');
  let submitting = $state(false);
  let apiBase = '';

  onMount(() => {
    apiBase = localStorage.getItem('weft_api_url') || 'http://localhost:3000';
    token = localStorage.getItem('operator_token') || '';
    if (token) {
      connect();
    }
  });

  function connect() {
    localStorage.setItem('operator_token', token);
    localStorage.setItem('weft_api_url', apiBase);
    connected = true;
    fetchTasks();
    pollInterval = setInterval(fetchTasks, 5000);
  }

  function disconnect() {
    connected = false;
    if (pollInterval) clearInterval(pollInterval);
    tasks = [];
    selectedTask = null;
  }

  async function fetchTasks() {
    try {
      const res = await fetch(`${apiBase}/extension/token/${token}/tasks`);
      if (!res.ok) throw new Error('Invalid token');
      const data = await res.json();
      tasks = data.tasks || [];
    } catch (e) {
      console.error('Fetch error:', e);
      disconnect();
    }
  }

  function selectTask(task) {
    selectedTask = task;
    decision = '';
    notes = '';
  }

  async function submitDecision(action) {
    if (!selectedTask) return;
    submitting = true;
    try {
      const url = action === 'cancel'
        ? `${apiBase}/extension/token/${token}/cancel/${selectedTask.executionId}`
        : `${apiBase}/extension/token/${token}/complete/${selectedTask.executionId}`;

      const body = action === 'cancel' ? undefined : JSON.stringify({
        nodeId: selectedTask.nodeId,
        input: { decision: action === 'approve', notes: notes },
        callbackId: selectedTask.executionId + '-' + selectedTask.nodeId + '-0',
      });

      const res = await fetch(url, {
        method: 'POST',
        headers: body ? { 'Content-Type': 'application/json' } : {},
        body: body,
      });

      if (res.ok) {
        selectedTask = null;
        fetchTasks();
      }
    } catch (e) {
      console.error('Submit error:', e);
    } finally {
      submitting = false;
    }
  }

  function severityColor(meta) {
    try {
      const sev = meta?.data?.severity || 0;
      if (sev >= 4) return 'border-red-500 bg-red-50';
      if (sev >= 2) return 'border-amber-500 bg-amber-50';
      return 'border-green-500 bg-green-50';
    } catch { return 'border-gray-300 bg-white'; }
  }
</script>

<div class="flex flex-col h-screen bg-gray-50">
  {#if !connected}
    <div class="flex-1 flex items-center justify-center">
      <div class="bg-white rounded-lg shadow-lg p-8 w-96">
        <h1 class="text-2xl font-bold mb-6 text-gray-800">NK-Check Operator</h1>

        <label class="block text-sm font-medium text-gray-700 mb-1">Weft API URL</label>
        <input
          type="text" bind:value={apiBase}
          placeholder="http://localhost:3000"
          class="w-full px-3 py-2 border rounded-md mb-4 text-sm"
        />

        <label class="block text-sm font-medium text-gray-700 mb-1">Operator Token</label>
        <input
          type="password" bind:value={token}
          placeholder="Extension-Token eingeben"
          class="w-full px-3 py-2 border rounded-md mb-6 text-sm"
        />

        <button
          onclick={connect}
          disabled={!token}
          class="w-full bg-indigo-600 text-white py-2 rounded-md hover:bg-indigo-700 disabled:opacity-50 font-medium"
        >
          Verbinden
        </button>
      </div>
    </div>
  {:else}
    <header class="bg-white border-b px-6 py-3 flex items-center justify-between shadow-sm">
      <div class="flex items-center gap-3">
        <h1 class="text-lg font-bold text-gray-800">NK-Check Operator</h1>
        <span class="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded-full">Verbunden</span>
      </div>
      <div class="flex items-center gap-3">
        <span class="text-sm text-gray-500">{tasks.length} ausstehende Tasks</span>
        <button onclick={disconnect} class="text-sm text-red-600 hover:text-red-800">Trennen</button>
      </div>
    </header>

    <div class="flex-1 flex overflow-hidden">
      <!-- Task List -->
      <div class="w-80 border-r bg-white overflow-y-auto">
        {#if tasks.length === 0}
          <div class="p-6 text-center text-gray-400">
            <div class="text-4xl mb-2">&#10003;</div>
            <p class="text-sm">Keine ausstehenden Tasks</p>
          </div>
        {:else}
          {#each tasks as task (task.executionId + task.nodeId)}
            <button
              onclick={() => selectTask(task)}
              class="w-full text-left p-4 border-b hover:bg-gray-50 transition-colors
                     {selectedTask?.executionId === task.executionId ? 'bg-indigo-50 border-l-4 border-l-indigo-500' : ''}"
            >
              <div class="flex items-center justify-between mb-1">
                <span class="font-medium text-sm text-gray-800 truncate">{task.title || 'Task'}</span>
                <span class="{severityColor(task.metadata) + ' px-1.5 py-0.5 rounded text-xs font-medium'}">
                  {task.metadata?.data?.severity || '-'}
                </span>
              </div>
              <p class="text-xs text-gray-500 truncate">{task.description || 'Keine Beschreibung'}</p>
              <p class="text-xs text-gray-400 mt-1">{new Date(task.createdAt).toLocaleString('de-DE')}</p>
            </button>
          {/each}
        {/if}
      </div>

      <!-- Detail Panel -->
      <div class="flex-1 overflow-y-auto p-6">
        {#if selectedTask}
          <div class="max-w-2xl mx-auto">
            <h2 class="text-xl font-bold text-gray-800 mb-2">{selectedTask.title}</h2>
            <p class="text-sm text-gray-600 mb-6">{selectedTask.description}</p>

            {#if selectedTask.formSchema?.fields}
              <div class="space-y-4 mb-8">
                {#each selectedTask.formSchema.fields as field}
                  <div class="bg-gray-50 rounded-lg p-4">
                    <p class="text-xs text-gray-400 uppercase mb-1">{field.fieldType}</p>
                    {#if field.fieldType === 'display'}
                      <p class="text-sm text-gray-700 whitespace-pre-wrap">{field.value || '(keine Daten)'}</p>
                    {:else if field.fieldType === 'approve_reject'}
                      <!-- handled by buttons below -->
                    {:else}
                      <p class="text-sm font-medium text-gray-800">{field.key}</p>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}

            <div class="mb-4">
              <label class="block text-sm font-medium text-gray-700 mb-1">Notizen (optional)</label>
              <textarea
                bind:value={notes}
                rows="3"
                placeholder="Interne Notizen zur Entscheidung..."
                class="w-full px-3 py-2 border rounded-md text-sm"
              ></textarea>
            </div>

            <div class="flex gap-3">
              <button
                onclick={() => submitDecision('approve')}
                disabled={submitting}
                class="flex-1 bg-green-600 text-white py-2.5 rounded-md hover:bg-green-700 disabled:opacity-50 font-medium"
              >
                {submitting ? '...' : 'Genehmigen'}
              </button>
              <button
                onclick={() => submitDecision('reject')}
                disabled={submitting}
                class="flex-1 bg-red-600 text-white py-2.5 rounded-md hover:bg-red-700 disabled:opacity-50 font-medium"
              >
                {submitting ? '...' : 'Ablehnen'}
              </button>
              <button
                onclick={() => submitDecision('cancel')}
                disabled={submitting}
                class="flex-1 bg-gray-600 text-white py-2.5 rounded-md hover:bg-gray-700 disabled:opacity-50 font-medium"
              >
                {submitting ? '...' : 'Uberspringen'}
              </button>
            </div>
          </div>
        {:else}
          <div class="h-full flex items-center justify-center text-gray-400">
            <div class="text-center">
              <div class="text-5xl mb-3">&#8592;</div>
              <p>Task auswahlen zur Bearbeitung</p>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
