<script lang="ts">
  import { currentPath, matchRoute } from '../lib/router';
  import Home from '../routes/Home.svelte';
  import Workflows from '../routes/Workflows.svelte';
  import WorkflowDetail from '../routes/WorkflowDetail.svelte';
  import Executions from '../routes/Executions.svelte';
  import ExecutionDetail from '../routes/ExecutionDetail.svelte';
  import SettingsPage from '../routes/SettingsPage.svelte';

  $: route = $currentPath;
  $: workflowParams = matchRoute('/workflows/:id', route);
  $: executionParams = matchRoute('/executions/:id', route);
</script>

<div class="router">
  {#if route === '/'}
    <Home />
  {:else if route === '/workflows'}
    <Workflows />
  {:else if workflowParams}
    <WorkflowDetail workflowId={workflowParams.id} />
  {:else if route === '/executions'}
    <Executions />
  {:else if executionParams}
    <ExecutionDetail executionId={executionParams.id} />
  {:else if route === '/settings'}
    <SettingsPage />
  {:else}
    <div class="not-found">
      <h1>404</h1>
      <p>Page not found</p>
      <button on:click={() => window.location.hash = '/'}>
        Go Home
      </button>
    </div>
  {/if}
</div>

<style>
  .router {
    min-height: 100vh;
  }

  .not-found {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 80vh;
    text-align: center;
  }

  .not-found h1 {
    font-size: 4rem;
    color: #00d4ff;
    margin-bottom: 1rem;
  }

  .not-found p {
    font-size: 1.25rem;
    color: #999;
    margin-bottom: 2rem;
  }

  .not-found button {
    padding: 0.75rem 1.5rem;
    background: #00d4ff;
    color: #000;
    border: none;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
  }
</style>
