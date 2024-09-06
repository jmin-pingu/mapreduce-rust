# Todo List
## Priority to Learn + Implement
- [ ] Documentation documentation documentation...
- [ ] Need to understand RPC (specifically tarpc) better. I.e. concurrent client requests and so on...
- [ ] Need to use lifetimes... This is the whole point of rust. Change String -> &str when possible. 
- [ ] On a similar note, logging logging logging...
- [ ] The API for `config.toml`. Need to think about what customizability + features that I want to expose to users of MapReduce. 
- [ ] Need to figure out configuration of sockets/ports with tarpc lib in Rust. This is just figuring out networking.
  - [ ] Worker + Coordinator orchestration
- [ ] Create basic unit tests and integration tests with/without network failures. 
  - Breaking the problem down...
    - [ ] Data structure functionality
    - [ ] Correctness of serial MapReduce (testing `do_map()`, `do_reduce()`, `prepare_for_reduce()`)
    - [ ] Correctness of distributed MapReduce
      - [ ] Correctness when running locally
      - [ ] Correctness when running over network (i.e. legit distributed. This can be done with AWS)
        - [ ] Correctness with controlled network failures
- [X] Need to figure out how to get the mapf and reducef function api to work once we compile a MapReduce program.
  - The solution was dynamic loading. Regardless, compilation is confusing so I need to learn more about it.

## Additional Interesting Functionality
- [ ] A repeatable, systematic way of benchmarking MapReduce along with nice visualizations + statistics
- [ ] Toggle-able functionality for reduce tasks that do not need to wait for all the map tasks to complete (i.e. word count).
- [ ] Toggle-able functionality, extending the MapReduce paper to more "modern" iterations like Spark, etc.
