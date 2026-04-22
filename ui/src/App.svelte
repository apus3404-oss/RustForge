<script lang="ts">
  let count = $state(0);
</script>

<div class="min-h-screen bg-navy-950 bg-grid-pattern bg-grid">
  <div class="relative z-10">
    <!-- Header -->
    <header class="border-b border-cyan-400/20 bg-navy-900/50 backdrop-blur-sm">
      <div class="container mx-auto px-6 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 border-2 border-cyan-400 flex items-center justify-center">
              <span class="text-cyan-400 font-mono text-sm font-bold">RF</span>
            </div>
            <h1 class="text-xl font-mono font-semibold tracking-tight">
              RustForge
            </h1>
          </div>
          <div class="flex items-center gap-2 text-xs font-mono text-cyan-400/60">
            <span>v0.1.0</span>
            <span class="text-cyan-400/30">|</span>
            <span>PHASE 5</span>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="container mx-auto px-6 py-12">
      <div class="max-w-4xl">
        <!-- Technical Label -->
        <div class="inline-flex items-center gap-2 mb-6 text-xs font-mono text-cyan-400/60 border border-cyan-400/20 px-3 py-1.5">
          <span class="w-1.5 h-1.5 bg-cyan-400 rounded-full animate-pulse"></span>
          <span>SYSTEM.INIT</span>
        </div>

        <!-- Title Section -->
        <div class="space-y-4 mb-12">
          <h2 class="text-4xl font-mono font-bold tracking-tight">
            Agentic Workflow Framework
          </h2>
          <p class="text-lg text-white/70 font-sans max-w-2xl">
            Visual workflow builder and execution monitor for Rust-based agent orchestration.
          </p>
        </div>

        <!-- Status Grid -->
        <div class="grid grid-cols-3 gap-4 mb-12">
          <div class="border border-cyan-400/20 bg-navy-900/30 p-4">
            <div class="text-xs font-mono text-cyan-400/60 mb-2">API.STATUS</div>
            <div class="text-2xl font-mono font-bold text-cyan-400">READY</div>
          </div>
          <div class="border border-cyan-400/20 bg-navy-900/30 p-4">
            <div class="text-xs font-mono text-cyan-400/60 mb-2">UI.VERSION</div>
            <div class="text-2xl font-mono font-bold">0.1.0</div>
          </div>
          <div class="border border-cyan-400/20 bg-navy-900/30 p-4">
            <div class="text-xs font-mono text-cyan-400/60 mb-2">COUNTER</div>
            <div class="text-2xl font-mono font-bold">{count}</div>
          </div>
        </div>

        <!-- Interactive Demo -->
        <div class="border border-cyan-400/20 bg-navy-900/30 p-6">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-sm font-mono text-cyan-400">INTERACTIVE.TEST</h3>
            <span class="text-xs font-mono text-white/40">Click to increment</span>
          </div>
          <button
            onclick={() => count++}
            class="w-full border-2 border-cyan-400 bg-cyan-400/10 hover:bg-cyan-400/20 transition-colors px-6 py-3 font-mono font-semibold text-cyan-400"
          >
            INCREMENT [{count}]
          </button>
        </div>
      </div>
    </main>

    <!-- Footer -->
    <footer class="border-t border-cyan-400/20 bg-navy-900/50 backdrop-blur-sm mt-24">
      <div class="container mx-auto px-6 py-4">
        <div class="flex items-center justify-between text-xs font-mono text-white/40">
          <span>RustForge © 2026</span>
          <span>Technical Blueprint Aesthetic</span>
        </div>
      </div>
    </footer>
  </div>
</div>
