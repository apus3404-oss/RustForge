<script lang="ts">
  import { navigate } from '../lib/router';
</script>

<div class="home">
  <div class="hero">
    <h1>RustForge Workflow Engine</h1>
    <p class="subtitle">Build and execute multi-agent workflows</p>
  </div>

  <div class="quick-actions">
    <div class="action-card">
      <h2>Create Workflow</h2>
      <p>Design a new multi-agent workflow with our visual builder</p>
      <button class="primary-button" on:click={() => navigate('/workflows')}>
        New Workflow
      </button>
    </div>

    <div class="action-card">
      <h2>View Executions</h2>
      <p>Monitor running workflows and review execution history</p>
      <button class="secondary-button" on:click={() => navigate('/executions')}>
        View Executions
      </button>
    </div>

    <div class="action-card">
      <h2>Settings</h2>
      <p>Configure API endpoints and system preferences</p>
      <button class="secondary-button" on:click={() => navigate('/settings')}>
        Open Settings
      </button>
    </div>
  </div>
</div>

<style>
  .home {
    padding: 3rem 2rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .hero {
    text-align: center;
    margin-bottom: 4rem;
  }

  .hero h1 {
    font-size: 2.5rem;
    margin-bottom: 1rem;
    color: #fff;
  }

  .subtitle {
    font-size: 1.25rem;
    color: #999;
  }

  .quick-actions {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
  }

  .action-card {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .action-card h2 {
    font-size: 1.5rem;
    color: #00d4ff;
    margin: 0;
  }

  .action-card p {
    color: #999;
    flex: 1;
    margin: 0;
  }

  .primary-button {
    padding: 0.75rem 1.5rem;
    background: #00d4ff;
    color: #000;
    border: none;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .primary-button:hover {
    background: #00b8e6;
    transform: translateY(-1px);
  }

  .secondary-button {
    padding: 0.75rem 1.5rem;
    background: transparent;
    color: #00d4ff;
    border: 1px solid #00d4ff;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .secondary-button:hover {
    background: rgba(0, 212, 255, 0.1);
    transform: translateY(-1px);
  }
</style>
