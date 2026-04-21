use rustforge::engine::{
    AgentEvent, EventBus, ExecutionContext, SequentialExecutor, WorkflowParser,
};
use std::sync::Arc;
use tempfile::TempDir;
use std::fs;

#[tokio::test]
async fn test_end_to_end_workflow_execution() {
    // Create temporary directory for test workflow
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("test-workflow.yaml");

    // Create test workflow YAML with variable interpolation
    let workflow_yaml = r#"
name: test-workflow
mode: sequential
agents:
  - id: agent1
    type: test
    task: "Process input"
  - id: agent2
    type: test
    task: "Use {agent1.output}"
"#;

    fs::write(&workflow_path, workflow_yaml).unwrap();

    // Step 1: Parse workflow with WorkflowParser
    let workflow = WorkflowParser::parse_file(&workflow_path).unwrap();
    assert_eq!(workflow.name, "test-workflow");
    assert_eq!(workflow.agents.len(), 2);
    assert_eq!(workflow.agents[0].id, "agent1");
    assert_eq!(workflow.agents[1].id, "agent2");

    // Step 2: Create EventBus and subscribe to events
    let event_bus = Arc::new(EventBus::new());
    let mut subscriber = event_bus.subscribe();

    // Step 3: Create SequentialExecutor
    let executor = SequentialExecutor::new(event_bus.clone());

    // Step 4: Execute workflow in separate task so we can receive events
    let mut context = ExecutionContext::new("test-workflow".to_string());
    let workflow_clone = workflow.clone();

    let exec_handle = tokio::spawn(async move {
        executor.execute(&workflow_clone, &mut context).await
    });

    // Step 5: Verify events are published
    // Expect: TaskStarted(agent1), TaskCompleted(agent1), TaskStarted(agent2), TaskCompleted(agent2)

    // Event 1: agent1 TaskStarted
    let event1 = subscriber.recv().await.unwrap();
    match event1 {
        AgentEvent::TaskStarted { agent_id, task } => {
            assert_eq!(agent_id, "agent1");
            assert_eq!(task, "Process input");
        }
        _ => panic!("Expected TaskStarted event for agent1"),
    }

    // Event 2: agent1 TaskCompleted
    let event2 = subscriber.recv().await.unwrap();
    match event2 {
        AgentEvent::TaskCompleted { agent_id, output } => {
            assert_eq!(agent_id, "agent1");
            assert!(output.contains("Stub output from agent1"));
        }
        _ => panic!("Expected TaskCompleted event for agent1"),
    }

    // Event 3: agent2 TaskStarted (with interpolated task)
    let event3 = subscriber.recv().await.unwrap();
    match event3 {
        AgentEvent::TaskStarted { agent_id, task } => {
            assert_eq!(agent_id, "agent2");
            // Task should have interpolated variable from agent1's output
            assert!(task.contains("Stub output from agent1"));
        }
        _ => panic!("Expected TaskStarted event for agent2"),
    }

    // Event 4: agent2 TaskCompleted
    let event4 = subscriber.recv().await.unwrap();
    match event4 {
        AgentEvent::TaskCompleted { agent_id, output } => {
            assert_eq!(agent_id, "agent2");
            assert!(output.contains("Stub output from agent2"));
        }
        _ => panic!("Expected TaskCompleted event for agent2"),
    }

    // Step 6: Wait for execution to complete and check final output
    let result = exec_handle.await.unwrap();
    assert!(result.is_ok());

    let final_output = result.unwrap();

    // Verify final output structure
    assert_eq!(final_output["workflow"], "test-workflow");
    assert!(final_output["execution_id"].is_string());

    // Verify both agents are in the output
    let agents = final_output["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 2);

    // Verify agent1 output
    assert_eq!(agents[0]["agent_id"], "agent1");
    assert_eq!(agents[0]["status"], "completed");
    assert!(agents[0]["result"].as_str().unwrap().contains("agent1"));

    // Verify agent2 output
    assert_eq!(agents[1]["agent_id"], "agent2");
    assert_eq!(agents[1]["status"], "completed");
    assert!(agents[1]["result"].as_str().unwrap().contains("agent2"));
}
