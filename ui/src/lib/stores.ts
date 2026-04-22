// ui/src/lib/stores.ts
import { writable, derived } from 'svelte/store';
import { api } from './api';
import type { WorkflowSummary, ExecutionSummary } from './types';
import type { AgentEvent } from './websocket';

// Writable stores for core application state
export const workflows = writable<WorkflowSummary[]>([]);
export const executions = writable<ExecutionSummary[]>([]);
export const events = writable<AgentEvent[]>([]);

// Derived store for active executions (status === 'running')
export const activeExecutions = derived(
  executions,
  ($executions) => $executions.filter(e => e.status === 'running')
);

// Store actions for loading data from API
export async function loadWorkflows() {
  try {
    const data = await api.listWorkflows();
    workflows.set(data);
  } catch (error) {
    console.error('Failed to load workflows:', error);
    throw error; // Re-throw for component handling
  }
}

export async function loadExecutions() {
  try {
    const data = await api.listExecutions();
    executions.set(data);
  } catch (error) {
    console.error('Failed to load executions:', error);
    throw error; // Re-throw for component handling
  }
}

export async function startExecution(workflowId: string, inputs: Record<string, any>) {
  const result = await api.executeWorkflow(workflowId, inputs);
  await loadExecutions(); // Refresh executions list
  return result;
}

// Store actions for managing events
export function addEvent(event: AgentEvent) {
  events.update(e => [...e, event]);
}

export function clearEvents() {
  events.set([]);
}
