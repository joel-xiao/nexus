use nexus::infrastructure::queue::task::{Task, TaskPriority};
use nexus::infrastructure::queue::TaskQueue;

/// 测试任务队列
#[tokio::test]
async fn test_task_queue() {
    let queue = TaskQueue::new(None);

    let task = Task::new(
        "test_task".to_string(),
        serde_json::json!({"test": "data"}),
        TaskPriority::Normal,
    );

    let task_id = queue.enqueue(task).unwrap();
    assert!(!task_id.is_empty());

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let _retrieved = queue.get_task(&task_id);
    assert!(!task_id.is_empty());
}
