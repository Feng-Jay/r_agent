# r_agent


<p align="center">
<a href="https://img.shields.io/badge/PRs-welcome-blue"><img alt="Static Badge" src="https://img.shields.io/badge/PRs-welcome-blue"></a>
<a href="https://github.com/Feng-Jay/r_agent/stargazers"><img alt="GitHub Repo stars" src="https://img.shields.io/github/stars/Feng-Jay/r_agent"> </a>
</p>

`r_agent` is an agent framework for building AI workflows with rust.

> [!NOTE]
> This project is currently in development. The exposed APIs may change in future releases. The inner implementation may directly use `unwrapped` calls, which may lead to panics and will be improved in future releases. Please use with caution.

## Features To Implement

- [x] OpenAI (OpenAI Compatible) backends support...
- [ ] More LLM backends (Azure, local models, etc.)
- [x] Tool Calling: Equip agents with the ability to utilize self-designed tools
- [x] Memory - Sliding window + auto-summarization strategies for context management
- [x] ReAct Loop - Standard think → act → observe reasoning cycle
- [ ] DAG Pipeline - Directed Acyclic Graph workflow orchestration with parallel execution
- [ ] Debug Friendly - Utilize rust's error handling for transparent debugging, and logging for traceability, with request content and token usage tracking.


## Contributing

Contributions are welcome! Please open issues and submit pull requests for bug fixes and new features!

## Thanks

Thanks to the following open source projects:

- [EasyAgnet](https://github.com/SNHuan/EasyAgent)
- [llm](https://github.com/graniet/llm)
- [tiktoken-rs](https://github.com/zurawiki/tiktoken-rs)
