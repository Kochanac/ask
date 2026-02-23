Overview of the system

Two main components of the system

- Agent
- UI

The UI should be decoupled from the agent to allow for different implementations.


The agent works, generates some progress, starts some subagents, runs some tools. All of this produces events (tool calls, subagents started, compactions etc)

Events: 
- New text from the model
- Tool calls
- User messages (UI -> agent)
- Requests for approvals of tool calls
- Approves the tool calls when needed


