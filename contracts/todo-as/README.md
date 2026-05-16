# todo-as

Example Qorvum smart contract written in AssemblyScript, demonstrating the Schema-based ORM API from `qorvum-contract-sdk`.

## Structure

```
assembly/
  index.ts          — dispatch entry point
  schema.ts         — TodoSchema & UserSchema definitions
  todo.service.ts   — TodoService class (all business logic)
```

## Contract functions

| Function | Args | Description |
|---|---|---|
| `create_todo` | `id`, `title`, `assignee_id`?, `due_date`? | Create a new todo |
| `get_todo` | `id` | Fetch a todo + its assignee user |
| `complete_todo` | `id` | Mark a todo as DONE |
| `delete_todo` | `id`, `reason`? | Soft-delete a todo |
| `list_todos` | `status`?, `limit`? | Query todos with optional filter |
| `assign_todo` | `id`, `assignee_id` | Assign todo to a user *(MANAGER role required)* |

## Build

```bash
npm install
npm run asbuild          # → build/release.wasm
```

For a debug build with readable WAT output:

```bash
npm run asbuild:debug    # → build/debug.wasm + build/debug.wat
```

## Deploy

```ts
import * as fs from "fs";

const wasm = fs.readFileSync("build/release.wasm");
await executor.deploy_wasm("todo-contract", wasm);
```

## Usage examples

### Create a todo

```json
{
  "fn": "create_todo",
  "args": { "id": "todo-1", "title": "Buy milk", "due_date": "2026-06-01" }
}
```

### List pending todos

```json
{
  "fn": "list_todos",
  "args": { "status": "PENDING", "limit": 20 }
}
```

### Complete a todo

```json
{
  "fn": "complete_todo",
  "args": { "id": "todo-1" }
}
```

### Assign a todo (requires MANAGER role)

```json
{
  "fn": "assign_todo",
  "args": { "id": "todo-1", "assignee_id": "user-99" }
}
```

## Schema

```ts
// schema.ts
const TodoSchema = new Schema("todos")
  .text("title")
  .text("status")
  .bool("done")
  .text("assignee_id", false)   // optional FK → users
  .text("due_date",    false)
  .belongsTo("assignee", UserSchema, "assignee_id");
```

Fields marked `false` are optional. All others are required and validated by `QvModel.validate()` before insert.

## SDK

This contract uses [`qorvum-contract-sdk`](../../../qorvum-contract-sdk) which provides:

- `Schema` — declare collection name, columns, and relationships
- `QvModel` — ORM-style CRUD + query builder + relationship traversal
- `Filter` / `Sort` — chainable query helpers
- `getField` / `getIntField` — flat JSON arg parsers for `dispatch()`
