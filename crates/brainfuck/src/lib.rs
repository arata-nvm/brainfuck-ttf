use std::{collections::HashMap, str::FromStr};

pub const TAPE_SIZE: usize = 30000;

pub fn execute_program(program_str: &str, input: &str) -> Result<String, Error> {
    let program: Program = program_str.parse()?;
    let program = program.optimize_loop();
    let tape = Tape::new(TAPE_SIZE);
    let mut machine = Machine::new(program, tape)?;
    let input = Input::new(input);
    machine.execute(input)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnmatchedLoopStart,
    UnmatchedLoopEnd,
    OutOfBounds,
    Timeout,
}

#[derive(Debug)]
struct Machine {
    program: Program,
    tape: Tape,
    pc: ProgramCounter,

    jump_table: JumpTable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgramCounter(usize);

impl ProgramCounter {
    #[inline]
    fn inc(&mut self) {
        self.0 += 1;
    }
}

impl From<usize> for ProgramCounter {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
struct Program(Vec<Command>);

#[derive(Debug, Clone, Copy)]
enum Command {
    IncCursor(usize),
    DecCursor(usize),
    IncData(u8),
    DecData(u8),
    PutChar,
    GetChar,
    LoopStart,
    LoopEnd,

    ClearData,
    SearchZeroInc(usize),
    SearchZeroDec(usize),
    MoveDataInc(usize),
    MoveDataDec(usize),
}

#[derive(Debug)]
struct Tape {
    tape: Vec<u8>,
    cursor: usize,
}

#[derive(Debug)]
struct Input {
    input: String,
    cursor: usize,
}

#[derive(Debug)]
struct JumpTable(HashMap<ProgramCounter, ProgramCounter>);

impl Machine {
    fn new(program: Program, tape: Tape) -> Result<Self, Error> {
        Ok(Self {
            jump_table: JumpTable::build(&program)?,

            program,
            tape,
            pc: 0.into(),
        })
    }

    fn execute(&mut self, mut input: Input) -> Result<String, Error> {
        let mut output = String::new();
        while !self.is_end_of_program() {
            match self.program.read(self.pc) {
                Command::IncCursor(n) => self.tape.inc_cursor(n),
                Command::DecCursor(n) => self.tape.dec_cursor(n),
                Command::IncData(n) => self.tape.inc_data(n)?,
                Command::DecData(n) => self.tape.dec_data(n)?,
                Command::PutChar => output.push(self.tape.read()? as char),
                Command::GetChar => self.tape.write(input.read())?,
                Command::LoopStart => {
                    if self.tape.read()? == 0 {
                        self.set_pc(self.jump_table.find(self.pc));
                    }
                }
                Command::LoopEnd => {
                    if self.tape.read()? != 0 {
                        self.set_pc(self.jump_table.find(self.pc));
                    }
                }
                Command::ClearData => self.tape.write(0)?,
                Command::SearchZeroInc(n) => {
                    while self.tape.read()? != 0 {
                        self.tape.inc_cursor(n);
                    }
                }
                Command::SearchZeroDec(n) => {
                    while self.tape.read()? != 0 {
                        self.tape.dec_cursor(n);
                    }
                }
                Command::MoveDataInc(n) => {
                    let data = self.tape.read()?;
                    if data != 0 {
                        self.tape.write(0)?;
                        self.tape.inc_cursor(n);
                        self.tape.inc_data(data)?;
                        self.tape.dec_cursor(n);
                    }
                }
                Command::MoveDataDec(n) => {
                    let data = self.tape.read()?;
                    if data != 0 {
                        self.tape.write(0)?;
                        self.tape.dec_cursor(n);
                        self.tape.inc_data(data)?;
                        self.tape.inc_cursor(n);
                    }
                }
            }

            self.inc_pc();
        }
        Ok(output)
    }

    #[inline]
    fn is_end_of_program(&self) -> bool {
        self.pc.0 >= self.program.0.len()
    }

    #[inline]
    fn inc_pc(&mut self) {
        self.pc.inc();
    }

    #[inline]
    fn set_pc(&mut self, pc: ProgramCounter) {
        self.pc = pc;
    }
}

impl Program {
    #[inline]
    fn read(&self, pc: ProgramCounter) -> Command {
        self.0[pc.0]
    }

    fn optimize_loop(self) -> Self {
        let mut new_program = Vec::new();

        let mut idx = 0;
        while idx < self.0.len() {
            if !matches!(self.0[idx], Command::LoopStart) {
                new_program.push(self.0[idx]);
                idx += 1;
                continue;
            }

            match (
                self.0.get(idx + 1),
                self.0.get(idx + 2),
                self.0.get(idx + 3),
                self.0.get(idx + 4),
                self.0.get(idx + 5),
            ) {
                (
                    Some(Command::IncData(1) | Command::DecData(1)),
                    Some(Command::LoopEnd),
                    _,
                    _,
                    _,
                ) => {
                    new_program.push(Command::ClearData);
                    idx += 3;
                }
                (Some(Command::IncCursor(n)), Some(Command::LoopEnd), _, _, _) => {
                    new_program.push(Command::SearchZeroInc(*n));
                    idx += 3;
                }
                (Some(Command::DecCursor(n)), Some(Command::LoopEnd), _, _, _) => {
                    new_program.push(Command::SearchZeroDec(*n));
                    idx += 3;
                }
                (
                    Some(Command::DecData(1)),
                    Some(Command::IncCursor(n1)),
                    Some(Command::IncData(1)),
                    Some(Command::DecCursor(n2)),
                    Some(Command::LoopEnd),
                ) if n1 == n2 => {
                    new_program.push(Command::MoveDataInc(*n1));
                    idx += 6;
                }
                (
                    Some(Command::DecData(1)),
                    Some(Command::DecCursor(n1)),
                    Some(Command::IncData(1)),
                    Some(Command::IncCursor(n2)),
                    Some(Command::LoopEnd),
                ) if n1 == n2 => {
                    new_program.push(Command::MoveDataDec(*n1));
                    idx += 6;
                }
                _ => {
                    new_program.push(self.0[idx]);
                    idx += 1;
                }
            }
        }

        Self(new_program)
    }
}

impl FromStr for Program {
    type Err = Error;

    fn from_str(program_str: &str) -> Result<Self, Self::Err> {
        let mut program = Vec::with_capacity(program_str.len());
        let program_chars: Vec<_> = program_str.chars().collect();

        let is_valid_char = |c: char| matches!(c, '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']');

        let mut idx = 0;
        while idx < program_chars.len() {
            let current = program_chars[idx];
            let command = match current {
                '>' | '<' | '+' | '-' => {
                    idx += 1;
                    let mut n_repeat = 1;
                    while idx < program_chars.len() {
                        if program_chars[idx] == current {
                            idx += 1;
                            n_repeat += 1;
                        } else if !is_valid_char(program_chars[idx]) {
                            idx += 1;
                        } else {
                            break;
                        }
                    }
                    idx -= 1;
                    match current {
                        '>' => Command::IncCursor(n_repeat),
                        '<' => Command::DecCursor(n_repeat),
                        '+' => Command::IncData(n_repeat as u8),
                        '-' => Command::DecData(n_repeat as u8),
                        _ => unreachable!(),
                    }
                }
                '.' => Command::PutChar,
                ',' => Command::GetChar,
                '[' => Command::LoopStart,
                ']' => Command::LoopEnd,
                _ => {
                    idx += 1;
                    continue;
                }
            };
            idx += 1;
            program.push(command);
        }

        Ok(Self(program))
    }
}

impl Tape {
    fn new(length: usize) -> Self {
        Self {
            tape: vec![0; length],
            cursor: 0,
        }
    }

    #[inline]
    fn read(&self) -> Result<u8, Error> {
        self.tape
            .get(self.cursor)
            .copied()
            .ok_or(Error::OutOfBounds)
    }

    #[inline]
    fn write(&mut self, value: u8) -> Result<(), Error> {
        let ptr = self.tape.get_mut(self.cursor).ok_or(Error::OutOfBounds)?;
        *ptr = value;
        Ok(())
    }

    #[inline]
    fn inc_data(&mut self, n: u8) -> Result<(), Error> {
        self.write(self.read()?.wrapping_add(n))
    }

    #[inline]
    fn dec_data(&mut self, n: u8) -> Result<(), Error> {
        self.write(self.read()?.wrapping_sub(n))
    }

    #[inline]
    fn inc_cursor(&mut self, n: usize) {
        self.cursor = self.cursor.wrapping_add(n) % TAPE_SIZE;
    }

    #[inline]
    fn dec_cursor(&mut self, n: usize) {
        let (new_cursor, overflow) = self.cursor.overflowing_sub(n);
        if overflow {
            self.cursor = TAPE_SIZE - (n - self.cursor);
        } else {
            self.cursor = new_cursor;
        }
    }
}

impl Input {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            cursor: 0,
        }
    }

    #[inline]
    fn read(&mut self) -> u8 {
        match self.input.as_bytes().get(self.cursor) {
            Some(c) => {
                self.cursor += 1;
                *c
            }
            None => 0,
        }
    }
}

impl JumpTable {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn build(program: &Program) -> Result<Self, Error> {
        let mut jump_table = Self::new();
        let mut stack = Vec::new();
        for (pc, command) in program.0.iter().enumerate() {
            match command {
                Command::LoopStart => stack.push(pc),
                Command::LoopEnd => {
                    let start = stack.pop().ok_or(Error::UnmatchedLoopEnd)?;
                    jump_table.0.insert(start.into(), pc.into());
                    jump_table.0.insert(pc.into(), start.into());
                }
                _ => (),
            }
        }

        if !stack.is_empty() {
            return Err(Error::UnmatchedLoopStart);
        }

        Ok(jump_table)
    }

    #[inline]
    fn find(&self, pc: ProgramCounter) -> ProgramCounter {
        self.0[&pc]
    }
}

#[cfg(test)]
mod tests {
    use crate::execute_program;

    #[test]
    fn hello_world() {
        let program_str = r#">++++++++[<+++++++++>-]<.>++++[<+++++++>-]<+.+++++++..+++.>>++++++[<+++++++>-]<++.------------.>++++++[<+++++++++>-]<+.<.+++.------.--------.>>>++++[<++++++++>-]<+."#;
        assert_eq!(
            execute_program(program_str, ""),
            Ok(String::from("Hello, World!"))
        );
    }
}
