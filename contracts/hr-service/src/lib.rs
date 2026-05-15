//! HR Service Contract — manages employee records.
//! 
//! Functions:
//!   hire_employee        — create new employee record
//!   get_employee         — fetch single employee
//!   update_salary        — patch salary fields
//!   transfer_department  — move employee to another dept
//!   terminate_employee   — soft-delete with status=TERMINATED
//!   restore_employee     — un-delete a terminated employee
//!   list_by_department   — query all active employees in a dept
//!   search_employees     — filter by salary range / position
//!   get_employee_history — full audit trail for an employee

use chain_sdk::ChainContext;
use chain_sdk::types::{FieldValue, Filter};
use serde::Deserialize;
use std::collections::HashMap;

const COLLECTION: &str = "employees";

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct HireInput {
    pub id:          String,
    pub name:        String,
    pub department:  String,
    pub position:    String,
    pub salary:      i64,
    pub join_date:   String,    // "YYYY-MM-DD"
    pub email:       String,
    pub phone:       Option<String>,
    pub manager_id:  Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSalaryInput {
    pub id:          String,
    pub department:  String,
    pub new_salary:  i64,
    pub reason:      String,
    pub effective:   String,
}

#[derive(Deserialize)]
pub struct TransferInput {
    pub id:          String,
    pub from_dept:   String,
    pub to_dept:     String,
    pub new_position:Option<String>,
    pub reason:      String,
}

#[derive(Deserialize)]
pub struct TerminateInput {
    pub id:          String,
    pub department:  String,
    pub reason:      String,
    pub exit_date:   String,
}

#[derive(Deserialize)]
pub struct SearchInput {
    pub department:  Option<String>,
    pub position:    Option<String>,
    pub salary_min:  Option<i64>,
    pub salary_max:  Option<i64>,
    pub limit:       Option<u32>,
    pub offset:      Option<u32>,
}

// ── Public dispatch function (called by executor) ────────────────────────────

pub fn dispatch(
    function_name: &str,
    args:          serde_json::Value,
    ctx:           &dyn ChainContext,
) -> Result<serde_json::Value, String> {
    match function_name {
        "hire_employee"        => hire(args, ctx),
        "get_employee"         => get(args, ctx),
        "update_salary"        => update_salary(args, ctx),
        "transfer_department"  => transfer(args, ctx),
        "terminate_employee"   => terminate(args, ctx),
        "restore_employee"     => restore(args, ctx),
        "list_by_department"   => list_by_dept(args, ctx),
        "search_employees"     => search(args, ctx),
        "get_employee_history" => history(args, ctx),
        other => Err(format!("Unknown function: {}", other)),
    }
}

// ── Write functions ───────────────────────────────────────────────────────────

fn hire(args: serde_json::Value, ctx: &dyn ChainContext) -> Result<serde_json::Value, String> {
    let input: HireInput = serde_json::from_value(args)
        .map_err(|e| format!("Invalid args: {}", e))?;

    // Role check
    if !ctx.has_role("HR_MANAGER") && !ctx.has_role("HR_ADMIN") {
        return Err("Requires HR_MANAGER or HR_ADMIN role".into());
    }

    // Business validations
    if input.salary < 3_000_000 {
        return Err(format!("Salary {} is below minimum wage (3,000,000)", input.salary));
    }
    if input.name.trim().is_empty() {
        return Err("Employee name cannot be empty".into());
    }
    if !input.email.contains('@') {
        return Err(format!("Invalid email: {}", input.email));
    }

    let mut fields: HashMap<String, FieldValue> = HashMap::new();
    fields.insert("name".into(),       FieldValue::Text(input.name));
    fields.insert("department".into(), FieldValue::Text(input.department.clone()));
    fields.insert("position".into(),   FieldValue::Text(input.position));
    fields.insert("salary".into(),     FieldValue::Int(input.salary));
    fields.insert("join_date".into(),  FieldValue::Text(input.join_date));
    fields.insert("email".into(),      FieldValue::Text(input.email));
    fields.insert("status".into(),     FieldValue::Text("ACTIVE".into()));

    if let Some(phone) = input.phone {
        fields.insert("phone".into(), FieldValue::Text(phone));
    }
    if let Some(mgr) = input.manager_id {
        fields.insert("manager_id".into(), FieldValue::Text(mgr));
    }

    let record = ctx.insert(COLLECTION, &input.department, &input.id, fields)
        .map_err(|e| e.to_string())?;

    ctx.emit_event("EMPLOYEE_HIRED", input.id.as_bytes());
    Ok(record)
}

fn update_salary(args: serde_json::Value, ctx: &dyn ChainContext) -> Result<serde_json::Value, String> {
    let input: UpdateSalaryInput = serde_json::from_value(args)
        .map_err(|e| format!("Invalid args: {}", e))?;

    if !ctx.has_role("HR_MANAGER") && !ctx.has_role("FINANCE") {
        return Err("Requires HR_MANAGER or FINANCE role".into());
    }
    if input.new_salary < 3_000_000 {
        return Err(format!("New salary {} below minimum", input.new_salary));
    }

    // Get current to validate status
    let current = ctx.get(COLLECTION, &input.department, &input.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Employee {} not found", input.id))?;

    if let Some(status) = current.get("status") {
        if status == "TERMINATED" { return Err("Cannot update terminated employee".into()); }
    }

    let mut patches = HashMap::new();
    patches.insert("salary".into(),            FieldValue::Int(input.new_salary));
    patches.insert("salary_reason".into(),     FieldValue::Text(input.reason));
    patches.insert("salary_effective".into(),  FieldValue::Text(input.effective));

    let record = ctx.patch(COLLECTION, &input.department, &input.id, patches)
        .map_err(|e| e.to_string())?;

    ctx.emit_event("SALARY_UPDATED", input.id.as_bytes());
    Ok(record)
}

fn transfer(args: serde_json::Value, ctx: &dyn ChainContext) -> Result<serde_json::Value, String> {
    let input: TransferInput = serde_json::from_value(args)
        .map_err(|e| format!("Invalid args: {}", e))?;

    if !ctx.has_role("HR_MANAGER") {
        return Err("Requires HR_MANAGER role".into());
    }

    // Get existing record
    let current = ctx.get(COLLECTION, &input.from_dept, &input.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Employee {} not found in {}", input.id, input.from_dept))?;

    // Soft-delete from old partition
    ctx.delete(COLLECTION, &input.from_dept, &input.id,
        Some(format!("Transferred to {}: {}", input.to_dept, input.reason)))
        .map_err(|e| e.to_string())?;

    // Rebuild fields for new partition
    let mut new_fields: HashMap<String, FieldValue> = HashMap::new();
    if let Some(obj) = current.as_object() {
        if let Some(fields) = obj.get("fields").and_then(|f| f.as_object()) {
            for (k, v) in fields {
                if let Ok(fv) = serde_json::from_value::<FieldValue>(v.clone()) {
                    new_fields.insert(k.clone(), fv);
                }
            }
        }
    }
    new_fields.insert("department".into(),    FieldValue::Text(input.to_dept.clone()));
    new_fields.insert("transfer_from".into(), FieldValue::Text(input.from_dept));
    new_fields.insert("transfer_reason".into(),FieldValue::Text(input.reason));
    if let Some(pos) = input.new_position {
        new_fields.insert("position".into(), FieldValue::Text(pos));
    }

    let record = ctx.insert(COLLECTION, &input.to_dept, &input.id, new_fields)
        .map_err(|e| e.to_string())?;

    ctx.emit_event("EMPLOYEE_TRANSFERRED", input.id.as_bytes());
    Ok(record)
}

fn terminate(args: serde_json::Value, ctx: &dyn ChainContext) -> Result<serde_json::Value, String> {
    let input: TerminateInput = serde_json::from_value(args)
        .map_err(|e| format!("Invalid args: {}", e))?;

    if !ctx.has_role("HR_MANAGER") {
        return Err("Requires HR_MANAGER role".into());
    }

    // Patch status fields before soft-delete so history has final state
    let mut patches = HashMap::new();
    patches.insert("status".into(),             FieldValue::Text("TERMINATED".into()));
    patches.insert("termination_reason".into(), FieldValue::Text(input.reason.clone()));
    patches.insert("exit_date".into(),          FieldValue::Text(input.exit_date));
    ctx.patch(COLLECTION, &input.department, &input.id, patches)
        .map_err(|e| e.to_string())?;

    ctx.delete(COLLECTION, &input.department, &input.id, Some(input.reason))
        .map_err(|e| e.to_string())?;

    ctx.emit_event("EMPLOYEE_TERMINATED", input.id.as_bytes());
    Ok(serde_json::json!({ "status": "terminated", "id": input.id }))
}

fn restore(args: serde_json::Value, ctx: &dyn ChainContext) -> Result<serde_json::Value, String> {
    #[derive(Deserialize)] struct Input { id: String, department: String }
    let input: Input = serde_json::from_value(args)
        .map_err(|e| format!("Invalid args: {}", e))?;

    if !ctx.has_role("HR_ADMIN") {
        return Err("Requires HR_ADMIN role".into());
    }

    // Reset status to ACTIVE
    let record = ctx.restore(COLLECTION, &input.department, &input.id)
        .map_err(|e| e.to_string())?;

    let mut patches = HashMap::new();
    patches.insert("status".into(), FieldValue::Text("ACTIVE".into()));
    ctx.patch(COLLECTION, &input.department, &input.id, patches)
        .map_err(|e| e.to_string())?;

    ctx.emit_event("EMPLOYEE_RESTORED", input.id.as_bytes());
    Ok(record)
}

// ── Read functions ────────────────────────────────────────────────────────────

fn get(args: serde_json::Value, ctx: &dyn ChainContext) -> Result<serde_json::Value, String> {
    #[derive(Deserialize)] struct Input { id: String, department: String }
    let input: Input = serde_json::from_value(args)
        .map_err(|e| format!("Invalid args: {}", e))?;

    ctx.get(COLLECTION, &input.department, &input.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Employee {} not found", input.id))
}

fn list_by_dept(args: serde_json::Value, ctx: &dyn ChainContext) -> Result<serde_json::Value, String> {
    #[derive(Deserialize)] struct Input {
        department:          String,
        include_terminated:  Option<bool>,
        limit:               Option<u32>,
        _offset:             Option<u32>,
    }
    let input: Input = serde_json::from_value(args)
        .map_err(|e| format!("Invalid args: {}", e))?;

    let filter = if input.include_terminated.unwrap_or(false) {
        Some(Filter::IncludeDeleted)
    } else {
        Some(Filter::Eq("status".into(), FieldValue::Text("ACTIVE".into())))
    };

    let result = ctx.query(
        COLLECTION,
        Some(&input.department),
        filter,
        Some(vec![chain_sdk::SortBy { field: "name".into(), descending: false }]),
        Some(chain_sdk::Pagination { limit: input.limit.unwrap_or(50), page_token: None }),
    ).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "records": result.records,
        "total":   result.total,
    }))
}

fn search(args: serde_json::Value, ctx: &dyn ChainContext) -> Result<serde_json::Value, String> {
    let input: SearchInput = serde_json::from_value(args)
        .map_err(|e| format!("Invalid args: {}", e))?;

    let mut conditions: Vec<Filter> = vec![];

    if let Some(pos) = input.position {
        conditions.push(Filter::Eq("position".into(), FieldValue::Text(pos)));
    }
    if let (Some(min), Some(max)) = (input.salary_min, input.salary_max) {
        conditions.push(Filter::Gte("salary".into(), FieldValue::Int(min)));
        conditions.push(Filter::Lte("salary".into(), FieldValue::Int(max)));
    } else if let Some(min) = input.salary_min {
        conditions.push(Filter::Gte("salary".into(), FieldValue::Int(min)));
    } else if let Some(max) = input.salary_max {
        conditions.push(Filter::Lte("salary".into(), FieldValue::Int(max)));
    }
    // Default: only active employees
    conditions.push(Filter::Eq("status".into(), FieldValue::Text("ACTIVE".into())));

    let filter = match conditions.len() {
        0 => None,
        1 => Some(conditions.remove(0)),
        _ => Some(Filter::And(conditions)),
    };

    let result = ctx.query(
        COLLECTION,
        input.department.as_deref(),
        filter,
        Some(vec![chain_sdk::SortBy { field: "salary".into(), descending: true }]),
        Some(chain_sdk::Pagination {
            limit:      input.limit.unwrap_or(20),
            page_token: None,
        }),
    ).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "records": result.records,
        "total":   result.total,
    }))
}

fn history(args: serde_json::Value, ctx: &dyn ChainContext) -> Result<serde_json::Value, String> {
    #[derive(Deserialize)] struct Input { id: String }
    let input: Input = serde_json::from_value(args)
        .map_err(|e| format!("Invalid args: {}", e))?;

    let entries = ctx.get_history(COLLECTION, &input.id)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({ "id": input.id, "history": entries }))
}

// ── Registration helper (used by qorvum-node) ─────────────────────────────────

/// Returns the function registry for native contract registration.
pub fn register() -> HashMap<String, qorvum_contracts::executor::NativeFn> {
    let mut m: HashMap<String, qorvum_contracts::executor::NativeFn> = HashMap::new();
    m.insert("hire_employee".into(),        |f, a, c| dispatch(f, a, c).map_err(|e| e));
    m.insert("get_employee".into(),         |f, a, c| dispatch(f, a, c).map_err(|e| e));
    m.insert("update_salary".into(),        |f, a, c| dispatch(f, a, c).map_err(|e| e));
    m.insert("transfer_department".into(),  |f, a, c| dispatch(f, a, c).map_err(|e| e));
    m.insert("terminate_employee".into(),   |f, a, c| dispatch(f, a, c).map_err(|e| e));
    m.insert("restore_employee".into(),     |f, a, c| dispatch(f, a, c).map_err(|e| e));
    m.insert("list_by_department".into(),   |f, a, c| dispatch(f, a, c).map_err(|e| e));
    m.insert("search_employees".into(),     |f, a, c| dispatch(f, a, c).map_err(|e| e));
    m.insert("get_employee_history".into(), |f, a, c| dispatch(f, a, c).map_err(|e| e));
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use chain_sdk::ChainContext;
    use qorvum_contracts::context::ChainContextImpl;
    use qorvum_ledger::backends::MemoryStore;
    use std::sync::Arc;

    fn make_ctx(caller_id: &str, roles: Vec<&str>) -> ChainContextImpl {
        let store = Arc::new(MemoryStore::new());
        let tx_id = qorvum_crypto::hash(b"test-tx");
        ChainContextImpl::new(
            store,
            caller_id.to_string(),
            "TestMSP".to_string(),
            roles.into_iter().map(|s| s.to_string()).collect(),
            tx_id,
            1_700_000_000_000_000_000u64,
        )
    }

    fn hire_emp(ctx: &dyn ChainContext, id: &str, dept: &str, salary: i64) -> serde_json::Value {
        dispatch("hire_employee", serde_json::json!({
            "id":         id,
            "name":       format!("Test Employee {}", id),
            "department": dept,
            "position":   "Engineer",
            "salary":     salary,
            "join_date":  "2025-01-01",
            "email":      format!("{}@company.com", id.to_lowercase()),
        }), ctx).unwrap()
    }

    #[test]
    fn test_hire_and_get() {
        let ctx = make_ctx("admin@org1", vec!["HR_MANAGER"]);
        let rec = hire_emp(&ctx, "EMP001", "IT", 15_000_000);
        assert_eq!(rec["meta"]["id"], "EMP001");
        assert_eq!(rec["fields"]["status"]["v"], "ACTIVE");
    }

    #[test]
    fn test_hire_requires_role() {
        let ctx = make_ctx("user@org1", vec!["EMPLOYEE"]);
        let result = dispatch("hire_employee", serde_json::json!({
            "id": "EMP002", "name": "X", "department": "IT",
            "position": "X", "salary": 5_000_000,
            "join_date": "2025-01-01", "email": "x@x.com",
        }), &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("HR_MANAGER"));
    }

    #[test]
    fn test_hire_below_min_wage() {
        let ctx = make_ctx("admin@org1", vec!["HR_MANAGER"]);
        let result = dispatch("hire_employee", serde_json::json!({
            "id": "EMP003", "name": "X", "department": "IT",
            "position": "X", "salary": 1_000_000,
            "join_date": "2025-01-01", "email": "x@x.com",
        }), &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("minimum wage"));
    }

    #[test]
    fn test_update_salary() {
        let ctx = make_ctx("admin@org1", vec!["HR_MANAGER"]);
        hire_emp(&ctx, "EMP004", "HR", 8_000_000);

        let result = dispatch("update_salary", serde_json::json!({
            "id": "EMP004", "department": "HR",
            "new_salary": 10_000_000,
            "reason": "Annual review",
            "effective": "2025-07-01",
        }), &ctx);
        assert!(result.is_ok());
        let rec = result.unwrap();
        assert_eq!(rec["fields"]["salary"]["v"], 10_000_000);
    }

    #[test]
    fn test_list_by_department() {
        let ctx = make_ctx("admin@org1", vec!["HR_MANAGER"]);
        hire_emp(&ctx, "EMP010", "IT", 15_000_000);
        hire_emp(&ctx, "EMP011", "IT", 18_000_000);
        hire_emp(&ctx, "EMP012", "HR", 12_000_000);

        let result = dispatch("list_by_department", serde_json::json!({
            "department": "IT"
        }), &ctx).unwrap();

        assert_eq!(result["total"], 2);
    }

    #[test]
    fn test_terminate_and_restore() {
        let ctx = make_ctx("admin@org1", vec!["HR_MANAGER", "HR_ADMIN"]);
        hire_emp(&ctx, "EMP020", "IT", 20_000_000);

        // Terminate
        dispatch("terminate_employee", serde_json::json!({
            "id": "EMP020", "department": "IT",
            "reason": "Resignation",
            "exit_date": "2025-06-01",
        }), &ctx).unwrap();

        // Employee should be hidden from active list
        let list = dispatch("list_by_department",
            serde_json::json!({"department": "IT"}), &ctx).unwrap();
        assert_eq!(list["total"], 0);

        // Restore
        dispatch("restore_employee", serde_json::json!({
            "id": "EMP020", "department": "IT"
        }), &ctx).unwrap();

        let list2 = dispatch("list_by_department",
            serde_json::json!({"department": "IT"}), &ctx).unwrap();
        assert_eq!(list2["total"], 1);
    }

    #[test]
    fn test_search_by_salary() {
        let ctx = make_ctx("admin@org1", vec!["HR_MANAGER"]);
        hire_emp(&ctx, "EMP030", "IT", 10_000_000);
        hire_emp(&ctx, "EMP031", "IT", 20_000_000);
        hire_emp(&ctx, "EMP032", "IT", 30_000_000);

        let result = dispatch("search_employees", serde_json::json!({
            "salary_min": 15_000_000,
            "salary_max": 35_000_000,
        }), &ctx).unwrap();
        assert_eq!(result["total"], 2);
    }
}
