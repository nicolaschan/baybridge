use super::{sqlite_controller::SqliteController, tasks};

pub struct TaskController {
    controller: SqliteController,
}

impl TaskController {
    pub fn new(controller: SqliteController) -> TaskController {
        TaskController { controller }
    }

    pub async fn run_tasks(&self) -> anyhow::Result<()> {
        tasks::gc_expired::run(&self.controller).await?;
        Ok(())
    }
}
