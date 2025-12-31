# r_agent

`r_agent` is an agent framework for building AI workflows with rust.


## Features To Implement

- [x] LLM support...
- [ ] Different LLM backends (OpenAI, Azure, local models, etc.)
- [ ] Tool Calling: Equip your agents with the ability to utilize external tools.
- [x] Memory - Sliding window + auto-summarization strategies for context management
- [x] ReAct Loop - Standard think → act → observe reasoning cycle
- [ ] DAG Pipeline - Directed Acyclic Graph workflow orchestration with parallel execution
- [ ] Debug Friendly - Colored logging, token usage and cost tracking
- [ ] Minimal Footprint - Only ~809 lines of core code, no bloat, easy to read/modify/extend