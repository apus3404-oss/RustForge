// Agent configuration for workflow definition
export interface AgentConfig {
  id: string;
  agent_type: string;
  task: string;
  depends_on: string[];
  config?: Record<string, any>;
}

// Workflow definition structure
export interface WorkflowDefinition {
  name: string;
  mode: 'sequential' | 'parallel' | 'dag';
  agents: AgentConfig[];
  inputs?: Record<string, any>;
}

// Workflow summary for list view
export interface WorkflowSummary {
  id: string;
  name: string;
  mode: string;
  agent_count: number;
  created_at: string;
}

// Response when creating a workflow
export interface WorkflowResponse {
  id: string;
  name: string;
  created_at: string;
}

// Execution status types
export type ExecutionStatus = 'pending' | 'running' | 'completed' | 'failed' | 'paused' | 'cancelled';

// Response when starting an execution
export interface ExecutionResponse {
  execution_id: string;
  status: ExecutionStatus;
  started_at: string;
}

// Detailed execution information
export interface ExecutionDetails {
  id: string;
  workflow_id: string;
  status: ExecutionStatus;
  started_at: string;
  completed_at?: string;
  outputs?: Record<string, any>;
  error?: string;
}

// Execution summary for list view
export interface ExecutionSummary {
  id: string;
  workflow_id: string;
  status: ExecutionStatus;
  started_at: string;
  completed_at?: string;
}

// Generic API error response
export interface ApiError {
  error: string;
  details?: string;
}

// System configuration
export interface SystemConfig {
  llm: {
    default_provider: string;
    fallback_enabled: boolean;
  };
  execution: {
    max_parallel_agents: number;
    default_timeout: number;
  };
  permissions: {
    default_policy: 'allow' | 'deny' | 'prompt';
  };
}
