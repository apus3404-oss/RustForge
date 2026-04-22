<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { ExecutionWebSocket } from '$lib/websocket';
  import { events, addEvent, clearEvents } from '$lib/stores';
  import { api } from '$lib/api';

  export let executionId: string;

  let status = $state<'running' | 'completed' | 'failed' | 'paused'>('running');
  let ws: ExecutionWebSocket | null = null;
  let error = $state<string | null>(null);

  onMount(() => {
    clearEvents();

    ws = new ExecutionWebSocket(executionId, (event) => {
      addEvent(event);
    });

    ws.connect();

    // Poll execution status periodically
    const statusInterval = setInterval(async () => {
      try {
        const execution = await api.getExecution(executionId);
        status = execution.status as any;
      } catch (err) {
        console.error('Failed to fetch execution status:', err);
      }
    }, 2000);

    return () => {
      clearInterval(statusInterval);
    };
  });

  onDestroy(() => {
    ws?.disconnect();
  });

  async function pause() {
    try {
      await api.pauseExecution(executionId);
      status = 'paused';
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to pause execution';
    }
  }

  async function resume() {
    try {
      await api.resumeExecution(executionId);
      status = 'running';
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to resume execution';
    }
  }

  async function cancel() {
    try {
      await api.cancelExecution(executionId);
      status = 'failed';
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to cancel execution';
    }
  }

  function formatTimestamp(event: any) {
    return new Date().toLocaleTimeString();
  }
</script>

<div class="execution-monitor">
  <div class="header">
    <div class="title-section">
      <h2>Execution Monitor</h2>
      <span class="execution-id">{executionId}</span>
    </div>
    <span class="status-badge status-{status}">{status.toUpperCase()}</span>
  </div>

  {#if error}
    <div class="error-banner">
      {error}
      <button onclick={() => error = null} class="dismiss">×</button>
    </div>
  {/if}

  <div class="controls">
    <button onclick={pause} disabled={status !== 'running'} class="control-btn">
      <span class="icon">⏸</span> Pause
    </button>
    <button onclick={resume} disabled={status !== 'paused'} class="control-btn">
      <span class="icon">▶</span> Resume
    </button>
    <button onclick={cancel} disabled={status === 'completed' || status === 'failed'} class="control-btn danger">
      <span class="icon">⏹</span> Cancel
    </button>
  </div>

  <div class="timeline">
    <h3>Event Timeline</h3>
    <div class="timeline-container">
      {#if $events.length === 0}
        <div class="empty-state">
          <span class="pulse"></span>
          Waiting for events...
        </div>
      {:else}
        {#each $events as event, i}
          <div class="event event-{event.type.toLowerCase()}">
            <div class="event-marker"></div>
            <div class="event-content">
              <div class="event-header">
                <span class="event-type">{event.type}</span>
                <span class="event-agent">Agent: {event.agent_id}</span>
                <span class="event-time">{formatTimestamp(event)}</span>
              </div>

              {#if event.type === 'TaskStarted'}
                <div class="event-body">
                  <span class="label">Task:</span>
                  <span class="task-name">{event.task}</span>
                </div>
              {:else if event.type === 'TaskCompleted'}
                <div class="event-body">
                  <span class="label">Output:</span>
                  <pre class="output">{event.output}</pre>
                </div>
              {:else if event.type === 'TaskFailed'}
                <div class="event-body error-body">
                  <span class="label">Error:</span>
                  <pre class="error-output">{event.error}</pre>
                </div>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .execution-monitor {
    background: #0a1628;
    color: #e5e7eb;
    padding: 2rem;
    min-height: 100vh;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid rgba(0, 217, 255, 0.2);
  }

  .title-section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .title-section h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: #00d9ff;
  }

  .execution-id {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.875rem;
    color: #9ca3af;
  }

  .status-badge {
    padding: 0.5rem 1rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    font-family: 'JetBrains Mono', monospace;
  }

  .status-running {
    background: rgba(0, 217, 255, 0.2);
    color: #00d9ff;
    border: 1px solid #00d9ff;
  }

  .status-completed {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
    border: 1px solid #10b981;
  }

  .status-failed {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
    border: 1px solid #ef4444;
  }

  .status-paused {
    background: rgba(251, 191, 36, 0.2);
    color: #fbbf24;
    border: 1px solid #fbbf24;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid #ef4444;
    color: #ef4444;
    padding: 1rem;
    border-radius: 0.25rem;
    margin-bottom: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .dismiss {
    background: none;
    border: none;
    color: #ef4444;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0;
    width: 2rem;
    height: 2rem;
  }

  .controls {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .control-btn {
    background: rgba(0, 217, 255, 0.1);
    border: 1px solid #00d9ff;
    color: #00d9ff;
    padding: 0.75rem 1.5rem;
    border-radius: 0.25rem;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    transition: all 0.2s;
  }

  .control-btn:hover:not(:disabled) {
    background: rgba(0, 217, 255, 0.2);
    transform: translateY(-1px);
  }

  .control-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .control-btn.danger {
    border-color: #ef4444;
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
  }

  .control-btn.danger:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.2);
  }

  .icon {
    font-size: 1rem;
  }

  .timeline {
    margin-top: 2rem;
  }

  .timeline h3 {
    margin: 0 0 1.5rem 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: #00d9ff;
  }

  .timeline-container {
    position: relative;
  }

  .empty-state {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 2rem;
    color: #9ca3af;
    font-style: italic;
  }

  .pulse {
    width: 0.5rem;
    height: 0.5rem;
    background: #00d9ff;
    border-radius: 50%;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.3;
    }
  }

  .event {
    display: flex;
    gap: 1rem;
    margin-bottom: 1.5rem;
    position: relative;
  }

  .event:not(:last-child)::after {
    content: '';
    position: absolute;
    left: 0.5rem;
    top: 2rem;
    bottom: -1.5rem;
    width: 2px;
    background: rgba(0, 217, 255, 0.2);
  }

  .event-marker {
    width: 1rem;
    height: 1rem;
    border-radius: 50%;
    border: 2px solid #00d9ff;
    background: #0a1628;
    flex-shrink: 0;
    margin-top: 0.25rem;
    z-index: 1;
  }

  .event-taskfailed .event-marker {
    border-color: #ef4444;
  }

  .event-taskcompleted .event-marker {
    border-color: #10b981;
    background: #10b981;
  }

  .event-content {
    flex: 1;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(0, 217, 255, 0.2);
    border-radius: 0.25rem;
    padding: 1rem;
  }

  .event-header {
    display: flex;
    gap: 1rem;
    margin-bottom: 0.75rem;
    flex-wrap: wrap;
  }

  .event-type {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.875rem;
    font-weight: 600;
    color: #00d9ff;
  }

  .event-agent {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem;
    color: #9ca3af;
  }

  .event-time {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem;
    color: #6b7280;
    margin-left: auto;
  }

  .event-body {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .label {
    font-size: 0.75rem;
    color: #9ca3af;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .task-name {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.875rem;
    color: #e5e7eb;
  }

  .output, .error-output {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.875rem;
    background: rgba(0, 0, 0, 0.3);
    padding: 0.75rem;
    border-radius: 0.25rem;
    overflow-x: auto;
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .output {
    color: #e5e7eb;
    border-left: 3px solid #10b981;
  }

  .error-output {
    color: #ef4444;
    border-left: 3px solid #ef4444;
  }

  .error-body {
    background: rgba(239, 68, 68, 0.05);
    padding: 0.75rem;
    border-radius: 0.25rem;
  }
</style>
