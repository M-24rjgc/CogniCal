use chrono::Utc;
use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::task::{TaskCreateInput, TaskUpdateInput};
use cognical_app_lib::services::task_service::TaskService;
use tempfile::tempdir;

#[test]
fn task_crud_flow() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("integration.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    let service = TaskService::new(pool.clone());

    // create
    let created = service
        .create_task(TaskCreateInput {
            title: "Integration Task".into(),
            status: Some("todo".into()),
            priority: Some("medium".into()),
            ..Default::default()
        })
        .expect("create task");

    assert!(!created.id.is_empty());
    assert_eq!(created.status, "todo");

    // list
    let tasks = service.list_tasks().expect("list tasks");
    assert!(!tasks.is_empty());

    // update
    let mut update = TaskUpdateInput::default();
    update.status = Some("done".into());
    update.completed_at = Some(Some(Utc::now().to_rfc3339()));

    let updated = service
        .update_task(&created.id, update)
        .expect("update task");
    assert_eq!(updated.status, "done");
    assert!(updated.completed_at.is_some());

    // delete
    service
        .delete_task(&created.id)
        .expect("delete task");

    let result = service.get_task(&created.id);
    assert!(result.is_err());
}
