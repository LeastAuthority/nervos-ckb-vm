#![cfg(all(unix, target_pointer_width = "64", feature = "asm"))]

use bytes::Bytes;
use ckb_vm::{
    machine::{
        aot::AotCompilingMachine,
        asm::{AsmCoreMachine, AsmMachine},
    },
    registers::{A0, A1, A2, A3, A4, A5, A7},
    DefaultMachineBuilder, Error, Instruction, Register, SupportMachine, Syscalls,
};
use std::fs::File;
use std::io::Read;

#[test]
pub fn test_aot_simple64() {
    let mut file = File::open("tests/programs/simple64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let mut aot_machine = AotCompilingMachine::load(&buffer, None).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::default_with_aot_code(&code);
    machine
        .load_program(&buffer, &vec!["simple".into()])
        .unwrap();
    let result = machine.run();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

pub struct CustomSyscall {}

impl<Mac: SupportMachine> Syscalls<Mac> for CustomSyscall {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, Error> {
        let code = &machine.registers()[A7];
        if code.to_i32() != 1111 {
            return Ok(false);
        }
        let result = machine.registers()[A0]
            .overflowing_add(&machine.registers()[A1])
            .overflowing_add(&machine.registers()[A2])
            .overflowing_add(&machine.registers()[A3])
            .overflowing_add(&machine.registers()[A4])
            .overflowing_add(&machine.registers()[A5]);
        machine.set_register(A0, result);
        Ok(true)
    }
}

#[test]
pub fn test_aot_with_custom_syscall() {
    let mut file = File::open("tests/programs/syscall64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let core = DefaultMachineBuilder::<Box<AsmCoreMachine>>::default()
        .syscall(Box::new(CustomSyscall {}))
        .build();
    let mut aot_machine = AotCompilingMachine::load(&buffer, None).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::new(core, Some(&code));
    machine
        .load_program(&buffer, &vec!["syscall".into()])
        .unwrap();
    let result = machine.run();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 39);
}

fn dummy_cycle_func(_i: Instruction) -> u64 {
    1
}

#[test]
pub fn test_aot_simple_cycles() {
    let mut file = File::open("tests/programs/simple64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let asm_core = AsmCoreMachine::new_with_max_cycles(517);
    let core = DefaultMachineBuilder::<Box<AsmCoreMachine>>::new(asm_core)
        .instruction_cycle_func(Box::new(dummy_cycle_func))
        .build();
    let mut aot_machine =
        AotCompilingMachine::load(&buffer, Some(Box::new(dummy_cycle_func))).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::new(core, Some(&code));
    machine
        .load_program(&buffer, &vec!["syscall".into()])
        .unwrap();
    let result = machine.run();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    assert_eq!(SupportMachine::cycles(&machine.machine), 517);
}

#[test]
pub fn test_aot_simple_max_cycles_reached() {
    let mut file = File::open("tests/programs/simple64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    // Running simple64 should consume 517 cycles using dummy cycle func
    let asm_core = AsmCoreMachine::new_with_max_cycles(500);
    let core = DefaultMachineBuilder::<Box<AsmCoreMachine>>::new(asm_core)
        .instruction_cycle_func(Box::new(dummy_cycle_func))
        .build();
    let mut aot_machine =
        AotCompilingMachine::load(&buffer, Some(Box::new(dummy_cycle_func))).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::new(core, Some(&code));
    machine
        .load_program(&buffer, &vec!["syscall".into()])
        .unwrap();
    let result = machine.run();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::InvalidCycles);
}

#[test]
pub fn test_aot_trace() {
    let mut file = File::open("tests/programs/trace64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let mut aot_machine = AotCompilingMachine::load(&buffer, None).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::default_with_aot_code(&code);
    machine
        .load_program(&buffer, &vec!["simple".into()])
        .unwrap();
    let result = machine.run();
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::InvalidPermission));
}

#[test]
pub fn test_aot_jump0() {
    let mut file = File::open("tests/programs/jump0_64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let mut aot_machine = AotCompilingMachine::load(&buffer, None).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::default_with_aot_code(&code);
    machine
        .load_program(&buffer, &vec!["jump0_64".into()])
        .unwrap();
    let result = machine.run();
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::InvalidPermission));
}

#[test]
pub fn test_aot_write_large_address() {
    let mut file = File::open("tests/programs/write_large_address64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let mut aot_machine = AotCompilingMachine::load(&buffer, None).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::default_with_aot_code(&code);
    machine
        .load_program(&buffer, &vec!["write_large_address64".into()])
        .unwrap();
    let result = machine.run();
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::OutOfBound));
}

#[test]
pub fn test_aot_misaligned_jump64() {
    let mut file = File::open("tests/programs/misaligned_jump64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let mut aot_machine = AotCompilingMachine::load(&buffer, None).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::default_with_aot_code(&code);
    machine
        .load_program(&buffer, &vec!["write_large_address64".into()])
        .unwrap();
    let result = machine.run();
    assert!(result.is_ok());
}

#[test]
pub fn test_aot_mulw64() {
    let mut file = File::open("tests/programs/mulw64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let mut aot_machine = AotCompilingMachine::load(&buffer, None).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::default_with_aot_code(&code);
    machine
        .load_program(&buffer, &vec!["mulw64".into()])
        .unwrap();
    let result = machine.run();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
pub fn test_aot_invalid_read64() {
    let mut file = File::open("tests/programs/invalid_read64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let mut aot_machine = AotCompilingMachine::load(&buffer, None).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::default_with_aot_code(&code);
    machine
        .load_program(&buffer, &vec!["invalid_read64".into()])
        .unwrap();
    let result = machine.run();
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::OutOfBound));
}

#[test]
pub fn test_aot_load_elf_crash_64() {
    let mut file = File::open("tests/programs/load_elf_crash_64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let mut aot_machine = AotCompilingMachine::load(&buffer, None).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::default_with_aot_code(&code);
    machine
        .load_program(&buffer, &vec!["load_elf_crash_64".into()])
        .unwrap();
    let result = machine.run();
    assert_eq!(result.err(), Some(Error::InvalidPermission));
}

#[test]
pub fn test_aot_wxorx_crash_64() {
    let mut file = File::open("tests/programs/wxorx_crash_64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let mut aot_machine = AotCompilingMachine::load(&buffer, None).unwrap();
    let code = aot_machine.compile().unwrap();
    let mut machine = AsmMachine::default_with_aot_code(&code);
    machine
        .load_program(&buffer, &vec!["wxorx_crash_64".into()])
        .unwrap();
    let result = machine.run();
    assert_eq!(result.err(), Some(Error::OutOfBound));
}

#[test]
pub fn test_aot_load_elf_section_crash_64() {
    let mut file = File::open("tests/programs/load_elf_section_crash_64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let result = AotCompilingMachine::load(&buffer, None);
    assert_eq!(result.err(), Some(Error::OutOfBound));
}

#[test]
pub fn test_aot_load_malformed_elf_crash_64() {
    let mut file = File::open("tests/programs/load_malformed_elf_crash_64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let result = AotCompilingMachine::load(&buffer, None);
    assert_eq!(result.err(), Some(Error::ParseError));
}

#[test]
pub fn test_aot_flat_crash_64() {
    let mut file = File::open("tests/programs/flat_crash_64").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer: Bytes = buffer.into();

    let result = AotCompilingMachine::load(&buffer, None);
    assert_eq!(result.err(), Some(Error::OutOfBound));
}
