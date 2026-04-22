// Manual test file for ExecutionWebSocket
// Run this in browser console or create a test page to verify functionality

import { ExecutionWebSocket, type AgentEvent } from './websocket';

/**
 * Manual Test 1: Basic Connection and Event Handling
 *
 * Prerequisites: Backend server running at localhost:3000 with an active execution
 */
export function testBasicConnection(executionId: string) {
  console.log('Test 1: Basic Connection');

  const ws = new ExecutionWebSocket(executionId, (event: AgentEvent) => {
    console.log('Received event:', event);
  });

  ws.connect();

  // Return ws instance for manual testing
  return ws;
}

/**
 * Manual Test 2: Command Sending
 *
 * Usage: const ws = testCommandSending('exec-123');
 *        ws.send({ type: 'Pause' });
 */
export function testCommandSending(executionId: string) {
  console.log('Test 2: Command Sending');

  const ws = new ExecutionWebSocket(executionId, (event: AgentEvent) => {
    console.log('Event:', event);
  });

  ws.connect();

  // Wait for connection, then send commands
  setTimeout(() => {
    console.log('Sending Pause command...');
    ws.send({ type: 'Pause' });
  }, 1000);

  return ws;
}

/**
 * Manual Test 3: Intentional Disconnect (should NOT reconnect)
 *
 * Expected: Connection closes, no reconnection attempts
 */
export function testIntentionalDisconnect(executionId: string) {
  console.log('Test 3: Intentional Disconnect');

  const ws = new ExecutionWebSocket(executionId, (event: AgentEvent) => {
    console.log('Event:', event);
  });

  ws.connect();

  // Disconnect after 2 seconds
  setTimeout(() => {
    console.log('Calling disconnect() - should NOT reconnect');
    ws.disconnect();
  }, 2000);

  return ws;
}

/**
 * Manual Test 4: Memory Leak Prevention
 *
 * Expected: Timeout cleared, no memory leaks
 */
export function testMemoryLeakPrevention(executionId: string) {
  console.log('Test 4: Memory Leak Prevention');

  const ws = new ExecutionWebSocket(executionId, (event: AgentEvent) => {
    console.log('Event:', event);
  });

  ws.connect();

  // Disconnect immediately to test timeout cleanup
  setTimeout(() => {
    console.log('Disconnecting to verify timeout cleanup...');
    ws.disconnect();
    console.log('If no reconnection attempts appear, timeout was cleared successfully');
  }, 100);

  return ws;
}

/**
 * Manual Test 5: Reconnection Logic (simulate server disconnect)
 *
 * Expected: Automatic reconnection with exponential backoff
 * Note: Requires manually stopping/starting backend to test
 */
export function testReconnection(executionId: string) {
  console.log('Test 5: Reconnection Logic');
  console.log('Stop the backend server after connection to test reconnection');

  let reconnectCount = 0;

  const ws = new ExecutionWebSocket(executionId, (event: AgentEvent) => {
    console.log('Event:', event);
  });

  ws.connect();

  // Monitor reconnection attempts (check console logs)
  console.log('Watch console for "Reconnecting in Xms..." messages');
  console.log('Expected delays: 2s, 4s, 8s, 10s (capped), 10s');

  return ws;
}

// Usage instructions
console.log(`
WebSocket Manual Tests Available:
- testBasicConnection('execution-id')
- testCommandSending('execution-id')
- testIntentionalDisconnect('execution-id')
- testMemoryLeakPrevention('execution-id')
- testReconnection('execution-id')

Example:
  import * as tests from './websocket.test';
  const ws = tests.testBasicConnection('my-execution-id');
  // Later: ws.disconnect();
`);
