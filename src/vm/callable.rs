#[repr(i16)]
#[derive(Clone, Copy, PartialEq)]
pub enum ExecStatus {
    Ok,
    AccessError,
    ValueError,
    BadMath,
    IllegalInstruction,
    BadArgs,
}

pub trait Callable<Engine> {
    fn invoke(&self, vm: &mut Engine) -> ExecStatus;
    fn get_id(&self) -> i32;
}
