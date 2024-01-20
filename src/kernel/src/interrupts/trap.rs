use core::panic;

use common::syscalls::trap_frame::{Register, TrapFrame};

use crate::{
    cpu::{self, read_satp, write_satp_and_fence},
    debug,
    interrupts::plic::{self, InterruptSource},
    io::{stdin_buf::STDIN_BUFFER, uart},
    memory::page_tables::{activate_page_table, KERNEL_PAGE_TABLES},
    processes::{
        scheduler::{self, get_current_process_expect},
        timer,
    },
    syscalls::handle_syscall,
};

use super::trap_cause::InterruptCause;
use super::trap_cause::{exception::ENVIRONMENT_CALL_FROM_U_MODE, interrupt::*};

#[no_mangle]
extern "C" fn supervisor_mode_trap(
    cause: InterruptCause,
    stval: usize,
    sepc: usize,
    trap_frame: &mut TrapFrame,
) {
    let old_tables = read_satp();
    debug!("Activate KERNEL_PAGE_TABLES");
    activate_page_table(&KERNEL_PAGE_TABLES.lock());
    debug!(
        "Supervisor mode trap occurred! (sepc: {:x?}) (cause: {:?})\nTrap Frame: {:?}",
        sepc,
        cause.get_reason(),
        trap_frame
    );
    if cause.is_interrupt() {
        handle_interrupt(cause, stval, sepc, trap_frame);
    } else {
        handle_exception(cause, stval, sepc, trap_frame);
    }

    // Restore old page tables
    // SAFTEY: They should be valid. If a process dies we don't come here
    // because the scheduler returns with restore_user_context
    // Hoewever: This is very ugly and prone to error.
    // TODO: Find a better way to do this
    unsafe {
        write_satp_and_fence(old_tables);
    }
}

fn handle_exception(cause: InterruptCause, stval: usize, sepc: usize, trap_frame: &mut TrapFrame) {
    match cause.get_exception_code() {
        ENVIRONMENT_CALL_FROM_U_MODE => {
            cpu::write_sepc(sepc + 4); // Skip the ecall instruction
            trap_frame[Register::a0] = handle_syscall(trap_frame) as usize;
        }
        _ => {
            panic!(
                "Unhandled exception! (Name: {}) (Exception code: {}) (stval: 0x{:x}) (sepc: 0x{:x}) (From Userspace: {})\nTrap Frame: {:?}",
                cause.get_reason(),
                cause.get_exception_code(),
                stval,
                sepc,
                get_current_process_expect().borrow().get_page_table().is_userspace_address(sepc),
                trap_frame
            );
        }
    }
}

fn handle_interrupt(cause: InterruptCause, _stval: usize, _sepc: usize, _trap_frame: &TrapFrame) {
    match cause.get_exception_code() {
        SUPERVISOR_TIMER_INTERRUPT => handle_supervisor_timer_interrupt(),
        SUPERVISOR_EXTERNAL_INTERRUPT => handle_external_interrupt(),
        _ => {
            panic!("Unknwon interrupt! (Name: {})", cause.get_reason());
        }
    }
}

fn handle_supervisor_timer_interrupt() {
    timer::set_timer(10000);
    scheduler::schedule();
}

fn handle_external_interrupt() {
    debug!("External interrupt occurred!");
    let plic_interrupt = plic::get_next_pending().expect("There should be a pending interrupt.");
    assert!(
        plic_interrupt == InterruptSource::Uart,
        "Plic interrupt should be uart."
    );

    let input = uart::read().expect("There should be input from the uart.");
    STDIN_BUFFER.lock().push(input);

    plic::complete_interrupt(plic_interrupt);
}
