//! Checked names/indexes and flags of the inspected animation action policies.
//! Synchronization, backward modes and time slicing are not executed here.
use crate::fight::name_hash;
use anyhow::{Result, bail};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CyclePolicy {
    FromAnimation,
    NotCyclic,
    Cyclic,
    HoldEndFrame,
    NotCyclicBackward,
    CyclicBackward,
    EndEarlyForTimeSlicing,
}
impl CyclePolicy {
    pub fn from_hash(hash: u64) -> Result<Self> {
        for policy in [
            Self::FromAnimation,
            Self::NotCyclic,
            Self::Cyclic,
            Self::HoldEndFrame,
            Self::NotCyclicBackward,
            Self::CyclicBackward,
            Self::EndEarlyForTimeSlicing,
        ] {
            if name_hash(policy.label()) == hash {
                return Ok(policy);
            }
        }
        bail!("unknown animation cycle policy 0x{hash:016x}")
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::FromAnimation => "From Animation",
            Self::NotCyclic => "Not Cyclic",
            Self::Cyclic => "Cyclic",
            Self::HoldEndFrame => "Hold End Frame",
            Self::NotCyclicBackward => "Not Cyclic - Backward",
            Self::CyclicBackward => "Cyclic - Backward",
            Self::EndEarlyForTimeSlicing => "End Early For Time Slicing",
        }
    }
    pub fn native_index(self) -> u8 {
        match self {
            Self::FromAnimation => 0,
            Self::NotCyclic => 1,
            Self::Cyclic => 2,
            Self::HoldEndFrame => 3,
            Self::NotCyclicBackward => 4,
            Self::CyclicBackward => 5,
            Self::EndEarlyForTimeSlicing => 6,
        }
    }
    /// Flags written by the inspected skeletal begin path, after range setup.
    /// These flags alone do not implement backward or time-slicing behavior.
    pub fn driver_flags(self, clip_cyclic: bool) -> DriverFlags {
        DriverFlags {
            cyclic: match self {
                Self::FromAnimation => clip_cyclic,
                Self::Cyclic | Self::CyclicBackward => true,
                _ => false,
            },
            hold_end_frame: self == Self::HoldEndFrame,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DriverFlags {
    pub cyclic: bool,
    pub hold_end_frame: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncPhasePolicy {
    Legacy,
    FromPuppetPhase,
}
impl SyncPhasePolicy {
    pub fn from_hash(hash: u64) -> Result<Self> {
        for policy in [Self::Legacy, Self::FromPuppetPhase] {
            if name_hash(policy.label()) == hash {
                return Ok(policy);
            }
        }
        bail!("unknown animation sync-phase policy 0x{hash:016x}")
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Legacy => "Legacy",
            Self::FromPuppetPhase => "From Puppet Phase",
        }
    }
    pub fn native_index(self) -> u8 {
        match self {
            Self::Legacy => 0,
            Self::FromPuppetPhase => 1,
        }
    }
}
