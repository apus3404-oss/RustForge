<script lang="ts">
  import Navigation from './components/Navigation.svelte';
  import Router from './components/Router.svelte';
</script>

<div class="app">
  <Navigation />
  <main>
    <Router />
  </main>
</div>

<style>
  .app {
    min-height: 100vh;
    background: #0d0d0d;
    color: #fff;
  }

  main {
    min-height: calc(100vh - 60px);
  }
</style>
