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
  const data = await api.listWorkflows();
  workflows.set(data);
}

export async function loadExecutions() {
  const data = await api.listExecutions();
  executions.set(data);
}

// Store actions for managing events
export function addEvent(event: AgentEvent) {
  events.update(e => [...e, event]);
}

export function clearEvents() {
  events.set([]);
}
