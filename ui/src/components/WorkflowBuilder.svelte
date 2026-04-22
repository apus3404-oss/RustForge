<script lang="ts">
  import {
    SvelteFlow,
    Background,
    Controls,
    type Node,
    type Edge,
    type Connection,
  } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/style.css';
  import AgentNode from './AgentNode.svelte';
  import type { WorkflowDefinition, AgentConfig } from '../lib/types';

  interface Props {
    initialWorkflow?: WorkflowDefinition;
    onSave?: (workflow: WorkflowDefinition) => void;
  }

  let { initialWorkflow, onSave }: Props = $props();

  // Node types registry
  const nodeTypes = {
    agent: AgentNode,
  };

  // State management with Svelte 5 runes
  let nodes = $state<Node[]>([]);
  let edges = $state<Edge[]>([]);
  let selectedNode = $state<Node | null>(null);
  let workflowName = $state('New Workflow');
  let workflowMode = $state<'sequential' | 'parallel' | 'dag'>('sequential');

  // Load initial workflow if provided
  $effect(() => {
    if (initialWorkflow) {
      workflowName = initialWorkflow.name;
      workflowMode = initialWorkflow.mode;
      loadWorkflowIntoCanvas(initialWorkflow);
    }
  });

  function loadWorkflowIntoCanvas(workflow: WorkflowDefinition) {
    // Convert WorkflowDefinition to nodes and edges
    const newNodes: Node[] = workflow.agents.map((agent, index) => ({
      id: agent.id,
      type: 'agent',
      position: { x: 100 + (index % 3) * 250, y: 100 + Math.floor(index / 3) * 150 },
      data: {
        agent_type: agent.agent_type,
        task: agent.task,
        tools: agent.config?.tools || [],
      },
    }));

    const newEdges: Edge[] = [];
    workflow.agents.forEach((agent) => {
      agent.depends_on.forEach((depId) => {
        newEdges.push({
          id: `${depId}-${agent.id}`,
          source: depId,
          target: agent.id,
          type: 'default',
        });
      });
    });

    nodes = newNodes;
    edges = newEdges;
  }

  function addNode(agentType: string) {
    const id = `agent-${Date.now()}`;
    const newNode: Node = {
      id,
      type: 'agent',
      position: { x: 100 + nodes.length * 50, y: 100 + nodes.length * 50 },
      data: {
        agent_type: agentType,
        task: '',
        tools: [],
      },
    };
    nodes = [...nodes, newNode];
  }

  function onConnect(connection: Connection) {
    const edge: Edge = {
      id: `${connection.source}-${connection.target}`,
      source: connection.source!,
      target: connection.target!,
      type: 'default',
    };
    edges = [...edges, edge];
  }

  function onNodeClick(event: CustomEvent) {
    selectedNode = event.detail.node;
  }

  function onPaneClick() {
    selectedNode = null;
  }

  function deleteSelectedNode() {
    if (!selectedNode) return;

    nodes = nodes.filter((n) => n.id !== selectedNode.id);
    edges = edges.filter((e) => e.source !== selectedNode.id && e.target !== selectedNode.id);
    selectedNode = null;
  }

  function exportWorkflow(): WorkflowDefinition {
    // Build dependency map from edges
    const dependencyMap = new Map<string, string[]>();
    edges.forEach((edge) => {
      const deps = dependencyMap.get(edge.target) || [];
      deps.push(edge.source);
      dependencyMap.set(edge.target, deps);
    });

    // Convert nodes to AgentConfig
    const agents: AgentConfig[] = nodes.map((node) => ({
      id: node.id,
      agent_type: node.data.agent_type,
      task: node.data.task || '',
      depends_on: dependencyMap.get(node.id) || [],
      config: node.data.tools.length > 0 ? { tools: node.data.tools } : undefined,
    }));

    return {
      name: workflowName,
      mode: workflowMode,
      agents,
    };
  }

  function handleSave() {
    const workflow = exportWorkflow();
    if (onSave) {
      onSave(workflow);
    } else {
      // Fallback: download as JSON
      const blob = new Blob([JSON.stringify(workflow, null, 2)], {
        type: 'application/json',
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${workflowName.replace(/\s+/g, '-').toLowerCase()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    }
  }

  function updateNodeData(field: string, value: any) {
    if (!selectedNode) return;

    nodes = nodes.map((node) => {
      if (node.id === selectedNode.id) {
        return {
          ...node,
          data: {
            ...node.data,
            [field]: value,
          },
        };
      }
      return node;
    });

    // Update selectedNode reference
    selectedNode = nodes.find((n) => n.id === selectedNode.id) || null;
  }
</script>

<div class="workflow-builder">
  <div class="toolbar">
    <div class="toolbar-section">
      <h3>Workflow</h3>
      <input
        type="text"
        bind:value={workflowName}
        placeholder="Workflow name"
        class="workflow-name-input"
      />
      <select bind:value={workflowMode} class="mode-select">
        <option value="sequential">Sequential</option>
        <option value="parallel">Parallel</option>
        <option value="dag">DAG</option>
      </select>
    </div>

    <div class="toolbar-section">
      <h3>Add Agent</h3>
      <button onclick={() => addNode('ResearchAgent')} class="add-btn">
        📚 Research
      </button>
      <button onclick={() => addNode('AnalysisAgent')} class="add-btn">
        🔍 Analysis
      </button>
      <button onclick={() => addNode('CodeAgent')} class="add-btn">
        💻 Code
      </button>
      <button onclick={() => addNode('TestAgent')} class="add-btn">
        🧪 Test
      </button>
    </div>

    <div class="toolbar-section">
      <h3>Actions</h3>
      <button onclick={handleSave} class="action-btn save">
        💾 Save
      </button>
      {#if selectedNode}
        <button onclick={deleteSelectedNode} class="action-btn delete">
          🗑️ Delete
        </button>
      {/if}
    </div>

    <div class="toolbar-stats">
      <div class="stat">
        <span class="stat-label">Nodes:</span>
        <span class="stat-value">{nodes.length}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Edges:</span>
        <span class="stat-value">{edges.length}</span>
      </div>
    </div>
  </div>

  <div class="canvas">
    <SvelteFlow
      {nodes}
      {edges}
      {nodeTypes}
      onconnect={onConnect}
      onnodeclick={onNodeClick}
      onpaneclick={onPaneClick}
      fitView
    >
      <Background color="#1e3a5f" gap={20} />
      <Controls />
    </SvelteFlow>
  </div>

  {#if selectedNode}
    <div class="properties-panel">
      <div class="panel-header">
        <h3>Node Properties</h3>
        <button onclick={() => (selectedNode = null)} class="close-btn">×</button>
      </div>

      <div class="panel-body">
        <div class="form-group">
          <label>Agent Type</label>
          <input
            type="text"
            value={selectedNode.data.agent_type}
            oninput={(e) => updateNodeData('agent_type', e.currentTarget.value)}
            class="form-input"
          />
        </div>

        <div class="form-group">
          <label>Task Description</label>
          <textarea
            value={selectedNode.data.task}
            oninput={(e) => updateNodeData('task', e.currentTarget.value)}
            placeholder="Describe what this agent should do..."
            class="form-textarea"
            rows="4"
          ></textarea>
        </div>

        <div class="form-group">
          <label>Tools (comma-separated)</label>
          <input
            type="text"
            value={selectedNode.data.tools.join(', ')}
            oninput={(e) =>
              updateNodeData(
                'tools',
                e.currentTarget.value.split(',').map((t) => t.trim()).filter(Boolean)
              )}
            placeholder="tool1, tool2, tool3"
            class="form-input"
          />
        </div>

        <div class="form-group">
          <label>Node ID</label>
          <input type="text" value={selectedNode.id} disabled class="form-input disabled" />
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .workflow-builder {
    display: grid;
    grid-template-columns: 250px 1fr;
    height: 100vh;
    background: #0a1628;
    font-family: 'JetBrains Mono', monospace;
    color: #e2e8f0;
  }

  .workflow-builder:has(.properties-panel) {
    grid-template-columns: 250px 1fr 320px;
  }

  .toolbar {
    background: #0f1f35;
    border-right: 1px solid #1e3a5f;
    padding: 1rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .toolbar-section h3 {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #00d9ff;
    margin-bottom: 0.75rem;
    font-weight: 600;
  }

  .workflow-name-input,
  .mode-select,
  .form-input,
  .form-textarea {
    width: 100%;
    background: #0a1628;
    border: 1px solid #1e3a5f;
    color: #e2e8f0;
    padding: 0.5rem;
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    border-radius: 2px;
    margin-bottom: 0.5rem;
  }

  .workflow-name-input:focus,
  .mode-select:focus,
  .form-input:focus,
  .form-textarea:focus {
    outline: none;
    border-color: #00d9ff;
    box-shadow: 0 0 0 2px rgba(0, 217, 255, 0.1);
  }

  .form-input.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .add-btn,
  .action-btn {
    width: 100%;
    background: #1e3a5f;
    border: 1px solid #2d4a6f;
    color: #e2e8f0;
    padding: 0.5rem;
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    cursor: pointer;
    border-radius: 2px;
    margin-bottom: 0.5rem;
    transition: all 0.2s ease;
    text-align: left;
  }

  .add-btn:hover,
  .action-btn:hover {
    background: #2d4a6f;
    border-color: #00d9ff;
  }

  .action-btn.save {
    background: #0d4d4d;
    border-color: #00d9ff;
  }

  .action-btn.save:hover {
    background: #0f6060;
  }

  .action-btn.delete {
    background: #4d0d0d;
    border-color: #ff4444;
  }

  .action-btn.delete:hover {
    background: #600f0f;
  }

  .toolbar-stats {
    margin-top: auto;
    padding-top: 1rem;
    border-top: 1px solid #1e3a5f;
  }

  .stat {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    margin-bottom: 0.5rem;
  }

  .stat-label {
    color: #718096;
  }

  .stat-value {
    color: #00d9ff;
    font-weight: 600;
  }

  .canvas {
    height: 100%;
    position: relative;
  }

  .properties-panel {
    background: #0f1f35;
    border-left: 1px solid #1e3a5f;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #1e3a5f;
  }

  .panel-header h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #00d9ff;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: #718096;
    font-size: 24px;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.2s ease;
  }

  .close-btn:hover {
    color: #00d9ff;
  }

  .panel-body {
    padding: 1rem;
    overflow-y: auto;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-group label {
    display: block;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #a0aec0;
    margin-bottom: 0.5rem;
  }

  /* Override SvelteFlow styles for blueprint theme */
  :global(.svelte-flow) {
    background: #0a1628;
  }

  :global(.svelte-flow__edge-path) {
    stroke: #00d9ff;
    stroke-width: 2;
  }

  :global(.svelte-flow__edge.selected .svelte-flow__edge-path) {
    stroke: #00ffff;
  }

  :global(.svelte-flow__controls) {
    background: #0f1f35;
    border: 1px solid #1e3a5f;
  }

  :global(.svelte-flow__controls-button) {
    background: #1e3a5f;
    border-bottom: 1px solid #2d4a6f;
    color: #e2e8f0;
  }

  :global(.svelte-flow__controls-button:hover) {
    background: #2d4a6f;
  }

  :global(.svelte-flow__controls-button svg) {
    fill: #e2e8f0;
  }
</style>
