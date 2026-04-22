<script lang="ts">
  import { onMount } from 'svelte';
  import { workflows, executions } from '../lib/stores';
  import { navigate } from '../lib/router';
  import type { WorkflowDefinition } from '../lib/types';

  let { workflowId }: { workflowId: string } = $props();

  let workflow: WorkflowDefinition | null = null;
  let loading = true;

  onMount(async () => {
    try {
      await workflows.load();
      workflow = $workflows.find(w => w.id === workflowId) || null;
    } catch (error) {
      console.error('Failed to load workflow:', error);
    } finally {
      loading = false;
    }
  });

  async function handleExecute() {
    if (!workflow) return;

    try {
      const execution = await executions.start(workflow.id, {});
      alert(`Workflow execution started: ${execution.id}`);
      navigate(`/executions/${execution.id}`);
    } catch (error) {
      console.error('Failed to start execution:', error);
      alert('Failed to start execution. Check console for details.');
    }
  }

  function handleBack() {
    navigate('/workflows');
  }
</script>

<div class="workflow-detail">
  <div class="page-header">
    <button class="back-button" on:click={handleBack}>← Back to Workflows</button>
  </div>

  {#if loading}
    <div class="loading">Loading workflow...</div>
  {:else if !workflow}
    <div class="error">
      <h2>Workflow Not Found</h2>
      <p>The workflow with ID "{workflowId}" could not be found.</p>
      <button on:click={handleBack}>Back to Workflows</button>
    </div>
  {:else}
    <div class="workflow-info">
      <div class="info-header">
        <h1>{workflow.name}</h1>
        <button class="execute-button" on:click={handleExecute}>
          Execute Workflow
        </button>
      </div>

      {#if workflow.description}
        <p class="description">{workflow.description}</p>
      {/if}

      <div class="workflow-details">
        <div class="detail-section">
          <h2>Agents ({workflow.agents.length})</h2>
          <div class="agents-list">
            {#each workflow.agents as agent}
              <div class="agent-card">
                <h3>{agent.name}</h3>
                <p class="agent-type">{agent.type}</p>
                {#if agent.config.system_prompt}
                  <p class="agent-prompt">{agent.config.system_prompt}</p>
                {/if}
              </div>
            {/each}
          </div>
        </div>

        {#if workflow.edges.length > 0}
          <div class="detail-section">
            <h2>Connections ({workflow.edges.length})</h2>
            <div class="edges-list">
              {#each workflow.edges as edge}
                <div class="edge-item">
                  {edge.source} → {edge.target}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .workflow-detail {
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .page-header {
    margin-bottom: 2rem;
  }

  .back-button {
    padding: 0.5rem 1rem;
    background: transparent;
    color: #999;
    border: 1px solid #333;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .back-button:hover {
    color: #fff;
    border-color: #666;
  }

  .loading, .error {
    text-align: center;
    padding: 3rem;
    color: #999;
  }

  .error h2 {
    color: #ff4444;
    margin-bottom: 1rem;
  }

  .error button {
    margin-top: 1rem;
    padding: 0.75rem 1.5rem;
    background: #00d4ff;
    color: #000;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .workflow-info {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 2rem;
  }

  .info-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .info-header h1 {
    font-size: 2rem;
    color: #fff;
    margin: 0;
  }

  .execute-button {
    padding: 0.75rem 1.5rem;
    background: #00d4ff;
    color: #000;
    border: none;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .execute-button:hover {
    background: #00b8e6;
  }

  .description {
    color: #999;
    margin-bottom: 2rem;
  }

  .workflow-details {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .detail-section h2 {
    font-size: 1.25rem;
    color: #00d4ff;
    margin-bottom: 1rem;
  }

  .agents-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1rem;
  }

  .agent-card {
    background: #0d0d0d;
    border: 1px solid #333;
    border-radius: 4px;
    padding: 1rem;
  }

  .agent-card h3 {
    color: #fff;
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
  }

  .agent-type {
    color: #00d4ff;
    font-size: 0.875rem;
    margin: 0 0 0.5rem 0;
  }

  .agent-prompt {
    color: #666;
    font-size: 0.875rem;
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .edges-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .edge-item {
    padding: 0.75rem;
    background: #0d0d0d;
    border: 1px solid #333;
    border-radius: 4px;
    color: #999;
  }
</style>
