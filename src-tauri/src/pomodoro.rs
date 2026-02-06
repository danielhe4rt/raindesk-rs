use serde::{Deserialize, Serialize};

/// Pomodoro timer phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PomodoroPhase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Default for PomodoroPhase {
    fn default() -> Self {
        Self::Work
    }
}

/// Pomodoro timer status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PomodoroStatus {
    Idle,
    Running,
    Paused,
}

impl Default for PomodoroStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// Pomodoro timer state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroState {
    /// Current phase
    pub phase: PomodoroPhase,
    /// Current status
    pub status: PomodoroStatus,
    /// Remaining seconds in current phase
    pub remaining_secs: u32,
    /// Completed work sessions count
    pub completed_sessions: u32,
    /// Work duration in seconds (default 25 min)
    pub work_duration_secs: u32,
    /// Short break duration in seconds (default 5 min)
    pub short_break_duration_secs: u32,
    /// Long break duration in seconds (default 15 min)
    pub long_break_duration_secs: u32,
    /// Number of work sessions before long break (default 4)
    pub sessions_until_long_break: u32,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self::new()
    }
}

impl PomodoroState {
    pub fn new() -> Self {
        let work_duration = 25 * 60;
        Self {
            phase: PomodoroPhase::Work,
            status: PomodoroStatus::Idle,
            remaining_secs: work_duration,
            completed_sessions: 0,
            work_duration_secs: work_duration,
            short_break_duration_secs: 5 * 60,
            long_break_duration_secs: 15 * 60,
            sessions_until_long_break: 4,
        }
    }

    /// Start or resume the timer
    pub fn start(&mut self) {
        self.status = PomodoroStatus::Running;
    }

    /// Pause the timer
    pub fn pause(&mut self) {
        if self.status == PomodoroStatus::Running {
            self.status = PomodoroStatus::Paused;
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.phase = PomodoroPhase::Work;
        self.status = PomodoroStatus::Idle;
        self.remaining_secs = self.work_duration_secs;
        self.completed_sessions = 0;
    }

    /// Skip to the next phase
    pub fn skip_phase(&mut self) {
        self.transition_to_next_phase();
    }

    /// Tick the timer (call every second when running).
    /// Returns the new phase if a transition occurred.
    pub fn tick(&mut self) -> Option<PomodoroPhase> {
        if self.status != PomodoroStatus::Running {
            return None;
        }

        if self.remaining_secs > 0 {
            self.remaining_secs -= 1;
        }

        if self.remaining_secs == 0 {
            self.transition_to_next_phase();
            Some(self.phase)
        } else {
            None
        }
    }

    /// Transition to the next phase
    fn transition_to_next_phase(&mut self) {
        match self.phase {
            PomodoroPhase::Work => {
                self.completed_sessions += 1;
                if self.completed_sessions % self.sessions_until_long_break == 0 {
                    self.phase = PomodoroPhase::LongBreak;
                    self.remaining_secs = self.long_break_duration_secs;
                } else {
                    self.phase = PomodoroPhase::ShortBreak;
                    self.remaining_secs = self.short_break_duration_secs;
                }
            }
            PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak => {
                self.phase = PomodoroPhase::Work;
                self.remaining_secs = self.work_duration_secs;
            }
        }
    }

    /// Get remaining time as formatted string (MM:SS)
    #[allow(dead_code)]
    pub fn formatted_time(&self) -> String {
        let minutes = self.remaining_secs / 60;
        let seconds = self.remaining_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Get the phase name for display
    #[allow(dead_code)]
    pub fn phase_name(&self) -> &'static str {
        match self.phase {
            PomodoroPhase::Work => "Work",
            PomodoroPhase::ShortBreak => "Short Break",
            PomodoroPhase::LongBreak => "Long Break",
        }
    }
}
