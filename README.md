# abort-on-drop
This crate provides a wrapper type of Tokio's JoinHandle: `ChildTask`, which aborts the task when it's dropped. `ChildTask` can still be awaited to join the child-task, and abort-on-drop will still trigger while it is being awaited.

For example, if task A spawned task B but is doing something else, and task B is waiting for task C to join, aborting A will also abort both B and C.

Basic usage:
```rust
use abort_on_drop::ChildTask;
{
  // Wrap a normal tokio JoinHandle in a ChidTask to make it abort when dropped
  let task_a = ChildTask::from(tokio::spawn(async { }));
  let task_b = tokio::spawn(async { });
} // task_a get dropped here
// task_a has now been aborted, but task_b is still running in the background
```