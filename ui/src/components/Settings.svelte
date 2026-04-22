<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { SystemConfig } from '$lib/types';

  let config = $state<SystemConfig | null>(null);
  let saving = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      config = await api.getConfig();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load settings';
    }
  });

  async function saveConfig() {
    if (!config) return;

    saving = true;
    error = null;

    try {
      await api.updateConfig(config);
      alert('Settings saved successfully');
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to save settings';
    } finally {
      saving = false;
    }
  }
</script>

<div class="settings-container">
  {#if error}
    <div class="error-banner">
      {error}
    </div>
  {/if}

  {#if config}
    <div class="settings">
      <h2>System Configuration</h2>

      <section class="settings-section">
        <h3>LLM Providers</h3>
        <div class="form-group">
          <label for="default-provider">Default Provider</label>
          <input
            id="default-provider"
            type="text"
            bind:value={config.llm.default_provider}
            placeholder="anthropic"
          />
        </div>
        <div class="form-group">
          <label for="fallback-enabled">
            <input
              id="fallback-enabled"
              type="checkbox"
              bind:checked={config.llm.fallback_enabled}
            />
            Enable Fallback
          </label>
        </div>
      </section>

      <section class="settings-section">
        <h3>Execution</h3>
        <div class="form-group">
          <label for="max-parallel">Max Parallel Agents</label>
          <input
            id="max-parallel"
            type="number"
            bind:value={config.execution.max_parallel_agents}
            min="1"
            max="16"
          />
        </div>
        <div class="form-group">
          <label for="default-timeout">Default Timeout (seconds)</label>
          <input
            id="default-timeout"
            type="number"
            bind:value={config.execution.default_timeout}
            min="30"
            max="3600"
          />
        </div>
      </section>

      <section class="settings-section">
        <h3>Permissions</h3>
        <div class="form-group">
          <label for="default-policy">Default Policy</label>
          <select id="default-policy" bind:value={config.permissions.default_policy}>
            <option value="allow">Allow</option>
            <option value="deny">Deny</option>
            <option value="prompt">Prompt</option>
          </select>
        </div>
      </section>

      <div class="actions">
        <button class="save-button" onclick={saveConfig} disabled={saving}>
          {saving ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </div>
  {:else if !error}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading settings...</p>
    </div>
  {/if}
</div>

<style>
  .settings-container {
    min-height: 100vh;
    background: #0a1628;
    color: #e2e8f0;
    font-family: 'JetBrains Mono', monospace;
    padding: 2rem;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid #ef4444;
    color: #fca5a5;
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 2rem;
  }

  .settings {
    max-width: 800px;
    margin: 0 auto;
  }

  .settings h2 {
    font-size: 24px;
    font-weight: 600;
    color: #00d9ff;
    margin-bottom: 2rem;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .settings-section {
    background: #0f1f35;
    border: 1px solid #1e3a5f;
    border-radius: 4px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .settings-section h3 {
    font-size: 14px;
    font-weight: 600;
    color: #00d9ff;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 1rem;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-group:last-child {
    margin-bottom: 0;
  }

  .form-group label {
    display: block;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #a0aec0;
    margin-bottom: 0.5rem;
  }

  .form-group label:has(input[type='checkbox']) {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    text-transform: none;
    font-size: 13px;
  }

  input[type='text'],
  input[type='number'],
  select {
    width: 100%;
    padding: 0.75rem;
    background: #0a1628;
    border: 1px solid #2d4a6f;
    border-radius: 4px;
    color: #e2e8f0;
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    transition: border-color 0.2s;
  }

  input[type='text']:focus,
  input[type='number']:focus,
  select:focus {
    outline: none;
    border-color: #00d9ff;
  }

  input[type='checkbox'] {
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: #00d9ff;
  }

  .actions {
    margin-top: 2rem;
    display: flex;
    justify-content: flex-end;
  }

  .save-button {
    padding: 0.75rem 2rem;
    background: #00d9ff;
    color: #0a1628;
    border: none;
    border-radius: 4px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .save-button:hover:not(:disabled) {
    background: #00ffff;
    transform: translateY(-1px);
  }

  .save-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    gap: 1rem;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid #1e3a5f;
    border-top-color: #00d9ff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .loading p {
    color: #a0aec0;
    font-size: 13px;
  }
</style>
