use std::io::{self, Write};
use std::mem;

// Define the structure of the multi-dimensional processing unit
struct ProcessingUnit {
    registers: Vec<i32>,
    memory: Vec<i32>,
    stack_pointer: usize,
}

// Define the structure to hold the state after execution
struct ProcessingUnitState {
    registers: Vec<i32>,
    stack: Vec<i32>,
}

// Define opcodes
#[derive(Debug, Copy, Clone)]
enum Opcode {
    Nop,
    Add,
    Sub,
    Mul,
    Div,
    Store,
    Load,
    LoadImmediate,
    Push,
    Pop,
    Jmp,
    Jz,
    Jnz,
    Mov,
    Je,
    Jne,
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
    Cmp,
    Test,
    B,
    Bz,
    Bnz,
    Neg,
    Abs,
    Mod,
    Inc,
    Dec,
    Halt,
}

// Define the structure of an instruction
struct Instruction {
    opcode: Opcode,
    reg1: usize,
    reg2: usize,
    reg3: usize,
    addr: usize,
    immediate: i32,
}

impl ProcessingUnit {
    // Function to initialize the processing unit
    fn initialize(num_registers: usize, memory_size: usize) -> Self {
        ProcessingUnit {
            registers: vec![0; num_registers],
            memory: vec![0; memory_size],
            stack_pointer: memory_size - 1, // Initialize stack pointer to the top of the memory
        }
    }

    // Helper function to check register bounds
    fn check_register_bounds(&self, reg: usize) {
        if reg >= self.registers.len() {
            eprintln!("Error: Register index out of bounds: R{}", reg);
            std::process::exit(1);
        }
    }

    // ++++++++++++++++++++++++++++++ Arithmetic operations ++++++++++++++++++++++++++++++ //
    fn add(&mut self, reg1: usize, reg2: usize, reg3: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.check_register_bounds(reg3);
        self.registers[reg3] = self.registers[reg1] + self.registers[reg2];
    }

    fn subtract(&mut self, reg1: usize, reg2: usize, reg3: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.check_register_bounds(reg3);
        self.registers[reg3] = self.registers[reg1] - self.registers[reg2];
    }

    fn multiply(&mut self, reg1: usize, reg2: usize, reg3: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.check_register_bounds(reg3);
        self.registers[reg3] = self.registers[reg1] * self.registers[reg2];
    }

    fn divide(&mut self, reg1: usize, reg2: usize, reg3: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.check_register_bounds(reg3);
        if self.registers[reg2] != 0 {
            self.registers[reg3] = self.registers[reg1] / self.registers[reg2];
        } else {
            eprintln!(
                "Error: Division by zero on R{} of value {}",
                reg2, self.registers[reg2]
            );
            std::process::exit(1);
        }
    }

    fn neg(&mut self, reg1: usize, reg2: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.registers[reg2] = -self.registers[reg1];
    }

    fn absolute(&mut self, reg1: usize, reg2: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.registers[reg2] = self.registers[reg1].abs();
    }

    fn mod_op(&mut self, reg1: usize, reg2: usize, reg3: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.check_register_bounds(reg3);
        if self.registers[reg2] != 0 {
            self.registers[reg3] = self.registers[reg1] % self.registers[reg2];
        } else {
            eprintln!(
                "Error: Division by zero on R{} of value {}",
                reg2, self.registers[reg2]
            );
            std::process::exit(1);
        }
    }

    // ++++++++++++++++++++++++++++++ Memory operations ++++++++++++++++++++++++++++++ //
    fn store(&mut self, reg: usize, addr: usize) {
        self.check_register_bounds(reg);
        if addr < self.memory.len() {
            self.memory[addr] = self.registers[reg];
        } else {
            eprintln!("Error: Memory address out of bounds: {}", addr);
            std::process::exit(1);
        }
    }

    fn load(&mut self, addr: usize, reg: usize) {
        self.check_register_bounds(reg);
        if addr < self.memory.len() {
            self.registers[reg] = self.memory[addr];
        } else {
            eprintln!("Error: Memory address out of bounds: {}", addr);
            std::process::exit(1);
        }
    }

    // ++++++++++++++++++++++++++++++ Stack operations ++++++++++++++++++++++++++++++ //
    fn push(&mut self, reg: usize) {
        self.check_register_bounds(reg);
        if self.stack_pointer > 0 {
            self.memory[self.stack_pointer] = self.registers[reg];
            self.stack_pointer -= 1;
        } else {
            eprintln!("Error: Stack overflow on R{}", reg);
            std::process::exit(1);
        }
    }

    fn pop(&mut self, reg: usize) {
        self.check_register_bounds(reg);
        if self.stack_pointer < self.memory.len() - 1 {
            self.stack_pointer += 1;
            self.registers[reg] = self.memory[self.stack_pointer];
        } else {
            eprintln!("Error: Stack underflow on R{}", reg);
            std::process::exit(1);
        }
    }

    fn mov(&mut self, reg1: usize, reg2: usize) {
        self.check_register_bounds(reg1);
        self.check_register_bounds(reg2);
        self.registers[reg1] = self.registers[reg2];
    }
}

// Function to run the program and return the state
fn run(pu: &mut ProcessingUnit, program: &[Instruction], mic: usize) -> ProcessingUnitState {
    execute_program(pu, program, mic);
    let stack_size = pu.memory.len() - pu.stack_pointer - 1;

    let stack = pu.memory[pu.stack_pointer + 1..].to_vec();
    let registers = pu.registers.clone();

    ProcessingUnitState { registers, stack }
}

// ++++++++++++++++++++++++++++++ Program execution ++++++++++++++++++++++++++++++ //
fn execute_program(pu: &mut ProcessingUnit, program: &[Instruction], mic: usize) {
    let max_instruction_count = mic;
    let mut instruction_count = 0;
    let mut instruction_pointer = 0;

    while instruction_pointer < program.len() {
        if instruction_count >= max_instruction_count {
            eprintln!("Error: Maximum instruction count exceeded, possible infinite loop");
            std::process::exit(1);
        }

        let instr = &program[instruction_pointer];
        match instr.opcode {
            Opcode::Add => pu.add(instr.reg1, instr.reg2, instr.reg3),
            Opcode::Sub => pu.subtract(instr.reg1, instr.reg2, instr.reg3),
            Opcode::Mul => pu.multiply(instr.reg1, instr.reg2, instr.reg3),
            Opcode::Div => pu.divide(instr.reg1, instr.reg2, instr.reg3),
            Opcode::Store => pu.store(instr.reg1, instr.addr),
            Opcode::Load => pu.load(instr.addr, instr.reg1),
            Opcode::LoadImmediate => {
                pu.check_register_bounds(instr.reg1);
                pu.registers[instr.reg1] = instr.immediate;
            }
            Opcode::Push => pu.push(instr.reg1),
            Opcode::Pop => pu.pop(instr.reg1),
            Opcode::Jmp => {
                instruction_pointer = instr.addr;
                continue;
            }
            Opcode::Jz => {
                pu.check_register_bounds(instr.reg1);
                if pu.registers[instr.reg1] == 0 {
                    instruction_pointer = instr.addr;
                    continue;
                }
            }
            Opcode::Jnz => {
                pu.check_register_bounds(instr.reg1);
                if pu.registers[instr.reg1] != 0 {
                    instruction_pointer = instr.addr;
                    continue;
                }
            }
            Opcode::Mov => pu.mov(instr.reg1, instr.reg2),
            Opcode::Je => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                if pu.registers[instr.reg1] == pu.registers[instr.reg2] {
                    instruction_pointer = instr.addr;
                }
            }
            Opcode::Jne => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                if pu.registers[instr.reg1] != pu.registers[instr.reg2] {
                    instruction_pointer = instr.addr;
                }
            }
            Opcode::And => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] & pu.registers[instr.reg2];
            }
            Opcode::Or => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] | pu.registers[instr.reg2];
            }
            Opcode::Xor => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] ^ pu.registers[instr.reg2];
            }
            Opcode::Not => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.registers[instr.reg2] = !pu.registers[instr.reg1];
            }
            Opcode::Shl => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] << pu.registers[instr.reg2];
            }
            Opcode::Shr => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] >> pu.registers[instr.reg2];
            }
            Opcode::Cmp => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = (pu.registers[instr.reg1] - pu.registers[instr.reg2]);
            }
            Opcode::Test => {
                pu.check_register_bounds(instr.reg1);
                pu.check_register_bounds(instr.reg2);
                pu.check_register_bounds(instr.reg3);
                pu.registers[instr.reg3] = pu.registers[instr.reg1] & pu.registers[instr.reg2];
            }
            Opcode::B => {
                instruction_pointer = instr.addr;
                continue;
            }
            Opcode::Bz => {
                pu.check_register_bounds(instr.reg1);
                if pu.registers[instr.reg1] == 0 {
                    instruction_pointer = instr.addr;
                    continue;
                }
            }
            Opcode::Bnz => {
                pu.check_register_bounds(instr.reg1);
                if pu.registers[instr.reg1] != 0 {
                    instruction_pointer = instr.addr;
                    continue;
                }
            }
            Opcode::Neg => pu.neg(instr.reg1, instr.reg2),
            Opcode::Abs => pu.absolute(instr.reg1, instr.reg2),
            Opcode::Mod => pu.mod_op(instr.reg1, instr.reg2, instr.reg3),
            Opcode::Inc => {
                pu.check_register_bounds(instr.reg1);
                pu.registers[instr.reg1] += 1;
            }
            Opcode::Dec => {
                pu.check_register_bounds(instr.reg1);
                pu.registers[instr.reg1] -= 1;
            }
            Opcode::Nop => {} // No operation
            Opcode::Halt => break, // Stop execution
        }

        instruction_count += 1;
        instruction_pointer += 1;
    }
}

fn main() {
    let mut pu = ProcessingUnit::initialize(8, 128);

    // Sample program instructions
    let program = vec![
        Instruction {
            opcode: Opcode::LoadImmediate,
            reg1: 0,
            reg2: 0,
            reg3: 0,
            addr: 0,
            immediate: 10,
        },
        Instruction {
            opcode: Opcode::LoadImmediate,
            reg1: 1,
            reg2: 0,
            reg3: 0,
            addr: 0,
            immediate: 20,
        },
        Instruction {
            opcode: Opcode::Add,
            reg1: 0,
            reg2: 1,
            reg3: 2,
            addr: 0,
            immediate: 0,
        },
        Instruction {
            opcode: Opcode::Halt,
            reg1: 0,
            reg2: 0,
            reg3: 0,
            addr: 0,
            immediate: 0,
        },
    ];

    let mic = 1000; // Maximum instruction count
    let state = run(&mut pu, &program, mic);

    println!("Registers: {:?}", state.registers);
    println!("Stack: {:?}", state.stack);
}
