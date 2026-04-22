<script lang="ts">
  import { currentPath, navigate } from '../lib/router';

  function isActive(path: string): boolean {
    return $currentPath === path || $currentPath.startsWith(path + '/');
  }

  function handleNavigate(path: string) {
    navigate(path);
  }
</script>

<nav class="nav">
  <div class="nav-brand">
    <button on:click={() => handleNavigate('/')} class="brand-button">
      RustForge
    </button>
  </div>

  <div class="nav-links">
    <button
      on:click={() => handleNavigate('/')}
      class="nav-link"
      class:active={$currentPath === '/'}
    >
      Home
    </button>
    <button
      on:click={() => handleNavigate('/workflows')}
      class="nav-link"
      class:active={isActive('/workflows')}
    >
      Workflows
    </button>
    <button
      on:click={() => handleNavigate('/executions')}
      class="nav-link"
      class:active={isActive('/executions')}
    >
      Executions
    </button>
    <button
      on:click={() => handleNavigate('/settings')}
      class="nav-link"
      class:active={isActive('/settings')}
    >
      Settings
    </button>
  </div>
</nav>

<style>
  .nav {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding: 1rem 1.5rem;
    background: #1a1a1a;
    border-bottom: 1px solid #333;
  }

  .nav-brand {
    font-weight: 600;
    font-size: 1.25rem;
    color: #00d4ff;
  }

  .brand-button {
    background: none;
    border: none;
    color: inherit;
    font: inherit;
    cursor: pointer;
    padding: 0;
  }

  .brand-button:hover {
    opacity: 0.8;
  }

  .nav-links {
    display: flex;
    gap: 0.5rem;
    flex: 1;
  }

  .nav-link {
    padding: 0.5rem 1rem;
    background: none;
    border: none;
    color: #999;
    cursor: pointer;
    font-size: 0.95rem;
    border-radius: 4px;
    transition: all 0.2s;
  }

  .nav-link:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.05);
  }

  .nav-link.active {
    color: #00d4ff;
    background: rgba(0, 212, 255, 0.1);
  }
</style>
