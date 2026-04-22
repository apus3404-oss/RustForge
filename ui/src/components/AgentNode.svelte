<script lang="ts">
  import { Handle, Position } from '@xyflow/svelte';

  interface Props {
    data: {
      agent_type: string;
      task: string;
      tools: string[];
    };
    selected?: boolean;
  }

  let { data, selected = false }: Props = $props();
</script>

<div class="agent-node" class:selected>
  <Handle type="target" position={Position.Top} />

  <div class="node-header">
    <div class="node-icon">🤖</div>
    <div class="node-type">{data.agent_type}</div>
  </div>

  <div class="node-body">
    {#if data.task}
      <div class="node-task">{data.task}</div>
    {:else}
      <div class="node-task placeholder">No task defined</div>
    {/if}

    {#if data.tools && data.tools.length > 0}
      <div class="node-tools">
        {data.tools.length} tool{data.tools.length !== 1 ? 's' : ''}
      </div>
    {/if}
  </div>

  <Handle type="source" position={Position.Bottom} />
</div>

<style>
  .agent-node {
    background: #0a1628;
    border: 1px solid #1e3a5f;
    border-radius: 4px;
    min-width: 200px;
    font-family: 'JetBrains Mono', monospace;
    transition: all 0.2s ease;
  }

  .agent-node.selected {
    border-color: #00d9ff;
    box-shadow: 0 0 10px rgba(0, 217, 255, 0.3);
  }

  .node-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: #0f1f35;
    border-bottom: 1px solid #1e3a5f;
  }

  .node-icon {
    font-size: 16px;
  }

  .node-type {
    font-size: 12px;
    font-weight: 600;
    color: #00d9ff;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .node-body {
    padding: 12px;
  }

  .node-task {
    font-size: 11px;
    color: #a0aec0;
    line-height: 1.4;
    margin-bottom: 8px;
  }

  .node-task.placeholder {
    color: #4a5568;
    font-style: italic;
  }

  .node-tools {
    font-size: 10px;
    color: #718096;
    padding: 4px 8px;
    background: #0f1f35;
    border-radius: 2px;
    display: inline-block;
  }

  :global(.svelte-flow__handle) {
    background: #00d9ff;
    border: 2px solid #0a1628;
    width: 8px;
    height: 8px;
  }

  :global(.svelte-flow__handle:hover) {
    background: #00ffff;
  }
</style>
