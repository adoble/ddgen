use serde::Deserialize;

/// Flow control patterns
#[derive(Deserialize, Debug, Default)]
#[serde(tag = "type")]
pub enum FlowControl {
    /// Request data is send to the peripheral and then the response data is directly read in afterwards.
    #[default]
    #[serde(alias = "direct")]
    Direct,

    /// After the request data is sent, a small and fixed amount of data (a header) is read and inspected.
    /// If a condition is met (e.g. a bit is set) then the rest of the data is read. If the condition is
    /// not met then another read of the header is performed until the condition is met.
    #[serde(alias = "polled")]
    Polled { on: String, condition: String },
    // The following flow control patterns are not currently supported
    // - Timer. Request data is send to the peripheral and then, after a delay, the response data is directly read in afterwards.
    // - ReadyPin. An extra ready pin signals that data can be read in.
    // - MisoPin. The MISO pin is used to signal that data is now available
}
