# Phase 5: UI Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build web-based user interface with visual workflow builder, execution monitor, and real-time updates. Optional Tauri wrapper for desktop experience.

**Architecture:** Svelte frontend served by Axum backend. WebSocket for real-time execution updates. Svelte Flow for visual workflow builder. Optional Tauri wrapper for native desktop app.

**Tech Stack:** Svelte 5, Vite, TypeScript, @xyflow/svelte (workflow builder), Tailwind CSS, Tauri (optional)

---

## File Structure

**Frontend:**
- `ui/package.json` - NPM dependencies
- `ui/vite.config.ts` - Vite configuration
- `ui/tsconfig.json` - TypeScript configuration
- `ui/tailwind.config.js` - Tailwind CSS configuration
- `ui/src/main.ts` - Entry point
- `ui/src/App.svelte` - Root component
- `ui/src/lib/api.ts` - API client
- `ui/src/lib/websocket.ts` - WebSocket client
- `ui/src/lib/stores.ts` - Svelte stores
- `ui/src/components/WorkflowBuilder.svelte` - Visual workflow builder
- `ui/src/components/ExecutionMonitor.svelte` - Execution monitor
- `ui/src/components/ConversationView.svelte` - Agent conversation view
- `ui/src/components/Settings.svelte` - Settings panel
- `ui/src/routes/+page.svelte` - Home page
- `ui/src/routes/workflows/+page.svelte` - Workflows list
- `ui/src/routes/executions/+page.svelte` - Executions list

**Backend Updates:**
- Modify: `src/api/server.rs` - Serve static UI files

**Tauri (Optional):**
- `src-tauri/Cargo.toml` - Tauri dependencies
- `src-tauri/tauri.conf.json` - Tauri configuration
- `src-tauri/src/main.rs` - Tauri entry point

---

## Task 1: Frontend Project Setup

**Files:**
- Create: `ui/package.json`
- Create: `ui/vite.config.ts`
- Create: `ui/tsconfig.json`
- Create: `ui/tailwind.config.js`
- Create: `ui/src/main.ts`
- Create: `ui/src/App.svelte`
- Create: `ui/index.html`

**Key Steps:**
- [ ] Initialize npm project: `npm init -y`
- [ ] Install dependencies: svelte, vite, typescript, tailwindcss, @xyflow/svelte
- [ ] Configure Vite for Svelte
- [ ] Configure TypeScript
- [ ] Configure Tailwind CSS
- [ ] Create minimal App.svelte
- [ ] Test dev server: `npm run dev`
- [ ] Commit

**package.json:**
```json
{
  "name": "rustforge-ui",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@xyflow/svelte": "^0.1.0",
    "svelte": "^5.0.0"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^4.0.0",
    "@tsconfig/svelte": "^5.0.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "tailwindcss": "^3.4.0",
    "typescript": "^5.5.0",
    "vite": "^5.4.0"
  }
}
```

**vite.config.ts:**
```typescript
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: '../dist/ui',
  },
});
```

---

## Task 2: API Client

**Files:**
- Create: `ui/src/lib/api.ts`
- Create: `ui/src/lib/types.ts`

**Key Steps:**
- [ ] Write API client with fetch wrapper
- [ ] Implement workflow CRUD methods
- [ ] Implement execution methods
- [ ] Add error handling
- [ ] Define TypeScript types for API responses
- [ ] Test API client manually
- [ ] Commit

**Implementation:**
```typescript
// ui/src/lib/api.ts
const API_BASE = '/api';

export class ApiClient {
  async listWorkflows(): Promise<WorkflowSummary[]> {
    const response = await fetch(`${API_BASE}/workflows`);
    if (!response.ok) throw new Error('Failed to fetch workflows');
    return response.json();
  }

  async getWorkflow(id: string): Promise<WorkflowDefinition> {
    const response = await fetch(`${API_BASE}/workflows/${id}`);
    if (!response.ok) throw new Error('Failed to fetch workflow');
    return response.json();
  }

  async createWorkflow(definition: WorkflowDefinition): Promise<WorkflowResponse> {
    const response = await fetch(`${API_BASE}/workflows`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(definition),
    });
    if (!response.ok) throw new Error('Failed to create workflow');
    return response.json();
  }

  async executeWorkflow(
    workflowId: string,
    inputs: Record<string, any>
  ): Promise<ExecutionResponse> {
    const response = await fetch(`${API_BASE}/workflows/${workflowId}/execute`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(inputs),
    });
    if (!response.ok) throw new Error('Failed to execute workflow');
    return response.json();
  }

  async getExecution(executionId: string): Promise<ExecutionDetails> {
    const response = await fetch(`${API_BASE}/executions/${executionId}`);
    if (!response.ok) throw new Error('Failed to fetch execution');
    return response.json();
  }

  async listExecutions(): Promise<ExecutionSummary[]> {
    const response = await fetch(`${API_BASE}/executions`);
    if (!response.ok) throw new Error('Failed to fetch executions');
    return response.json();
  }
}

export const api = new ApiClient();
```

---

## Task 3: WebSocket Client

**Files:**
- Create: `ui/src/lib/websocket.ts`

**Key Steps:**
- [ ] Implement WebSocket client with reconnection logic
- [ ] Add event subscription
- [ ] Add event handlers (onTaskStarted, onTaskCompleted, etc.)
- [ ] Handle connection errors and reconnection
- [ ] Test WebSocket connection
- [ ] Commit

**Implementation:**
```typescript
// ui/src/lib/websocket.ts
export type AgentEvent = 
  | { type: 'TaskStarted'; agent_id: string; task: string }
  | { type: 'TaskCompleted'; agent_id: string; output: string }
  | { type: 'TaskFailed'; agent_id: string; error: string };

export class ExecutionWebSocket {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;

  constructor(
    private executionId: string,
    private onEvent: (event: AgentEvent) => void
  ) {}

  connect() {
    const wsUrl = `ws://localhost:3000/api/ws/executions/${this.executionId}`;
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
    };

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.onEvent(data);
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.ws.onclose = () => {
      console.log('WebSocket closed');
      this.attemptReconnect();
    };
  }

  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 10000);
      console.log(`Reconnecting in ${delay}ms...`);
      setTimeout(() => this.connect(), delay);
    }
  }

  send(command: any) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(command));
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}
```

---

## Task 4: Svelte Stores

**Files:**
- Create: `ui/src/lib/stores.ts`

**Key Steps:**
- [ ] Create writable stores for workflows, executions, events
- [ ] Create derived stores for filtered/sorted data
- [ ] Add store actions (loadWorkflows, startExecution, etc.)
- [ ] Test stores in components
- [ ] Commit

**Implementation:**
```typescript
// ui/src/lib/stores.ts
import { writable, derived } from 'svelte/store';
import { api } from './api';
import type { WorkflowSummary, ExecutionSummary, AgentEvent } from './types';

export const workflows = writable<WorkflowSummary[]>([]);
export const executions = writable<ExecutionSummary[]>([]);
export const events = writable<AgentEvent[]>([]);

export const activeExecutions = derived(
  executions,
  ($executions) => $executions.filter(e => e.status === 'Running')
);

export async function loadWorkflows() {
  const data = await api.listWorkflows();
  workflows.set(data);
}

export async function loadExecutions() {
  const data = await api.listExecutions();
  executions.set(data);
}

export function addEvent(event: AgentEvent) {
  events.update(e => [...e, event]);
}

export function clearEvents() {
  events.set([]);
}
```

---

## Task 5: Workflow Builder Component

**Files:**
- Create: `ui/src/components/WorkflowBuilder.svelte`
- Create: `ui/src/components/AgentNode.svelte`

**Key Steps:**
- [ ] Install @xyflow/svelte
- [ ] Create WorkflowBuilder component with SvelteFlow
- [ ] Create custom AgentNode component
- [ ] Implement node addition (drag from toolbar)
- [ ] Implement node connection (sequential flow)
- [ ] Implement node properties panel
- [ ] Convert visual graph to YAML workflow
- [ ] Test workflow builder
- [ ] Commit

**Implementation:**
```svelte
<!-- ui/src/components/WorkflowBuilder.svelte -->
<script lang="ts">
  import { SvelteFlow, Background, Controls } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/style.css';
  
  let nodes = $state([]);
  let edges = $state([]);
  let selectedNode = $state(null);
  
  function onNodeAdd(type: string) {
    const newNode = {
      id: `node-${Date.now()}`,
      type: 'agent',
      position: { x: 100, y: 100 },
      data: {
        agentType: type,
        task: '',
        tools: [],
      },
    };
    nodes = [...nodes, newNode];
  }
  
  function onConnect(connection) {
    edges = [...edges, connection];
  }
  
  function onNodeClick(event) {
    selectedNode = event.detail.node;
  }
  
  function exportWorkflow() {
    // Convert nodes and edges to WorkflowDefinition
    const workflow = {
      name: 'New Workflow',
      mode: 'sequential',
      agents: nodes.map(node => ({
        id: node.id,
        type: node.data.agentType,
        task: node.data.task,
        tools: node.data.tools,
      })),
    };
    
    return workflow;
  }
</script>

<div class="workflow-builder">
  <div class="toolbar">
    <button onclick={() => onNodeAdd('ResearchAgent')}>
      Add Research Agent
    </button>
    <button onclick={() => onNodeAdd('AnalysisAgent')}>
      Add Analysis Agent
    </button>
    <button onclick={() => exportWorkflow()}>
      Export YAML
    </button>
  </div>
  
  <div class="canvas">
    <SvelteFlow 
      {nodes} 
      {edges}
      onconnect={onConnect}
      onnodeclick={onNodeClick}
    >
      <Background />
      <Controls />
    </SvelteFlow>
  </div>
  
  {#if selectedNode}
    <div class="properties-panel">
      <h3>Node Properties</h3>
      <label>
        Agent Type:
        <input bind:value={selectedNode.data.agentType} />
      </label>
      <label>
        Task:
        <textarea bind:value={selectedNode.data.task}></textarea>
      </label>
      <label>
        Tools:
        <input bind:value={selectedNode.data.tools} />
      </label>
    </div>
  {/if}
</div>

<style>
  .workflow-builder {
    display: grid;
    grid-template-columns: 200px 1fr 300px;
    height: 100vh;
  }
  
  .canvas {
    height: 100%;
  }
  
  .properties-panel {
    padding: 1rem;
    border-left: 1px solid #ccc;
  }
</style>
```

---

## Task 6: Execution Monitor Component

**Files:**
- Create: `ui/src/components/ExecutionMonitor.svelte`
- Create: `ui/src/components/Timeline.svelte`
- Create: `ui/src/components/ConversationView.svelte`

**Key Steps:**
- [ ] Create ExecutionMonitor component
- [ ] Connect to WebSocket for real-time updates
- [ ] Display execution status badge
- [ ] Show timeline of events
- [ ] Show agent conversation view
- [ ] Add pause/resume/cancel buttons
- [ ] Test with real execution
- [ ] Commit

**Implementation:**
```svelte
<!-- ui/src/components/ExecutionMonitor.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { ExecutionWebSocket } from '$lib/websocket';
  import { events, addEvent } from '$lib/stores';
  
  export let executionId: string;
  
  let status = $state('running');
  let ws: ExecutionWebSocket | null = null;
  
  onMount(() => {
    ws = new ExecutionWebSocket(executionId, (event) => {
      addEvent(event);
      
      if (event.type === 'ExecutionCompleted') {
        status = 'completed';
      } else if (event.type === 'ExecutionFailed') {
        status = 'failed';
      }
    });
    
    ws.connect();
  });
  
  onDestroy(() => {
    ws?.disconnect();
  });
  
  function pause() {
    ws?.send({ type: 'Pause' });
  }
  
  function resume() {
    ws?.send({ type: 'Resume' });
  }
  
  function cancel() {
    ws?.send({ type: 'Cancel' });
  }
</script>

<div class="execution-monitor">
  <div class="header">
    <h2>Execution {executionId}</h2>
    <span class="status-badge status-{status}">{status}</span>
  </div>
  
  <div class="controls">
    <button onclick={pause}>Pause</button>
    <button onclick={resume}>Resume</button>
    <button onclick={cancel}>Cancel</button>
  </div>
  
  <div class="timeline">
    <h3>Timeline</h3>
    {#each $events as event}
      <div class="event">
        <span class="event-type">{event.type}</span>
        <span class="event-agent">{event.agent_id}</span>
        {#if event.type === 'TaskCompleted'}
          <pre>{event.output}</pre>
        {:else if event.type === 'TaskFailed'}
          <pre class="error">{event.error}</pre>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .execution-monitor {
    padding: 2rem;
  }
  
  .status-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
  }
  
  .status-running {
    background: #3b82f6;
    color: white;
  }
  
  .status-completed {
    background: #10b981;
    color: white;
  }
  
  .status-failed {
    background: #ef4444;
    color: white;
  }
  
  .timeline {
    margin-top: 2rem;
  }
  
  .event {
    padding: 1rem;
    border-left: 2px solid #e5e7eb;
    margin-bottom: 1rem;
  }
  
  .error {
    color: #ef4444;
  }
</style>
```

---

## Task 7: Settings Component

**Files:**
- Create: `ui/src/components/Settings.svelte`

**Key Steps:**
- [ ] Create Settings component
- [ ] Fetch config from API
- [ ] Display LLM provider settings
- [ ] Display execution settings
- [ ] Display permission settings
- [ ] Implement save functionality
- [ ] Test settings update
- [ ] Commit

**Implementation:**
```svelte
<!-- ui/src/components/Settings.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  
  let config = $state(null);
  let saving = $state(false);
  
  onMount(async () => {
    config = await api.getConfig();
  });
  
  async function saveConfig() {
    saving = true;
    try {
      await api.updateConfig(config);
      alert('Settings saved successfully');
    } catch (err) {
      alert('Failed to save settings: ' + err.message);
    } finally {
      saving = false;
    }
  }
</script>

{#if config}
  <div class="settings">
    <h2>Settings</h2>
    
    <section>
      <h3>LLM Providers</h3>
      <label>
        Default Provider:
        <input bind:value={config.llm.default_provider} />
      </label>
      <label>
        Enable Fallback:
        <input type="checkbox" bind:checked={config.llm.fallback_enabled} />
      </label>
    </section>
    
    <section>
      <h3>Execution</h3>
      <label>
        Max Parallel Agents:
        <input type="number" bind:value={config.execution.max_parallel_agents} />
      </label>
      <label>
        Default Timeout (seconds):
        <input type="number" bind:value={config.execution.default_timeout} />
      </label>
    </section>
    
    <section>
      <h3>Permissions</h3>
      <label>
        Default Policy:
        <select bind:value={config.permissions.default_policy}>
          <option value="allow">Allow</option>
          <option value="deny">Deny</option>
          <option value="prompt">Prompt</option>
        </select>
      </label>
    </section>
    
    <button onclick={saveConfig} disabled={saving}>
      {saving ? 'Saving...' : 'Save Settings'}
    </button>
  </div>
{:else}
  <p>Loading settings...</p>
{/if}

<style>
  .settings {
    max-width: 600px;
    margin: 2rem auto;
    padding: 2rem;
  }
  
  section {
    margin-bottom: 2rem;
  }
  
  label {
    display: block;
    margin-bottom: 1rem;
  }
  
  input, select {
    display: block;
    width: 100%;
    padding: 0.5rem;
    margin-top: 0.25rem;
  }
</style>
```

---

## Task 8: Routing and Pages

**Files:**
- Create: `ui/src/routes/+page.svelte` (Home)
- Create: `ui/src/routes/workflows/+page.svelte` (Workflows list)
- Create: `ui/src/routes/workflows/[id]/+page.svelte` (Workflow detail)
- Create: `ui/src/routes/executions/+page.svelte` (Executions list)
- Create: `ui/src/routes/executions/[id]/+page.svelte` (Execution detail)
- Create: `ui/src/routes/settings/+page.svelte` (Settings)

**Key Steps:**
- [ ] Set up SvelteKit routing (or simple hash routing)
- [ ] Create home page with quick actions
- [ ] Create workflows list page
- [ ] Create workflow detail/builder page
- [ ] Create executions list page
- [ ] Create execution monitor page
- [ ] Create settings page
- [ ] Add navigation menu
- [ ] Test navigation
- [ ] Commit

---

## Task 9: Serve UI from Backend

**Files:**
- Modify: `src/api/server.rs`

**Key Steps:**
- [ ] Build UI: `cd ui && npm run build`
- [ ] Configure Axum to serve static files from dist/ui
- [ ] Add fallback route for SPA (serve index.html for all non-API routes)
- [ ] Test UI served from Rust backend
- [ ] Update build script to include UI build
- [ ] Commit

**Implementation:**
```rust
// src/api/server.rs
use tower_http::services::ServeDir;

pub async fn start_server(state: AppState, port: u16) -> Result<()> {
    let app = Router::new()
        // API routes...
        .route("/api/workflows", post(handlers::workflows::create_workflow))
        // ... other API routes ...
        
        // Serve UI static files
        .nest_service("/", ServeDir::new("dist/ui"))
        
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    // ... rest of server setup
}
```

---

## Task 10: Tauri Desktop Wrapper (Optional)

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`

**Key Steps:**
- [ ] Install Tauri CLI: `cargo install tauri-cli`
- [ ] Initialize Tauri: `cargo tauri init`
- [ ] Configure Tauri to use existing UI
- [ ] Add system tray integration
- [ ] Add native file picker for workflows
- [ ] Build desktop app: `cargo tauri build`
- [ ] Test desktop app on Windows/macOS/Linux
- [ ] Commit

**tauri.conf.json:**
```json
{
  "build": {
    "beforeDevCommand": "cd ui && npm run dev",
    "beforeBuildCommand": "cd ui && npm run build",
    "devPath": "http://localhost:5173",
    "distDir": "../dist/ui"
  },
  "package": {
    "productName": "RustForge",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "dialog": {
        "all": false,
        "open": true,
        "save": true
      }
    },
    "systemTray": {
      "iconPath": "icons/icon.png"
    },
    "windows": [
      {
        "title": "RustForge",
        "width": 1200,
        "height": 800
      }
    ]
  }
}
```

---

## Task 11: Documentation and Polish

**Files:**
- Modify: `README.md`
- Create: `docs/ui-guide.md`
- Create: `docs/screenshots/` (add screenshots)

**Key Steps:**
- [ ] Take screenshots of UI
- [ ] Document UI features and usage
- [ ] Add keyboard shortcuts documentation
- [ ] Update README with UI setup instructions
- [ ] Add demo video/GIF
- [ ] Final polish (loading states, error messages, animations)
- [ ] Commit

---

## Phase 5 Complete

**Deliverables:**
- ✅ Svelte web UI with Vite
- ✅ Visual workflow builder (Svelte Flow)
- ✅ Execution monitor with real-time updates
- ✅ Agent conversation view
- ✅ Settings panel
- ✅ WebSocket integration
- ✅ Responsive design with Tailwind CSS
- ✅ Optional Tauri desktop wrapper
- ✅ Documentation and screenshots

---

## All Phases Complete! 🎉

**RustForge MVP is now complete with:**
- ✅ Phase 1: Core Foundation (config, CLI, storage, engine)
- ✅ Phase 2: LLM & Agent Layer (Ollama, OpenAI, agents, memory)
- ✅ Phase 3: Tool Layer & Security (6 tools, permissions, audit)
- ✅ Phase 4: API & Execution Patterns (REST, WebSocket, parallel)
- ✅ Phase 5: UI Layer (web UI, workflow builder, monitor)

**Next Steps:**
1. Execute Phase 1 plan to build core foundation
2. Execute Phase 2 plan to add LLM and agents
3. Execute Phase 3 plan to add tools and security
4. Execute Phase 4 plan to add API and parallel execution
5. Execute Phase 5 plan to build UI
6. Launch on GitHub, Reddit, HackerNews
7. Gather feedback and iterate
8. Plan v1.0 features (supervisor pattern, vector memory, etc.)

**Estimated Timeline:**
- Phase 1: 1 week
- Phase 2: 1 week
- Phase 3: 1 week
- Phase 4: 1 week
- Phase 5: 1 week
- **Total MVP: 5 weeks**

Good luck building RustForge! 🚀