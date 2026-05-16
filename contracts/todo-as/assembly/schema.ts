import { Schema } from "qorvum-contract-sdk/assembly/index";

export const UserSchema = new Schema("users");

export const TodoSchema = new Schema("todos")
  .text("title")
  .text("status")
  .bool("done")
  .text("assignee_id", false)
  .text("due_date",    false)
  .belongsTo("assignee", UserSchema, "assignee_id");
