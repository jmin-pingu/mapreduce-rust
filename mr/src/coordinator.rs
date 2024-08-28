struct Coordinator {
    task_manager: crate::ds::TaskManager, // NOTE: what if we have a daemon looping through the
                                          // task_manager looking for idle map jobs?
    nreduce_remaining: usize, // NOTE: when num_reduce == 0
}

