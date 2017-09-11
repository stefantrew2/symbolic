use errors::Result;

// TODO(ja): Add more options (http://llvm.org/doxygen/Support_2MachO_8h_source.html)
// TODO(ja): Should this be splitted into type and subtype?
// TODO(ja): Implement Display for Architecture, when the structure is final
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Architecture {
    X86,
    X86_64,
    Arm64,
    ArmV7,
    ArmV7f,
}

impl Architecture {
    pub fn parse(string: &str) -> Result<Architecture> {
        use Architecture::*;
        Ok(match string {
            "x86" => X86,
            "x86_64" => X86_64,
            "arm64" => Arm64,
            "armv7" => ArmV7,
            "armv7f" => ArmV7f,
            _ => return Err("".into()),
        })
    }
}
