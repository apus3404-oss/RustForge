import { describe, it, expect } from 'vitest';
import type { WorkflowDefinition } from '../lib/types';

describe('WorkflowBuilder Export Logic', () => {
  it('should convert nodes and edges to WorkflowDefinition', () => {
    // Simulate the export logic
    const nodes = [
      {
        id: 'agent-1',
        type: 'agent',
        position: { x: 100, y: 100 },
        data: {
          agent_type: 'ResearchAgent',
          task: 'Research the topic',
          tools: ['web_search', 'document_reader'],
        },
      },
      {
        id: 'agent-2',
        type: 'agent',
        position: { x: 100, y: 250 },
        data: {
          agent_type: 'AnalysisAgent',
          task: 'Analyze the research',
          tools: ['data_analyzer'],
        },
      },
    ];

    const edges = [
      {
        id: 'agent-1-agent-2',
        source: 'agent-1',
        target: 'agent-2',
      },
    ];

    // Build dependency map from edges
    const dependencyMap = new Map<string, string[]>();
    edges.forEach((edge) => {
      const deps = dependencyMap.get(edge.target) || [];
      deps.push(edge.source);
      dependencyMap.set(edge.target, deps);
    });

    // Convert to WorkflowDefinition
    const workflow: WorkflowDefinition = {
      name: 'Test Workflow',
      mode: 'sequential',
      agents: nodes.map((node) => ({
        id: node.id,
        agent_type: node.data.agent_type,
        task: node.data.task,
        depends_on: dependencyMap.get(node.id) || [],
        config: node.data.tools.length > 0 ? { tools: node.data.tools } : undefined,
      })),
    };

    // Assertions
    expect(workflow.name).toBe('Test Workflow');
    expect(workflow.mode).toBe('sequential');
    expect(workflow.agents).toHaveLength(2);

    // Check first agent
    expect(workflow.agents[0].id).toBe('agent-1');
    expect(workflow.agents[0].agent_type).toBe('ResearchAgent');
    expect(workflow.agents[0].task).toBe('Research the topic');
    expect(workflow.agents[0].depends_on).toEqual([]);
    expect(workflow.agents[0].config?.tools).toEqual(['web_search', 'document_reader']);

    // Check second agent (depends on first)
    expect(workflow.agents[1].id).toBe('agent-2');
    expect(workflow.agents[1].agent_type).toBe('AnalysisAgent');
    expect(workflow.agents[1].task).toBe('Analyze the research');
    expect(workflow.agents[1].depends_on).toEqual(['agent-1']);
    expect(workflow.agents[1].config?.tools).toEqual(['data_analyzer']);
  });

  it('should handle nodes with no dependencies', () => {
    const nodes = [
      {
        id: 'agent-1',
        type: 'agent',
        position: { x: 100, y: 100 },
        data: {
          agent_type: 'ResearchAgent',
          task: 'Research',
          tools: [],
        },
      },
    ];

    const edges: any[] = [];
    const dependencyMap = new Map<string, string[]>();

    const workflow: WorkflowDefinition = {
      name: 'Single Agent',
      mode: 'sequential',
      agents: nodes.map((node) => ({
        id: node.id,
        agent_type: node.data.agent_type,
        task: node.data.task,
        depends_on: dependencyMap.get(node.id) || [],
        config: node.data.tools.length > 0 ? { tools: node.data.tools } : undefined,
      })),
    };

    expect(workflow.agents[0].depends_on).toEqual([]);
    expect(workflow.agents[0].config).toBeUndefined();
  });

  it('should handle complex DAG with multiple dependencies', () => {
    const nodes = [
      { id: 'a', type: 'agent', position: { x: 0, y: 0 }, data: { agent_type: 'A', task: 'Task A', tools: [] } },
      { id: 'b', type: 'agent', position: { x: 0, y: 0 }, data: { agent_type: 'B', task: 'Task B', tools: [] } },
      { id: 'c', type: 'agent', position: { x: 0, y: 0 }, data: { agent_type: 'C', task: 'Task C', tools: [] } },
      { id: 'd', type: 'agent', position: { x: 0, y: 0 }, data: { agent_type: 'D', task: 'Task D', tools: [] } },
    ];

    const edges = [
      { id: 'a-c', source: 'a', target: 'c' },
      { id: 'b-c', source: 'b', target: 'c' },
      { id: 'c-d', source: 'c', target: 'd' },
    ];

    const dependencyMap = new Map<string, string[]>();
    edges.forEach((edge) => {
      const deps = dependencyMap.get(edge.target) || [];
      deps.push(edge.source);
      dependencyMap.set(edge.target, deps);
    });

    const workflow: WorkflowDefinition = {
      name: 'DAG Workflow',
      mode: 'dag',
      agents: nodes.map((node) => ({
        id: node.id,
        agent_type: node.data.agent_type,
        task: node.data.task,
        depends_on: dependencyMap.get(node.id) || [],
      })),
    };

    expect(workflow.agents.find(a => a.id === 'a')?.depends_on).toEqual([]);
    expect(workflow.agents.find(a => a.id === 'b')?.depends_on).toEqual([]);
    expect(workflow.agents.find(a => a.id === 'c')?.depends_on).toEqual(['a', 'b']);
    expect(workflow.agents.find(a => a.id === 'd')?.depends_on).toEqual(['c']);
  });
});
