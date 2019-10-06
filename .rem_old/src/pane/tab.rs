use super::{Pane, PaneID};
use size::Size;

pub struct Tab {
    pane: Pane,
}

pub struct TabGroup {
    tabs: Vec<Tabs>
}
