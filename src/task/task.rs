use super::TaskContext;

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
}

// Used to represent the status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Uninit,
    Ready,
    Running,
    Exited,
}