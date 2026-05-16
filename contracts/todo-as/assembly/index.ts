/**
 * Todo Contract — entry point
 *
 * Build:  npm run asbuild
 * Output: build/release.wasm
 *
 * Deploy to Qorvum node:
 *   const wasm = fs.readFileSync("build/release.wasm");
 *   executor.deploy_wasm("todo-contract", wasm);
 */

import { Context, QvModel, readString, qv_err } from "qorvum-contract-sdk/assembly/index";
export { alloc } from "qorvum-contract-sdk/assembly/index";

import { TodoSchema, UserSchema } from "./schema";
import { TodoService } from "./todo.service";

export function dispatch(
  fn_ptr:   i32, fn_len:   i32,
  args_ptr: i32, args_len: i32,
): i64 {
  const name = readString(fn_ptr,   fn_len);
  const args = readString(args_ptr, args_len);

  const ctx     = new Context();
  const service = new TodoService(
    new QvModel(ctx, TodoSchema),
    new QvModel(ctx, UserSchema),
  );

  if (name == "create_todo")   return service.create(args);
  if (name == "get_todo")      return service.get(args);
  if (name == "complete_todo") return service.complete(args);
  if (name == "delete_todo")   return service.delete(args);
  if (name == "list_todos")    return service.list(args);
  if (name == "assign_todo")   return service.assign(args);

  return qv_err("Unknown function: " + name);
}
