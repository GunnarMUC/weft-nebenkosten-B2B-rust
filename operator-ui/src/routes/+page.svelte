<script>
  import { onMount } from 'svelte';
  import { OperatorApiClient } from '$lib/api';

  let apiBase = $state('');
  let token = $state('');
  let client = $state(null);
  let connected = $state(false);
  let tasks = $state([]);
  let selectedTask = $state(null);
  let pollHandle = $state(null);
  let notes = $state('');
  let submitting = $state(false);
  let errorMsg = $state('');

  onMount(() => {
    apiBase = localStorage.getItem('nkcheck_api_url') || 'http://localhost:3000';
    token = localStorage.getItem('nkcheck_token') || '';
  });

  async function connect() {
    errorMsg = '';
    localStorage.setItem('nkcheck_api_url', apiBase);
    localStorage.setItem('nkcheck_token', token);

    const api = new OperatorApiClient(apiBase, token);
    const valid = await api.validateToken();
    if (!valid) {
      errorMsg = 'Ungueltiger Token oder API nicht erreichbar';
      return;
    }

    client = api;
    connected = true;
    await fetchTasks();
    pollHandle = setInterval(fetchTasks, 5000);
  }

  function disconnect() {
    connected = false;
    if (pollHandle) { clearInterval(pollHandle); pollHandle = null; }
    client = null;
    tasks = [];
    selectedTask = null;
  }

  async function fetchTasks() {
    if (!client) return;
    try {
      tasks = await client.fetchTasks();
    } catch {
      disconnect();
    }
  }

  async function submitDecision(action) {
    if (!client || !selectedTask) return;
    submitting = true;
    try {
      if (action === 'cancel') {
        await client.cancelTask(selectedTask.executionId);
      } else {
        await client.completeTask(
          selectedTask.executionId,
          selectedTask.nodeId,
          {
            decision: action === 'approve',
            notes: notes,
          }
        );
      }
      selectedTask = null;
      notes = '';
      await fetchTasks();
    } catch (e) {
      errorMsg = `Fehler beim Submit: ${e.message}`;
    } finally {
      submitting = false;
    }
  }

  function taskSeverity(task) {
    try { return task.metadata?.data?.severity ?? task.data?.severity ?? 0; }
    catch { return 0; }
  }

  function severityColor(sev) {
    if (sev >= 4) return 'border-l-red-500 bg-red-50';
    if (sev >= 2) return 'border-l-amber-500 bg-amber-50';
    return 'border-l-green-500 bg-green-50';
  }

  function formatDate(iso) {
    try { return new Date(iso).toLocaleString('de-DE'); }
    catch { return iso || '-'; }
  }
</script>

<div class="flex flex-col min-h-screen bg-gray-50 font-sans">
  {#if !connected}
    <!-- Login Screen -->
    <div class="flex-1 flex items-center justify-center p-6">
      <div class="bg-white rounded-xl shadow-lg p-8 w-full max-w-sm">
        <div class="text-center mb-6">
          <h1 class="text-2xl font-bold text-gray-800">NK-Check Operator</h1>
          <p class="text-sm text-gray-500 mt-1">Human Review Interface</p>
        </div>

        <label class="block text-xs font-medium text-gray-600 mb-1">Weft API URL</label>
        <input
          type="text" bind:value={apiBase}
          placeholder="http://localhost:3000"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg mb-4 text-sm focus:ring-2 focus:ring-indigo-300 focus:outline-none"
        />

        <label class="block text-xs font-medium text-gray-600 mb-1">Operator Token</label>
        <input
          type="password" bind:value={token}
          placeholder="Extension-Token"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg mb-2 text-sm focus:ring-2 focus:ring-indigo-300 focus:outline-none"
        />
        <p class="text-xs text-gray-400 mb-6">
          Token aus dem Weft-Dashboard (Einstellungen &rarr; Extension-Token)
        </p>

        {#if errorMsg}
          <div class="bg-red-50 text-red-700 text-sm p-3 rounded-lg mb-4">{errorMsg}</div>
        {/if}

        <button
          onclick={connect}
          disabled={!token || !apiBase}
          class="w-full bg-indigo-600 text-white py-2.5 rounded-lg hover:bg-indigo-700 disabled:opacity-40 font-medium transition-colors"
        >
          Verbinden
        </button>
      </div>
    </div>
  {:else}
    <!-- Connected: Split View -->
    <header class="bg-white border-b border-gray-200 px-6 py-3 flex items-center justify-between shadow-sm">
      <div class="flex items-center gap-3">
        <h1 class="text-lg font-bold text-gray-800">NK-Check Operator</h1>
        <span class="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded-full font-medium">Verbunden</span>
      </div>
      <div class="flex items-center gap-3">
        <span class="text-sm text-gray-500">{tasks.length} ausstehend</span>
        <button onclick={disconnect} class="text-sm text-red-600 hover:text-red-800 font-medium">Trennen</button>
      </div>
    </header>

    <div class="flex-1 flex overflow-hidden">
      <!-- Left: Task List -->
      <div class="w-80 border-r border-gray-200 bg-white overflow-y-auto">
        {#if tasks.length === 0}
          <div class="flex flex-col items-center justify-center h-64 text-gray-400">
            <span class="text-4xl mb-2">&#10003;</span>
            <p class="text-sm">Keine ausstehenden Tasks</p>
          </div>
        {:else}
          {#each tasks as task (task.executionId + '/' + task.nodeId)}
            {@const sev = taskSeverity(task)}
            <button
              onclick={() => { selectedTask = task; notes = ''; }}
              class="w-full text-left p-4 border-b border-gray-100 hover:bg-gray-50 transition-colors
                     border-l-4 {selectedTask?.executionId === task.executionId ? 'bg-indigo-50 border-l-indigo-500!' : severityColor(sev)}"
            >
              <div class="flex items-center justify-between mb-1">
                <span class="font-medium text-sm text-gray-800 truncate max-w-[200px]">
                  {task.title || 'Prufauftrag'}
                </span>
                {#if sev > 0}
                  <span class="text-xs px-1.5 py-0.5 rounded font-medium leading-tight
                    {sev >= 4 ? 'bg-red-100 text-red-700' : sev >= 2 ? 'bg-amber-100 text-amber-700' : 'bg-green-100 text-green-700'}">
                    {sev}/5
                  </span>
                {/if}
              </div>
              <p class="text-xs text-gray-500 truncate">{task.description || 'Keine Beschreibung'}</p>
              <p class="text-xs text-gray-400 mt-1">{formatDate(task.createdAt)}</p>
            </button>
          {/each}
        {/if}
      </div>

      <!-- Right: Detail Panel -->
      <div class="flex-1 overflow-y-auto p-6">
        {#if selectedTask}
          <div class="max-w-2xl">
            <div class="mb-6">
              <h2 class="text-xl font-bold text-gray-800 mb-1">{selectedTask.title || 'Prufauftrag'}</h2>
              {#if selectedTask.description}
                <p class="text-sm text-gray-600">{selectedTask.description}</p>
              {/if}
              <p class="text-xs text-gray-400 mt-1">
                Execution: <code class="bg-gray-100 px-1 rounded">{selectedTask.executionId}</code>
              </p>
            </div>

            <!-- Form Fields -->
            {#if selectedTask.formSchema?.fields && selectedTask.formSchema.fields.length > 0}
              <div class="space-y-3 mb-8">
                {#each selectedTask.formSchema.fields as field}
                  <div class="bg-gray-50 rounded-lg p-4 border border-gray-100">
                    <div class="flex items-center gap-2 mb-2">
                      <span class="text-[10px] font-mono uppercase bg-gray-200 px-1.5 py-0.5 rounded text-gray-500">
                        {field.fieldType}
                      </span>
                      <span class="text-xs font-medium text-gray-500">{field.key}</span>
                    </div>
                    {#if field.fieldType === 'display'}
                      <p class="text-sm text-gray-700 whitespace-pre-wrap leading-relaxed">
                        {JSON.stringify(field.value) ?? '(keine Daten)'}
                      </p>
                    {:else}
                      <p class="text-xs text-gray-400">Feld-Typ: {field.fieldType}</p>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}

            <!-- Notes -->
            <div class="mb-4">
              <label class="block text-xs font-medium text-gray-600 mb-1">Interne Notizen</label>
              <textarea
                bind:value={notes}
                rows="3"
                placeholder="Notizen zur Entscheidung..."
                class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-indigo-300 focus:outline-none resize-y"
              ></textarea>
            </div>

            <!-- Action Buttons -->
            <div class="flex gap-3">
              <button
                onclick={() => submitDecision('approve')}
                disabled={submitting}
                class="flex-1 bg-green-600 text-white py-2.5 rounded-lg hover:bg-green-700 disabled:opacity-40 font-medium transition-colors"
              >{submitting ? '...' : 'Genehmigen'}</button>
              <button
                onclick={() => submitDecision('reject')}
                disabled={submitting}
                class="flex-1 bg-red-600 text-white py-2.5 rounded-lg hover:bg-red-700 disabled:opacity-40 font-medium transition-colors"
              >{submitting ? '...' : 'Ablehnen'}</button>
              <button
                onclick={() => submitDecision('cancel')}
                disabled={submitting}
                class="flex-1 bg-gray-500 text-white py-2.5 rounded-lg hover:bg-gray-600 disabled:opacity-40 font-medium transition-colors"
              >{submitting ? '...' : 'Ueberspringen'}</button>
            </div>

            {#if errorMsg}
              <div class="bg-red-50 text-red-700 text-sm p-3 rounded-lg mt-4">{errorMsg}</div>
            {/if}
          </div>
        {:else}
          <div class="h-full flex items-center justify-center text-gray-400">
            <div class="text-center">
              <span class="text-5xl block mb-3">&larr;</span>
              <p class="text-sm">Task aus der Liste auswaehlen</p>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
