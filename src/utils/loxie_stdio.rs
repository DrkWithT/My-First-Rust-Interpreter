use crate::vm::{callable::ExecStatus, engine::Engine, value::Value};

pub fn native_read_int(engine_ref: &mut Engine) -> ExecStatus {
    println!("Enter an integer: ");

    let mut raw_input = String::new();

    if std::io::stdin().read_line(&mut raw_input).is_err() {
        return ExecStatus::NotOk;
    }

    let temp_int = raw_input.trim().parse::<i32>();

    if temp_int.is_err() {
        eprintln!("Invalid input for int: '{raw_input}'");
        return ExecStatus::BadArgs;
    }

    engine_ref.push_in(Value::Int(temp_int.unwrap()));

    ExecStatus::Ok
}

pub fn native_print_val(engine_ref: &mut Engine) -> ExecStatus {
    let temp_value_opt = engine_ref.pop_off();

    if temp_value_opt.is_none() {
        engine_ref.push_in(Value::Bool(false));
        ExecStatus::NotOk
    } else {
        let temp_value = temp_value_opt.unwrap();
        println!("{temp_value}");

        engine_ref.push_in(Value::Bool(true));

        ExecStatus::Ok
    }
}
