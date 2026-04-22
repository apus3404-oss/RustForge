<script lang="ts">
  import { onMount } from 'svelte';
  import { executions } from '../lib/stores';
  import { navigate } from '../lib/router';

  onMount(async () => {
    try {
      await executions.load();
    } catch (error) {
      console.error('Failed to load executions:', error);
    }
  });

  function handleViewExecution(id: string) {
    navigate(`/executions/${id}`);
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'running': return '#00d4ff';
      case 'completed': return '#00ff88';
      case 'failed': return '#ff4444';
      default: return '#999';
    }
  }

  function formatDate(timestamp: string): string {
    return new Date(timestamp).toLocaleString();
  }
</script>

<div class="executions-page">
  <div class="page-header">
    <h1>Executions</h1>
    <p>Monitor workflow execution history</p>
  </div>

  {#if $executions.length === 0}
    <div class="empty-state">
      <p>No executions yet</p>
      <button on:click={() => navigate('/workflows')}>
        Create a Workflow
      </button>
    </div>
  {:else}
    <div class="executions-list">
      {#each $executions as execution}
        <div class="execution-card" on:click={() => handleViewExecution(execution.id)}>
          <div class="execution-header">
            <h3>{execution.workflow_id}</h3>
            <span
              class="status-badge"
              style="background: {getStatusColor(execution.status)}"
            >
              {execution.status}
            </span>
          </div>

          <div class="execution-meta">
            <div class="meta-item">
              <span class="label">ID:</span>
              <span class="value">{execution.id}</span>
            </div>
            <div class="meta-item">
              <span class="label">Started:</span>
              <span class="value">{formatDate(execution.started_at)}</span>
            </div>
            {#if execution.completed_at}
              <div class="meta-item">
                <span class="label">Completed:</span>
                <span class="value">{formatDate(execution.completed_at)}</span>
              </div>
            {/if}
          </div>

          {#if execution.error}
            <div class="error-message">
              Error: {execution.error}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .executions-page {
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .page-header {
    margin-bottom: 2rem;
  }

  .page-header h1 {
    font-size: 2rem;
    color: #fff;
    margin-bottom: 0.5rem;
  }

  .page-header p {
    color: #999;
  }

  .empty-state {
    text-align: center;
    padding: 4rem 2rem;
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
  }

  .empty-state p {
    color: #999;
    margin-bottom: 1.5rem;
    font-size: 1.125rem;
  }

  .empty-state button {
    padding: 0.75rem 1.5rem;
    background: #00d4ff;
    color: #000;
    border: none;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
  }

  .executions-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .execution-card {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 1.5rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .execution-card:hover {
    border-color: #00d4ff;
    transform: translateY(-2px);
  }

  .execution-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .execution-header h3 {
    color: #fff;
    margin: 0;
    font-size: 1.125rem;
  }

  .status-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.875rem;
    font-weight: 600;
    color: #000;
  }

  .execution-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
  }

  .meta-item {
    display: flex;
    gap: 0.5rem;
  }

  .label {
    color: #666;
    font-size: 0.875rem;
  }

  .value {
    color: #999;
    font-size: 0.875rem;
  }

  .error-message {
    margin-top: 1rem;
    padding: 0.75rem;
    background: rgba(255, 68, 68, 0.1);
    border: 1px solid #ff4444;
    border-radius: 4px;
    color: #ff4444;
    font-size: 0.875rem;
  }
</style>
