use breakpad::{CallStack, FrameInfoMap, ProcessState, StackFrame};
use symbolic_common::Result;
use symbolic_common::ErrorKind::BreakpadError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    threads: Vec<Thread>,
}

impl From<ProcessState> for Event {
    fn from(state: ProcessState) -> Event {
        Event {
            threads: state.threads().iter().map(|s| (*s).into()).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thread {
    id: u32,
    frames: Vec<Frame>,
}

impl<'a> From<&'a CallStack> for Thread {
    fn from(stack: &'a CallStack) -> Self {
        Thread {
            id: stack.thread_id(),
            frames: stack.frames().iter().map(|f| (*f).into()).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    offset: u64,
    module: String,
}

impl<'a> From<&'a StackFrame> for Frame {
    fn from(frame: &'a StackFrame) -> Self {
        let module = frame.module().unwrap();
        Frame {
            offset: frame.instruction() - module.base_address(),
            module: module.id().to_string(),
        }
    }
}

pub fn get_minidump_modules(minidump: &[u8]) -> Result<Vec<String>> {
    ProcessState::from_minidump_buffer(minidump, None)
        .map(|state| {
            state
                .referenced_modules()
                .iter()
                .map(|m| m.id().to_string())
                .collect()
        })
        .map_err(|err| BreakpadError(err.description().into()).into())
}

pub fn process_minidump(minidump: &[u8], frame_infos: &FrameInfoMap) -> Result<Event> {
    ProcessState::from_minidump_buffer(minidump, Some(frame_infos))
        .map(|state| state.into())
        .map_err(|err| BreakpadError(err.description().into()).into())
}
