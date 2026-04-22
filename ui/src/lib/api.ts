import type {
  WorkflowSummary,
  WorkflowDefinition,
  WorkflowResponse,
  ExecutionResponse,
  ExecutionDetails,
  ExecutionSummary,
} from './types';

const API_BASE = '/api';

export class ApiClient {
  private async fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
    const response = await fetch(url, options);

    if (!response.ok) {
      const errorText = await response.text();
      let errorMessage = `HTTP ${response.status}: ${response.statusText}`;

      try {
        const errorJson = JSON.parse(errorText);
        if (errorJson.error) {
          errorMessage = errorJson.error;
          if (errorJson.details) {
            errorMessage += ` - ${errorJson.details}`;
          }
        }
      } catch {
        if (errorText) {
          errorMessage += ` - ${errorText}`;
        }
      }

      throw new Error(errorMessage);
    }

    return response.json();
  }

  async listWorkflows(): Promise<WorkflowSummary[]> {
    return this.fetchJson<WorkflowSummary[]>(`${API_BASE}/workflows`);
  }

  async getWorkflow(id: string): Promise<WorkflowDefinition> {
    return this.fetchJson<WorkflowDefinition>(`${API_BASE}/workflows/${id}`);
  }

  async createWorkflow(definition: WorkflowDefinition): Promise<WorkflowResponse> {
    return this.fetchJson<WorkflowResponse>(`${API_BASE}/workflows`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(definition),
    });
  }

  async deleteWorkflow(id: string): Promise<void> {
    await this.fetchJson<void>(`${API_BASE}/workflows/${id}`, {
      method: 'DELETE',
    });
  }

  async executeWorkflow(
    workflowId: string,
    inputs: Record<string, any>
  ): Promise<ExecutionResponse> {
    return this.fetchJson<ExecutionResponse>(
      `${API_BASE}/workflows/${workflowId}/execute`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(inputs),
      }
    );
  }

  async getExecution(executionId: string): Promise<ExecutionDetails> {
    return this.fetchJson<ExecutionDetails>(`${API_BASE}/executions/${executionId}`);
  }

  async listExecutions(): Promise<ExecutionSummary[]> {
    return this.fetchJson<ExecutionSummary[]>(`${API_BASE}/executions`);
  }

  async pauseExecution(executionId: string): Promise<void> {
    await this.fetchJson<void>(`${API_BASE}/executions/${executionId}/pause`, {
      method: 'POST',
    });
  }

  async resumeExecution(executionId: string): Promise<void> {
    await this.fetchJson<void>(`${API_BASE}/executions/${executionId}/resume`, {
      method: 'POST',
    });
  }

  async cancelExecution(executionId: string): Promise<void> {
    await this.fetchJson<void>(`${API_BASE}/executions/${executionId}/cancel`, {
      method: 'POST',
    });
  }
}

export const api = new ApiClient();
