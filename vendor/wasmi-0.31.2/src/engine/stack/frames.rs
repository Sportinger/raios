//! Data structures to represent the Wasm call stack during execution.

use super::{err_stack_overflow, DEFAULT_MAX_RECURSION_DEPTH};
use crate::{
    core::TrapCode,
    engine::{code_map::InstructionPtr, CompiledFunc},
    Instance,
};
use alloc::vec::Vec;

/// Identity of the guest function represented by an execution frame.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ExecutionFunc {
    function_index: u32,
    body: CompiledFunc,
}

impl ExecutionFunc {
    /// Creates guest function execution metadata.
    pub(crate) fn new(function_index: u32, body: CompiledFunc) -> Self {
        Self {
            function_index,
            body,
        }
    }

    /// Returns the module-local Wasm function index.
    pub(crate) fn function_index(self) -> u32 {
        self.function_index
    }

    /// Returns the compiled function body identity.
    pub(crate) fn body(self) -> CompiledFunc {
        self.body
    }
}

/// A function frame of a function on the call stack.
#[derive(Debug, Copy, Clone)]
pub struct FuncFrame {
    /// The pointer to the currently executed instruction.
    ip: InstructionPtr,
    /// The instance in which the function has been defined.
    ///
    /// # Note
    ///
    /// The instance is used to inspect and manipulate with data that is
    /// non-local to the function such as linear memories, global variables
    /// and tables.
    instance: Instance,
    /// Guest function identity when opt-in execution profiling is active.
    execution_func: Option<ExecutionFunc>,
}

impl FuncFrame {
    /// Creates a new [`FuncFrame`].
    pub(crate) fn new(
        ip: InstructionPtr,
        instance: &Instance,
        execution_func: Option<ExecutionFunc>,
    ) -> Self {
        Self {
            ip,
            instance: *instance,
            execution_func,
        }
    }

    /// Returns the current instruction pointer.
    pub fn ip(&self) -> InstructionPtr {
        self.ip
    }

    /// Returns the instance of the [`FuncFrame`].
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// Returns guest function identity when execution profiling is active.
    pub(crate) fn execution_func(&self) -> Option<ExecutionFunc> {
        self.execution_func
    }
}

/// The live function call stack storing the live function activation frames.
#[derive(Debug)]
pub struct CallStack {
    /// The call stack featuring the function frames in order.
    frames: Vec<FuncFrame>,
    /// The maximum allowed depth of the `frames` stack.
    recursion_limit: usize,
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_RECURSION_DEPTH)
    }
}

impl CallStack {
    /// Creates a new [`CallStack`] using the given recursion limit.
    pub fn new(recursion_limit: usize) -> Self {
        Self {
            frames: Vec::new(),
            recursion_limit,
        }
    }

    /// Initializes the [`CallStack`] given the Wasm function.
    pub(crate) fn init(
        &mut self,
        ip: InstructionPtr,
        instance: &Instance,
        execution_func: Option<ExecutionFunc>,
    ) {
        self.reset();
        self.frames
            .push(FuncFrame::new(ip, instance, execution_func));
    }

    /// Pushes a Wasm caller function onto the [`CallStack`].
    #[inline]
    pub fn push(&mut self, caller: FuncFrame) -> Result<(), TrapCode> {
        if self.len() == self.recursion_limit {
            return Err(err_stack_overflow());
        }
        self.frames.push(caller);
        Ok(())
    }

    /// Pops the last [`FuncFrame`] from the [`CallStack`] if any.
    #[inline]
    pub fn pop(&mut self) -> Option<FuncFrame> {
        self.frames.pop()
    }

    /// Peeks the last [`FuncFrame`] from the [`CallStack`] if any.
    #[inline]
    pub fn peek(&self) -> Option<&FuncFrame> {
        self.frames.last()
    }

    /// Returns the amount of function frames on the [`CallStack`].
    #[inline]
    fn len(&self) -> usize {
        self.frames.len()
    }

    /// Clears the [`CallStack`] entirely.
    ///
    /// # Note
    ///
    /// This is required since sometimes execution can halt in the middle of
    /// function execution which leaves the [`CallStack`] in an unspecified
    /// state. Therefore the [`CallStack`] is required to be reset before
    /// function execution happens.
    pub fn reset(&mut self) {
        self.frames.clear();
    }
}
