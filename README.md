# abort-on-drop
This crate provides a wrapper type of Tokio's JoinHandle: `ChildTask`, which aborts the task when it's dropped. `ChildTask` can still be awaited to join the child-task, and abort-on-drop will still trigger while it is being awaited.

For example, if task A spawned task B but is doing something else, and task B is waiting for task C to join, aborting A will also abort both B and C.
