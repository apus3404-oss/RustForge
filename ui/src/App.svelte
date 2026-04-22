<script lang="ts">
  import WorkflowBuilder from './components/WorkflowBuilder.svelte';
  import type { WorkflowDefinition } from './lib/types';

  function handleSave(workflow: WorkflowDefinition) {
    console.log('Workflow saved:', workflow);
    alert(`Workflow "${workflow.name}" saved with ${workflow.agents.length} agents!`);
  }
</script>

<WorkflowBuilder onSave={handleSave} />
