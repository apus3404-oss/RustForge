<script lang="ts">
  import Settings from '../components/Settings.svelte';
</script>

<div class="settings-page">
  <div class="page-header">
    <h1>Settings</h1>
    <p>Configure API endpoints and system preferences</p>
  </div>

  <div class="settings-container">
    <Settings />
  </div>
</div>

<style>
  .settings-page {
    padding: 2rem;
    max-width: 800px;
    margin: 0 auto;
  }

  .page-header {
    margin-bottom: 2rem;
  }

  .page-header h1 {
    font-size: 2rem;
    color: #fff;
    margin-bottom: 0.5rem;
  }

  .page-header p {
    color: #999;
  }

  .settings-container {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 2rem;
  }
</style>
