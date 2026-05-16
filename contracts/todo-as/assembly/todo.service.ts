import {
  QvModel, Fields, Filter, Sort,
  getField, getIntField, qv_ok, qv_err, escStr,
} from "qorvum-contract-sdk/assembly/index";

export class TodoService {
  private todos: QvModel;
  private users: QvModel;

  constructor(todos: QvModel, users: QvModel) {
    this.todos = todos;
    this.users = users;
  }

  // ── create_todo ─────────────────────────────────────────────────────────────
  // Args: { "id", "title", "assignee_id"?, "due_date"? }

  create(args: string): i64 {
    const id         = getField(args, "id");
    const title      = getField(args, "title");
    const assigneeId = getField(args, "assignee_id");
    const dueDate    = getField(args, "due_date");

    if (id.length    == 0) return qv_err("id is required");
    if (title.length == 0) return qv_err("title is required");

    const fields = new Fields()
      .text("title",  title)
      .text("status", "PENDING")
      .bool("done",   false);

    if (assigneeId.length > 0) fields.text("assignee_id", assigneeId);
    if (dueDate.length    > 0) fields.text("due_date",    dueDate);

    const validErr = this.todos.validate(fields);
    if (validErr.length > 0) return qv_err(validErr);

    const record = this.todos.create(id, fields);
    if (this.todos.hasError()) return qv_err(this.todos.lastError());

    this.todos.emit("TODO_CREATED",
      '{"id":' + escStr(id) + ',"title":' + escStr(title) + "}");
    return qv_ok(record);
  }

  // ── get_todo ────────────────────────────────────────────────────────────────
  // Args: { "id" }
  // Returns the todo + its assignee user (if set).

  get(args: string): i64 {
    const id = getField(args, "id");
    if (id.length == 0) return qv_err("id is required");

    const todo = this.todos.findById(id);
    if (this.todos.hasError()) return qv_err("Todo not found: " + id);

    const assigneeId = getField(todo, "assignee_id");
    if (assigneeId.length > 0) {
      const user = this.todos.rel("assignee", this.users).find(assigneeId);
      if (!this.users.hasError()) {
        return qv_ok('{"todo":' + todo + ',"assignee":' + user + "}");
      }
    }

    return qv_ok('{"todo":' + todo + ',"assignee":null}');
  }

  // ── complete_todo ───────────────────────────────────────────────────────────
  // Args: { "id" }

  complete(args: string): i64 {
    const id = getField(args, "id");
    if (id.length == 0) return qv_err("id is required");

    this.todos.findById(id);
    if (this.todos.hasError()) return qv_err("Todo not found: " + id);

    const record = this.todos.patch(id, new Fields()
      .bool("done",   true)
      .text("status", "DONE"),
    );
    if (this.todos.hasError()) return qv_err(this.todos.lastError());

    this.todos.emit("TODO_COMPLETED", '{"id":' + escStr(id) + "}");
    return qv_ok(record);
  }

  // ── delete_todo ─────────────────────────────────────────────────────────────
  // Args: { "id", "reason"? }

  delete(args: string): i64 {
    const id     = getField(args, "id");
    const reason = getField(args, "reason");
    if (id.length == 0) return qv_err("id is required");

    const result = this.todos.remove(id, reason);
    if (this.todos.hasError()) return qv_err(this.todos.lastError());

    this.todos.emit("TODO_DELETED", '{"id":' + escStr(id) + "}");
    return qv_ok(result);
  }

  // ── list_todos ──────────────────────────────────────────────────────────────
  // Args: { "status"?: "PENDING"|"DONE", "limit"?: 50 }

  list(args: string): i64 {
    const status = getField(args, "status");
    const limit  = getIntField(args, "limit");

    const filter: Filter | null = status.length > 0
      ? Filter.eq("status", status)
      : null;

    const result = this.todos.select()
      .where(filter)
      .orderBy(new Sort().asc("title"))
      .limit(limit > 0 ? limit as i32 : 50)
      .find();

    if (this.todos.hasError()) return qv_err(this.todos.lastError());
    return qv_ok(result);
  }

  // ── assign_todo ─────────────────────────────────────────────────────────────
  // Args: { "id", "assignee_id" }
  // Requires MANAGER role.

  assign(args: string): i64 {
    if (!this.todos.hasRole("MANAGER")) return qv_err("Requires MANAGER role");

    const id         = getField(args, "id");
    const assigneeId = getField(args, "assignee_id");
    if (id.length         == 0) return qv_err("id is required");
    if (assigneeId.length == 0) return qv_err("assignee_id is required");

    this.todos.findById(id);
    if (this.todos.hasError()) return qv_err("Todo not found: " + id);

    this.todos.rel("assignee", this.users).find(assigneeId);
    if (this.users.hasError()) return qv_err("User not found: " + assigneeId);

    const record = this.todos.patch(id, new Fields().text("assignee_id", assigneeId));
    if (this.todos.hasError()) return qv_err(this.todos.lastError());

    this.todos.emit("TODO_ASSIGNED",
      '{"id":' + escStr(id) + ',"assignee_id":' + escStr(assigneeId) + "}");
    return qv_ok(record);
  }
}
