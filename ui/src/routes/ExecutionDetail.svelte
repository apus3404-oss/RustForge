<script lang="ts">
  import ExecutionMonitor from '../components/ExecutionMonitor.svelte';
  import { navigate } from '../lib/router';

  let { executionId }: { executionId: string } = $props();

  function handleBack() {
    navigate('/executions');
  }
</script>

<div class="execution-detail">
  <div class="page-header">
    <button class="back-button" on:click={handleBack}>← Back to Executions</button>
  </div>

  <div class="monitor-container">
    <ExecutionMonitor {executionId} />
  </div>
</div>

<style>
  .execution-detail {
    padding: 2rem;
    max-width: 1400px;
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

  .monitor-container {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 2rem;
  }
</style>
