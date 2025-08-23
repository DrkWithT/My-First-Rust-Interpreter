use crate::vm::{bytecode, callable::ExecStatus, engine::Engine, heap::HeapValue, value::Value};

pub fn native_intrin_varchar_len(engine_ref: &mut Engine) -> ExecStatus {
    let vc_ref_opt = engine_ref.pop_off();

    if vc_ref_opt.is_none() {
        eprintln!("Unexpected none reference to varchar!");
        return ExecStatus::RefError;
    }

    let mut fallback_dud: HeapValue = HeapValue::Empty();

    let vc_heap_id = vc_ref_opt.unwrap();
    let vc_len = engine_ref.fetch_heap_value_by(
        (
            bytecode::ArgMode::HeapId,
            if let Value::HeapRef(obj_id) = vc_heap_id { obj_id as i32 } else { -1 }
        )
    ).unwrap_or(&mut fallback_dud).try_varchar_len();

    engine_ref.push_in(Value::Int(vc_len));

    ExecStatus::Ok
}

pub fn native_intrin_varchar_get(engine_ref: &mut Engine) -> ExecStatus {
    let vc_index = engine_ref.pop_off().unwrap_or(Value::Int(-1));
    let vc_ref_opt = engine_ref.pop_off();

    if vc_ref_opt.is_none() {
        eprintln!("Unexpected none reference to varchar!");
        return ExecStatus::RefError;
    }

    let mut fallback_dud: HeapValue = HeapValue::Empty();

    let vc_heap_id = vc_ref_opt.unwrap();
    let vc_item = engine_ref.fetch_heap_value_by(
        (
            bytecode::ArgMode::HeapId,
            if let Value::HeapRef(obj_id) = vc_heap_id { obj_id as i32 } else { -1 }
        )
    ).unwrap_or(&mut fallback_dud).try_varchar_get(vc_index.into::<>());

    engine_ref.push_in(Value::Char(vc_item));

    ExecStatus::NotOk
}

pub fn native_intrin_varchar_set(engine_ref: &mut Engine) -> ExecStatus {
    let next_ascii_c = engine_ref.pop_off().unwrap_or(Value::Char(0));
    let vc_index = engine_ref.pop_off().unwrap_or(Value::Int(-1));
    let vc_ref_opt = engine_ref.pop_off();

    if vc_ref_opt.is_none() {
        eprintln!("Unexpected none reference to varchar!");
        return ExecStatus::RefError;
    }

    let mut fallback_dud: HeapValue = HeapValue::Empty();

    let vc_heap_id = vc_ref_opt.unwrap();
    let result_flag = engine_ref.fetch_heap_value_by(
        (
            bytecode::ArgMode::HeapId,
            if let Value::HeapRef(obj_id) = vc_heap_id { obj_id as i32 } else { -1 }
        )
    ).unwrap_or(&mut fallback_dud).try_varchar_set(vc_index.into::<>(), next_ascii_c.into::<>());

    engine_ref.push_in(Value::Bool(result_flag));

    ExecStatus::NotOk
}

pub fn native_intrin_varchar_push(engine_ref: &mut Engine) -> ExecStatus {
    let next_ascii_c = engine_ref.pop_off().unwrap_or(Value::Char(0));
    let vc_ref_opt = engine_ref.pop_off();

    if vc_ref_opt.is_none() {
        eprintln!("Unexpected none reference to varchar!");
        return ExecStatus::RefError;
    }

    let mut fallback_dud = HeapValue::Empty();

    let vc_heap_id = vc_ref_opt.unwrap();
    let result_flag = engine_ref.fetch_heap_value_by(
        (
            bytecode::ArgMode::HeapId,
            if let Value::HeapRef(obj_id) = vc_heap_id { obj_id as i32 } else { -1 }
        )
    ).unwrap_or(&mut fallback_dud).try_varchar_push(next_ascii_c.into::<>());

    engine_ref.push_in(Value::Bool(result_flag));

    ExecStatus::NotOk
}

pub fn native_intrin_varchar_pop(engine_ref: &mut Engine) -> ExecStatus {
    let vc_ref_opt = engine_ref.pop_off();

    if vc_ref_opt.is_none() {
        eprintln!("Unexpected none reference to varchar!");
        return ExecStatus::RefError;
    }

    let mut fallback_dud: HeapValue = HeapValue::Empty();

    let vc_heap_id = vc_ref_opt.unwrap();
    let vc_item = engine_ref.fetch_heap_value_by(
        (
            bytecode::ArgMode::HeapId,
            if let Value::HeapRef(obj_id) = vc_heap_id { obj_id as i32 } else { -1 }
        )
    ).unwrap_or(&mut fallback_dud).try_varchar_pop();

    engine_ref.push_in(Value::Char(vc_item));

    ExecStatus::NotOk
}
