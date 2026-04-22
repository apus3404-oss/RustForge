# RustForge UI Guide

## Overview

The RustForge UI is a modern web interface for building, executing, and monitoring multi-agent workflows. Built with Svelte 5 and styled with a Technical Blueprint aesthetic, it provides an intuitive visual experience for workflow orchestration.

## Getting Started

### Prerequisites

- Node.js 18+ and npm
- RustForge backend running (see main README)

### Installation

```bash
cd ui
npm install
```

### Development

```bash
npm run dev
```

The UI will be available at `http://localhost:5173`

### Production Build

```bash
npm run build
npm run preview
```

## Features

### 1. Home Dashboard

The home page provides quick access to all major features:

- **Create Workflow** - Launch the visual workflow builder
- **View Executions** - Monitor running and completed workflows
- **Settings** - Configure API endpoints and preferences

### 2. Workflow Builder

A visual canvas for designing multi-agent workflows.

**Key Features:**
- Drag-and-drop agent nodes
- Visual connection editor for dependencies
- Real-time workflow validation
- Support for Sequential, Parallel, and DAG execution modes
- Export workflows as YAML

**Creating a Workflow:**
1. Click "Add Agent" to create a new agent node
2. Configure agent properties in the right panel:
   - Agent ID (unique identifier)
   - Agent Type (base, specialized, etc.)
   - Task description
3. Connect agents by dragging from output to input ports
4. Set workflow name and execution mode
5. Click "Save Workflow" to persist

**Execution Modes:**
- **Sequential** - Agents execute one after another
- **Parallel** - Agents execute concurrently
- **DAG** - Directed Acyclic Graph with dependencies

### 3. Workflows List

Browse and manage all workflows:

- View workflow summaries
- See execution mode and agent count
- Quick access to workflow details
- Execute workflows directly from the list

### 4. Execution Monitor

Real-time monitoring of workflow execution:

**Features:**
- Live event timeline with WebSocket updates
- Execution status (Running, Completed, Failed, Paused)
- Execution controls (Pause, Resume, Cancel)
- Detailed event logs with timestamps
- Agent output and error messages
- Visual status indicators

**Event Types:**
- **WorkflowStarted** - Workflow execution begins
- **TaskStarted** - Agent begins processing
- **TaskCompleted** - Agent completes successfully
- **TaskFailed** - Agent encounters an error
- **WorkflowCompleted** - All agents finished

### 5. Execution History

View past workflow executions:

- Execution status and timestamps
- Workflow identification
- Error messages for failed executions
- Click any execution to view detailed timeline

### 6. Settings

Configure the UI and backend connection:

- **API Base URL** - Backend server endpoint (default: `http://localhost:3000`)
- **WebSocket URL** - Real-time event stream (default: `ws://localhost:3000`)
- **Auto-refresh** - Automatic data updates
- **Theme preferences** (future)

## UI Components

### Navigation

Top navigation bar with:
- Logo and branding
- Page links (Home, Workflows, Executions, Settings)
- Active page highlighting

### Status Badges

Color-coded status indicators:
- **Running** - Cyan (#00d4ff)
- **Completed** - Green (#00ff88)
- **Failed** - Red (#ff4444)
- **Paused** - Yellow (#fbbf24)

### Empty States

Helpful placeholders when no data exists:
- "No workflows yet" - with "Create Workflow" button
- "No executions yet" - with "Create a Workflow" button
- "Waiting for events..." - with pulse animation

### Error Handling

User-friendly error messages:
- API connection errors
- Workflow validation errors
- Execution failures with details
- Dismissible error banners

## Technical Details

### Architecture

```
ui/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── AgentNode.svelte        # Workflow canvas node
│   │   ├── ExecutionMonitor.svelte # Real-time execution view
│   │   ├── Navigation.svelte       # Top nav bar
│   │   ├── Router.svelte           # Client-side routing
│   │   ├── Settings.svelte         # Settings form
│   │   └── WorkflowBuilder.svelte  # Visual workflow editor
│   ├── routes/              # Page components
│   │   ├── Home.svelte
│   │   ├── Workflows.svelte
│   │   ├── WorkflowDetail.svelte
│   │   ├── Executions.svelte
│   │   ├── ExecutionDetail.svelte
│   │   └── SettingsPage.svelte
│   ├── lib/                 # Core utilities
│   │   ├── api.ts          # REST API client
│   │   ├── websocket.ts    # WebSocket client
│   │   ├── stores.ts       # Svelte stores
│   │   ├── router.ts       # Routing logic
│   │   └── types.ts        # TypeScript types
│   ├── App.svelte          # Root component
│   └── main.ts             # Entry point
└── public/                  # Static assets
```

### State Management

Uses Svelte 5 runes for reactive state:
- `$state` - Reactive variables
- `$derived` - Computed values
- `$effect` - Side effects
- Svelte stores for global state (workflows, executions, events)

### API Integration

REST API client (`lib/api.ts`) provides:
- `listWorkflows()` - Get all workflows
- `getWorkflow(id)` - Get workflow details
- `createWorkflow(workflow)` - Create new workflow
- `executeWorkflow(id)` - Start execution
- `getExecution(id)` - Get execution details
- `pauseExecution(id)` - Pause running execution
- `resumeExecution(id)` - Resume paused execution
- `cancelExecution(id)` - Cancel execution

### Real-time Updates

WebSocket client (`lib/websocket.ts`) streams execution events:
- Automatic reconnection on disconnect
- Event buffering during reconnection
- Type-safe event handling
- Integration with Svelte stores

## Design System

### Colors

**Primary Palette:**
- Background: `#0a0a0a` (dark)
- Surface: `#1a1a1a` (cards)
- Border: `#333` (subtle)
- Accent: `#00d4ff` (cyan)
- Text: `#fff` (primary), `#999` (secondary), `#666` (tertiary)

**Status Colors:**
- Success: `#00ff88`
- Error: `#ff4444`
- Warning: `#fbbf24`
- Info: `#00d4ff`

### Typography

- Font Family: `-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`
- Monospace: `'JetBrains Mono', monospace` (for IDs, code)
- Headings: Bold, larger sizes
- Body: Regular weight, readable sizes

### Spacing

Consistent spacing scale:
- `0.5rem` (8px) - Tight
- `1rem` (16px) - Normal
- `1.5rem` (24px) - Comfortable
- `2rem` (32px) - Spacious

### Animations

Subtle transitions for better UX:
- Hover effects: `transition: all 0.2s`
- Transform on hover: `translateY(-1px)`
- Pulse animation for loading states
- Smooth color transitions

## Troubleshooting

### UI won't connect to backend

1. Check backend is running: `rustforge serve`
2. Verify API URL in Settings matches backend port
3. Check browser console for CORS errors
4. Ensure WebSocket URL uses `ws://` not `wss://` for local dev

### Workflows not loading

1. Check API connection in Settings
2. Verify backend has workflows in database
3. Check browser console for errors
4. Try refreshing the page

### Execution events not updating

1. Verify WebSocket connection in browser DevTools (Network tab)
2. Check WebSocket URL in Settings
3. Ensure backend WebSocket server is running
4. Look for reconnection messages in console

### Build errors

1. Delete `node_modules` and reinstall: `rm -rf node_modules && npm install`
2. Clear Vite cache: `rm -rf .svelte-kit`
3. Check Node.js version: `node --version` (should be 18+)
4. Update dependencies: `npm update`

## Screenshots

### Home Dashboard
*Screenshot placeholder: Shows the home page with three action cards (Create Workflow, View Executions, Settings) on a dark background with cyan accents.*

### Workflow Builder
*Screenshot placeholder: Shows the visual workflow builder with agent nodes on a canvas, connection lines between nodes, and a properties panel on the right.*

### Execution Monitor
*Screenshot placeholder: Shows the execution monitor with a live event timeline, status badge, and control buttons (Pause, Resume, Cancel).*

### Executions List
*Screenshot placeholder: Shows a list of workflow executions with status badges, timestamps, and metadata.*

## Demo Video

A demo video should showcase:

1. **Opening the UI** (0:00-0:05)
   - Navigate to home page
   - Show clean, modern interface

2. **Creating a Workflow** (0:05-0:30)
   - Click "Create Workflow"
   - Add 2-3 agent nodes
   - Connect them with dependencies
   - Configure agent properties
   - Save workflow

3. **Executing a Workflow** (0:30-0:45)
   - Navigate to Workflows list
   - Click "Execute" on a workflow
   - Redirect to Execution Monitor

4. **Monitoring Execution** (0:45-1:15)
   - Show real-time event updates
   - Highlight status changes
   - Show agent outputs
   - Demonstrate pause/resume controls

5. **Viewing History** (1:15-1:30)
   - Navigate to Executions list
   - Show completed executions
   - Click to view detailed timeline

6. **Configuring Settings** (1:30-1:45)
   - Open Settings page
   - Show API configuration
   - Demonstrate save functionality

Total duration: ~2 minutes

## Best Practices

### Workflow Design

- Use descriptive agent IDs (e.g., `data-fetcher`, `analyzer`, `reporter`)
- Write clear task descriptions
- Test workflows with sequential mode first
- Use DAG mode for complex dependencies

### Monitoring

- Keep Execution Monitor open during runs
- Watch for error events immediately
- Use pause/resume for debugging
- Review completed executions for patterns

### Performance

- Limit parallel agents to reasonable numbers
- Use sequential mode for dependent tasks
- Monitor backend resource usage
- Clear old executions periodically

## Future Enhancements

Planned features:
- Keyboard shortcuts for common actions
- Dark/light theme toggle
- Workflow templates library
- Advanced filtering and search
- Export execution logs
- Workflow versioning
- Collaborative editing
- Mobile-responsive design

## Support

For issues or questions:
- Check the main README for backend setup
- Review browser console for errors
- Check backend logs for API issues
- File issues on GitHub repository
