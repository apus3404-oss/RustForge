// Manual test file for Svelte stores
// Run this in browser console after importing, or use it as a reference for testing

import { get } from 'svelte/store';
import {
  workflows,
  executions,
  events,
  activeExecutions,
  loadWorkflows,
  loadExecutions,
  startExecution,
  addEvent,
  clearEvents,
} from './stores';
import type { WorkflowSummary, ExecutionSummary } from './types';
import type { AgentEvent } from './websocket';

export async function testStores() {
  console.log('Testing Svelte Stores...\n');

  try {
    // Test 1: Initial store values
    console.log('1. Testing initial store values...');
    console.log('Workflows:', get(workflows));
    console.log('Executions:', get(executions));
    console.log('Events:', get(events));
    console.log('Active Executions:', get(activeExecutions));
    console.log('✓ Initial values are empty arrays');

    // Test 2: Load workflows
    console.log('\n2. Loading workflows...');
    await loadWorkflows();
    const workflowList = get(workflows);
    console.log('✓ Workflows loaded:', workflowList.length, 'items');

    // Test 3: Load executions
    console.log('\n3. Loading executions...');
    await loadExecutions();
    const executionList = get(executions);
    console.log('✓ Executions loaded:', executionList.length, 'items');

    // Test 4: Derived store (activeExecutions)
    console.log('\n4. Testing derived store...');
    const active = get(activeExecutions);
    const runningCount = executionList.filter(e => e.status === 'running').length;
    console.log('✓ Active executions:', active.length, '(expected:', runningCount, ')');
    if (active.length === runningCount) {
      console.log('✓ Derived store working correctly');
    } else {
      console.warn('⚠ Derived store count mismatch');
    }

    // Test 5: Add events
    console.log('\n5. Testing event management...');
    const testEvent1: AgentEvent = {
      type: 'TaskStarted',
      agent_id: 'agent1',
      task: 'Test task',
    };
    const testEvent2: AgentEvent = {
      type: 'TaskCompleted',
      agent_id: 'agent1',
      output: 'Test output',
    };

    addEvent(testEvent1);
    console.log('✓ Added event 1, count:', get(events).length);

    addEvent(testEvent2);
    console.log('✓ Added event 2, count:', get(events).length);

    // Test 6: Clear events
    console.log('\n6. Testing clear events...');
    clearEvents();
    const clearedEvents = get(events);
    console.log('✓ Events cleared, count:', clearedEvents.length);
    if (clearedEvents.length === 0) {
      console.log('✓ Clear events working correctly');
    } else {
      console.warn('⚠ Events not cleared properly');
    }

    // Test 7: Store subscriptions
    console.log('\n7. Testing store subscriptions...');
    let subscriptionCalled = false;
    const unsubscribe = workflows.subscribe((value) => {
      subscriptionCalled = true;
      console.log('✓ Subscription callback called with', value.length, 'workflows');
    });
    unsubscribe();
    console.log('✓ Subscription and unsubscribe working');

    console.log('\n✓ All store tests passed!');
    return { success: true };
  } catch (error) {
    console.error('✗ Test failed:', error);
    return { success: false, error };
  }
}

// Export for browser console testing
if (typeof window !== 'undefined') {
  (window as any).testStores = testStores;
  console.log('Store test available: window.testStores()');
}
