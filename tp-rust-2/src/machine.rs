use std::io::{self, Write};

const MEMORY_SIZE: usize = 4096;
const NREGS: usize = 16;

const IP: usize = 0;

pub struct Machine{
    memory: [u8; MEMORY_SIZE],
    registers: [u32;NREGS]
}

#[derive(Debug)]
pub enum MachineError {
    //If PC value is greater than the largest adress in memory.
    PCTooLarge,
    //If the value in a register cannot be the adress of another register.
    RegOutOfScale,
    //The value of a register cannot be the adress of a data in memory.
    AdressOutOfMemory,
    //The value of a register cannot be an Unicode.
    NotUnicode,
    //When the execution goes on while an exit instruction was decoded.
    ShouldHaveExited,
    //When the instruction id in a register refers to no instruction.
    NotAnInstruction,
    //When an out instruction fails.
    CouldNotWrite

}

fn as_u32(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) <<  0) +
    ((array[1] as u32) <<  8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}

impl Machine{
    /// Create a new machine in its reset state. The `memory` parameter will
    /// be copied at the beginning of the machine memory.
    ///
    /// # Panics
    /// This function panics when `memory` is larger than the machine memory.
    pub fn new(memory: &[u8]) -> Machine {
        let mut new_mem: [u8;4096] = [0;4096];
        let mut i:u32=0;
            for element in memory.iter() {
                new_mem[i as usize] = *element;
                i+=1;
            }
        if memory.len() > MEMORY_SIZE {panic!();}
        else {
            Machine {
                memory: new_mem,
                registers: [0;16]
            }
        }
    }

    /// Returns value of instruction pointer.
    pub fn get_ip(&mut self) -> u32 {
        return self.registers[0];
    }

    /// Increments the instruction pointer with given value.
    pub fn increment_ip(&mut self, incr:u32) -> () {
        self.registers[0] += incr;
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on `fd`.
    pub fn run_on<T: Write>(&mut self, fd: &mut T) -> Result<(), MachineError> {
        while !self.step_on(fd)? {}
        return Ok(())
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on standard output.
    pub fn run(&mut self) -> Result<(), MachineError> {
        self.run_on(&mut io::stdout().lock())
    }

    /// Execute the next instruction by doing the following steps:
    ///   - decode the instruction located at IP (register 0)
    ///   - increment the IP by the size of the instruction
    ///   - execute the decoded instruction
    ///
    /// If output instructions are run, they print on `fd`.
    /// If an error happens at either of those steps, an error is
    /// returned.
    ///
    /// In case of success, `true` is returned if the program is
    /// terminated (upon encountering an exit instruction), or
    /// `false` if the execution must continue.
    pub fn step_on<T: Write>(&mut self, fd: &mut T) -> Result<bool, MachineError> {
        let pc: usize = self.get_ip().try_into().unwrap();
        if pc >= MEMORY_SIZE {return Err(MachineError::PCTooLarge);}
        else {
            let instr_id: u8 = self.memory[pc];
            let res:Result<(), MachineError>;
            if instr_id==7 {return self.exit()}
            match instr_id {
                1 => res = self.move_if(),
                2 => res = self.store(),
                3 => res = self.load(),
                4 => res = self.loadimm(),
                5 => res = self.sub(),
                6 => res = self.out(fd),
                7 => return Err(MachineError::ShouldHaveExited),
                8 => res = self.out_number(fd),
                _=> return Err(MachineError::NotAnInstruction)
            }
            match res {
                Ok(()) => return Ok(false),
                Err(err) => return Err(err)
            }
        }
    }

    /// Similar to [step_on](Machine::step_on).
    /// If output instructions are run, they print on standard output.
    pub fn step(&mut self) -> Result<bool, MachineError> {
        self.step_on(&mut io::stdout().lock())
    }

    /// Reference onto the machine current set of registers.
    pub fn regs(&self) -> &[u32] {
        return &self.registers;
    }

    /// Sets a register to the given value.
    pub fn set_reg(&mut self, reg: u8, value: u32) -> Result<(), MachineError> {
        if reg > 15 {return Err(MachineError::RegOutOfScale)}
        else {
            self.registers[reg as usize] = value;
            return Ok(());
        }
    }

    /// Sets an adress in memory to the given value.
    pub fn set_mem(&mut self, addr: usize, value: u32) -> Result<(), MachineError> {
        if addr > MEMORY_SIZE-5 {return Err(MachineError::AdressOutOfMemory)}
        else {
            let reg_value:[u8;4] = value.to_be_bytes();
            self.memory[addr] = reg_value[3];
            self.memory[addr+1] = reg_value[2];
            self.memory[addr+2] = reg_value[1];
            self.memory[addr+3] = reg_value[0];
            return Ok(());
        }
    }

    /// Loads a value from given address in memory in given register.
    pub fn load_mem(&mut self, addr: usize, reg: u8) -> Result<(), MachineError> {
        if addr > MEMORY_SIZE-5 {return Err(MachineError::AdressOutOfMemory)}
        if reg > 15 {return Err(MachineError::RegOutOfScale)}
        else {
            let new_reg_value: [u8; 4] = [self.memory[addr as usize], self.memory[(addr as usize)+1], self.memory[(addr as usize)+2], self.memory[(addr as usize)+3]];
            self.registers[reg as usize] = as_u32(&new_reg_value);
            return Ok(());
        }
    }

    /// Reference onto the machine current memory.
    pub fn memory(&self) -> &[u8] {
        return &self.memory;  // Implement me!
    }

    /// 1 regA regB regC : 
    /// If register regC contains a non-zero value, copy the content of register regB into register regA; otherwise do nothing.
    pub fn move_if(&mut self) -> Result<(), MachineError> {
        let reg_a: u8 = self.memory[(self.get_ip()+1) as usize];
        let reg_b: u8 = self.memory[(self.get_ip()+2) as usize];
        let reg_c: u8 = self.memory[(self.get_ip()+3) as usize];
        self.registers[0] += 4;
        if reg_c > 15 || reg_b > 15 {return Err(MachineError::RegOutOfScale)}
        else if self.registers[reg_c as usize] != 0 {
            self.set_reg(reg_a, self.registers[reg_b as usize])
        }
        else {return Ok(())}
    }

    /// 2 regA regB : 
    /// Store the content of register regB into the memory starting at address pointed by register regA using little-endian representation.
    pub fn store(&mut self) -> Result<(), MachineError> {
        let reg_a: u8 = self.memory[(self.get_ip()+1) as usize];
        let reg_b: u8 = self.memory[(self.get_ip()+2) as usize];
        self.registers[0] += 3;
        if reg_b>15 || reg_a>15 {return Err(MachineError::RegOutOfScale)}
        else if self.registers[reg_a as usize] as usize > MEMORY_SIZE-5 {return Err(MachineError::AdressOutOfMemory)}
        else {self.set_mem(self.registers[reg_a as usize] as usize, self.registers[reg_b as usize])}
    }

    /// 3 regA regB : 
    /// Load the 32-bit content from memory at address pointed by register regB into register regA using little-endian representation.
    pub fn load(&mut self) -> Result<(), MachineError> {
        let reg_a: u8 = self.memory[(self.get_ip()+1) as usize];
        let reg_b: u8 = self.memory[(self.get_ip()+2) as usize];
        self.registers[0] += 3;
        if reg_a>15 || reg_b>15 {return Err(MachineError::RegOutOfScale)}
        else if self.registers[reg_b as usize] as usize > MEMORY_SIZE-5 {return Err(MachineError::AdressOutOfMemory)}
        else {self.load_mem(self.registers[reg_b as usize] as usize, reg_a)}
    }

    /// 4 regA L H : 
    /// Interpret H and L respectively as the high-order and the low-order bytes of a 16-bit signed value, sign-extend it to 32 bits, and store it into register regA.
    pub fn loadimm(&mut self) -> Result<(), MachineError> {
        let reg_a: u8 = self.memory[(self.get_ip()+1) as usize];
        let l: u8 = self.memory[(self.get_ip()+2) as usize];
        let h: u8 = self.memory[(self.get_ip()+3) as usize];
        let new_reg_value:u32 = (((l as i16) + ((h as i16) <<8)) as i32) as u32;
        self.registers[0] += 4;
        if reg_a > 15 {return Err(MachineError::RegOutOfScale)}
        else {
            self.registers[reg_a as usize] = new_reg_value;
            return Ok(())
        }

    }


    /// 5 regA regB regC : 
    /// Store the content of register regB minus the content of register regC into register regA
    pub fn sub(&mut self) -> Result<(), MachineError> {
        let reg_a: u8 = self.memory[(self.get_ip()+1) as usize];
        let reg_b: u8 = self.memory[(self.get_ip()+2) as usize];
        let reg_c: u8 = self.memory[(self.get_ip()+3) as usize];
        self.registers[0] += 4;
        if reg_b > 15 || reg_c > 15 {return Err(MachineError::RegOutOfScale)}
        else {
            let value = self.registers[reg_b as usize].wrapping_sub(self.registers[reg_c as usize]);
            self.set_reg(reg_a, value)
        }
    }

    /// 6 regA : 
    /// Output the character whose unicode value is stored in the 8 low bits of register regA.
    pub fn out<T: Write>(&mut self, fd: &mut T) -> Result<(), MachineError> {
        let reg_a: u8 = self.memory[(self.get_ip()+1) as usize];
        self.registers[0] += 2;
        if reg_a > 15 {return Err(MachineError::RegOutOfScale)}
        else {
            let code = self.registers[reg_a as usize] as u32;
            let unichar: Option<char> = char::from_u32(code);
            if unichar == None {return Err(MachineError::NotUnicode)}
            let c = unichar.unwrap().to_string();
            match fd.write(c.as_bytes()) {
                Ok(_c) => return Ok(()),
                Err(_e) => return Err(MachineError::CouldNotWrite)
            };
        }
    }

    /// 7
    /// Exit the current program
    pub fn exit(&mut self) -> Result<bool, MachineError> {
        self.registers[0]+=1;
        return Ok(true)
    }

    /// 8 regA : 
    /// Output the signed number stored in register regA in decimal.
    pub fn out_number<T: Write>(&mut self,fd: &mut T) -> Result<(), MachineError> {
        let reg_a:u8 = self.memory[(self.get_ip()+1) as usize];
        self.registers[0] += 2;
        if reg_a > 15 {return Err(MachineError::RegOutOfScale)}
        else {
            let number: i32 = self.registers[reg_a as usize] as i32;
            let dec = number.to_string();
            match fd.write(dec.as_bytes()) {
                Ok(_c) => return Ok(()),
                Err(_e) => return Err(MachineError::CouldNotWrite)
            };
        }
    }
}
