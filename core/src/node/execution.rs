use crate::pushr::push::instructions::{InstructionCache, InstructionSet};
use crate::pushr::push::interpreter::{PushInterpreter, PushInterpreterState};
use crate::pushr::push::parser::PushParser;
use crate::pushr::push::state::PushState;
use crate::pushr::push::io::PushMessage;
use crate::pushr::push::vector::{IntVector, BoolVector};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct Breakpoint {
    pub duration: Duration,
    pub name: String,
}

impl Breakpoint {
    pub fn new(arg_name: String, arg_duration: Duration) -> Self {
        Self {
            duration: arg_duration,
            name: arg_name,
        }
    }
}

pub struct PushExecutor {
    pub push_state: PushState,
    pub instruction_set: InstructionSet,
    pub instruction_cache: InstructionCache,
    pub breakpoints: Vec<Breakpoint>,
    pub last_break_time: Instant,
}

impl PushExecutor {
    pub fn new() -> Self {
        Self {
            push_state: PushState::new(),
            instruction_set: InstructionSet::new(),
            instruction_cache: InstructionCache::new(vec![]),
            breakpoints: vec![],
            last_break_time: Instant::now(),
        }
    }
    /// Loads default instruction set
    pub fn initialize(&mut self) {
        self.instruction_set.load();
        self.instruction_cache = self.instruction_set.cache();
    }

    /// Load program code to execution stack
    pub fn load(&mut self, program: String) {
        PushParser::parse_program(&mut self.push_state, &self.instruction_set, &program);
    }

    /// Run execution stack while receiving messages
    pub fn run(&mut self) -> PushInterpreterState {
        // PushInterpreter::copy_to_code_stack(&mut self.push_state);
        let icache = self.instruction_set.cache();
        let mut step_counter = 0;
        let start = Instant::now();
        loop {
            //println!("Step # {}", step_counter);
            if step_counter > self.push_state.configuration.eval_push_limit {
                return PushInterpreterState::StepLimitExceeded;
            }
            if start.elapsed()
                > Duration::from_millis(self.push_state.configuration.eval_time_limit)
            {
                return PushInterpreterState::TimeLimitExceeded;
            }
            let size_before_step = self.push_state.size();
            if PushInterpreter::step(&mut self.push_state, &mut self.instruction_set, &icache) {
                break;
            }
            if self.push_state.size()
                > size_before_step + self.push_state.configuration.growth_cap as usize
            {
                return PushInterpreterState::GrowthCapExceeded;
            }
            step_counter += 1;
        }
        PushInterpreterState::NoErrors
    }

    /// Execute program until a specific instruction is reached.
    /// If the instruction is not part of the stack it executes until
    /// the stack is empty.
    pub fn step_until(&mut self, instruction: String) {
        loop {
            if PushInterpreter::step(
                &mut self.push_state,
                &mut self.instruction_set,
                &self.instruction_cache,
            ) {
                break;
            }
            // Check duration for breakpoint
            if let Some(next_instruction) = self.push_state.exec_stack.get(0) {
                if next_instruction.to_string().starts_with("Identifier(BP") {
                    self.breakpoints.push(Breakpoint::new(
                        next_instruction.to_string(),
                        self.last_break_time.elapsed(),
                    ));
                    self.last_break_time = Instant::now();
                }
                if next_instruction.to_string() == instruction {
                    return;
                }
            }
        }
    }

    /// Invokes one step on the execution stack. Returns true if
    /// the stack is empty
    pub fn step(&mut self) -> bool {
        PushInterpreter::step(
            &mut self.push_state,
            &mut self.instruction_set,
            &self.instruction_cache,
        )
    }

    ///  Invoke program on input
    pub fn inject(&mut self, input: Vec<bool>)  {
        let input_vec = BoolVector::new(input);
        self.push_state.input_stack.push(PushMessage::new(IntVector::new(vec![]), input_vec));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn icache() -> InstructionCache {
        InstructionCache::new(vec![])
    }
}
