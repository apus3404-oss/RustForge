// Manual test file for API client
// Run this in browser console after importing, or use it as a reference for testing

import { api } from './api';
import type { WorkflowDefinition } from './types';

// Test workflow definition
const testWorkflow: WorkflowDefinition = {
  name: 'Test Workflow',
  mode: 'sequential',
  agents: [
    {
      id: 'agent1',
      agent_type: 'researcher',
      task: 'Research topic',
      depends_on: [],
    },
    {
      id: 'agent2',
      agent_type: 'writer',
      task: 'Write report',
      depends_on: ['agent1'],
    },
  ],
  inputs: {
    topic: 'AI trends',
  },
};

export async function testApiClient() {
  console.log('Testing API Client...\n');

  try {
    // Test 1: List workflows
    console.log('1. Listing workflows...');
    const workflows = await api.listWorkflows();
    console.log('✓ Workflows:', workflows);

    // Test 2: Create workflow
    console.log('\n2. Creating workflow...');
    const created = await api.createWorkflow(testWorkflow);
    console.log('✓ Created:', created);

    // Test 3: Get workflow
    console.log('\n3. Getting workflow...');
    const workflow = await api.getWorkflow(created.id);
    console.log('✓ Workflow:', workflow);

    // Test 4: Execute workflow
    console.log('\n4. Executing workflow...');
    const execution = await api.executeWorkflow(created.id, { topic: 'AI trends' });
    console.log('✓ Execution started:', execution);

    // Test 5: Get execution
    console.log('\n5. Getting execution details...');
    const executionDetails = await api.getExecution(execution.execution_id);
    console.log('✓ Execution details:', executionDetails);

    // Test 6: List executions
    console.log('\n6. Listing executions...');
    const executions = await api.listExecutions();
    console.log('✓ Executions:', executions);

    console.log('\n✓ All tests passed!');
    return { success: true, workflowId: created.id, executionId: execution.execution_id };
  } catch (error) {
    console.error('✗ Test failed:', error);
    return { success: false, error };
  }
}

// Export for browser console testing
if (typeof window !== 'undefined') {
  (window as any).testApiClient = testApiClient;
  console.log('API test available: window.testApiClient()');
}
