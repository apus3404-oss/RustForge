<script lang="ts">
  import { onMount } from 'svelte';
  import WorkflowBuilder from '../components/WorkflowBuilder.svelte';
  import { workflows } from '../lib/stores';
  import { navigate } from '../lib/router';
  import { toasts } from '../lib/toast';
  import type { WorkflowDefinition } from '../lib/types';

  let loading = $state(true);

  onMount(async () => {
    try {
      await workflows.load();
    } catch (error) {
      console.error('Failed to load workflows:', error);
      toasts.show('Failed to load workflows', 'error');
    } finally {
      loading = false;
    }
  });

  async function handleSave(workflow: WorkflowDefinition) {
    try {
      await workflows.create(workflow);
      toasts.show(`Workflow "${workflow.name}" saved successfully!`, 'success');
    } catch (error) {
      console.error('Failed to save workflow:', error);
      toasts.show('Failed to save workflow. Please try again.', 'error');
    }
  }

  function handleViewWorkflow(id: string) {
    navigate(`/workflows/${id}`);
  }
</script>

<div class="workflows-page">
  <div class="page-header">
    <h1>Workflows</h1>
    <p>Create and manage multi-agent workflows</p>
  </div>

  <div class="workflow-builder-container">
    <WorkflowBuilder onSave={handleSave} />
  </div>

  {#if $workflows.length > 0}
    <div class="workflows-list">
      <h2>Saved Workflows</h2>
      <div class="workflow-cards">
        {#each $workflows as workflow}
          <div class="workflow-card">
            <h3>{workflow.name}</h3>
            <p>{workflow.description || 'No description'}</p>
            <div class="workflow-meta">
              <span>{workflow.agents.length} agents</span>
            </div>
            <button on:click={() => handleViewWorkflow(workflow.id)}>
              View Details
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .workflows-page {
    padding: 2rem;
    max-width: 1400px;
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

  .workflow-builder-container {
    margin-bottom: 3rem;
  }

  .workflows-list {
    margin-top: 3rem;
  }

  .workflows-list h2 {
    font-size: 1.5rem;
    color: #00d4ff;
    margin-bottom: 1.5rem;
  }

  .workflow-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1.5rem;
  }

  .workflow-card {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .workflow-card h3 {
    color: #fff;
    margin: 0;
  }

  .workflow-card p {
    color: #999;
    flex: 1;
    margin: 0;
  }

  .workflow-meta {
    color: #666;
    font-size: 0.875rem;
  }

  .workflow-card button {
    padding: 0.5rem 1rem;
    background: transparent;
    color: #00d4ff;
    border: 1px solid #00d4ff;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .workflow-card button:hover {
    background: rgba(0, 212, 255, 0.1);
  }
</style>
