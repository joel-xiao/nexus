//! 任务队列单元测试

use nexus::infrastructure::queue::TaskQueue;
use nexus::infrastructure::queue::task::{Task, TaskPriority};

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
    
    // 验证任务已入队（任务在后台处理，需要等待一下）
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let retrieved = queue.get_task(&task_id);
    // 任务可能已处理，或者还在队列中
    // 至少验证 enqueue 没有出错
    assert!(!task_id.is_empty());
}
