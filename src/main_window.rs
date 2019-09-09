use crate::view;

use view::{CellSize, ViewSize, LineCache, Cells, View};


#[derive(Debug, Clone)]
pub struct TabBar;

/// Represents the layout and structure of the main window.
/// I.E where the directory tree will be rendered, tab bar, message bar,
///     and handling of pane splits.
#[derive(Debug)]
struct MainWindow {
    /// all of the file views of this editor.
    views: Vec<View>,
    /// this is only shown when there are
    tab_bar: Option<TabBar>,
    /// a reference to all of the edit panes.
    // panes: Vec<pane::Pane>,
    /// the window the main window is associated with.
    window: Window,
    /// The size of a cell
    cell_size: CellSize,
    /// The window dimensions in cells.
    cells: Size<u32>,
}

impl MainWindow {
    fn new(window: Window, cell_size: CellSize) -> Result<Self> {
        // this the physical size of the window.
        let (width, height): (f64, f64) = window.get_physical_size().into();
        println!("Main Window Size: ({}, {})", width, height);

        let cells = Cells::compute_cells(width as f32, height as f32, cell_size);

        let loc = pane::Loc::new(0f32, 0f32);

        // let pane = Pane::new(
        //     PaneKind::Edit,
        //     Size::new(width as f32, height as f32),
        //     cells,
        //     loc,
        // );

        let mut main_window = Self {
            pane,
            pane_states: HashMap::new(),
            tab_bar: None,
            window,
            cell_size,
            cells,
        };

        // @TODO: Abstract this out to a method
        
        let id = main_window.pane.id();
        let size = main_window.pane.size().clone();

        main_window.create_pane_state(size, id)?;

        main_window.set_pane_active(main_window.pane.id());

        Ok(main_window)
    }
    
    /*
    pub fn pane(&self) -> &Pane {
        &self.pane
    }

    pub fn create_pane_state(&mut self, sz: Size<f32>, id: PaneID) -> Result<()> {
        self.pane_states.insert(
            id,
            PaneState::new(sz, id)
                .map_err(|e| Error::RenderError(render::Error::FrameBufferError(e)))?,
        );
        Ok(())
    }

    pub fn pane_mut(&mut self) -> &mut Pane {
        &mut self.pane
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    pub fn active_pane(&self) -> &pane::Pane {
        if let Some(id) = self.find_active_pane_id() {
            if let Some(pane) = Self::find_pane_by_id(self.pane(), id) {
                pane
            }
            else {
                panic!(format!("Unable to find active pane. Invalid ID: {:?}", id));
            }
        }
        else {
            unreachable!();
        }
    }

    pub fn active_pane_mut(&mut self) -> &mut pane::Pane {
        if let Some(id) = self.find_active_pane_id() {
            if let Some(pane) = Self::find_pane_by_id_mut(self.pane_mut(), id) {
                pane
            }
            else {
                panic!(format!("Unable to find active pane. Invalid ID: {:?}", id));
            }
        }
        else {
            unreachable!();
        }
    }

    fn find_pane_by_id(pane: &pane::Pane, id: PaneID) -> Option<&pane::Pane> {
        match *pane.kind() {
            PaneKind::Edit => {
                if pane.id() == id {
                    return Some(pane);
                }
                else {
                    return None;
                    // panic!(format!("Unable to find pane of given id: {:?}", id));
                }
            },
            PaneKind::Vert(ref layout) => {
                for pane in layout.iter() {
                    let p = Self::find_pane_by_id(pane, id);
                    if p.is_some() {
                        return p;
                    }
                }
            },
            PaneKind::Hor(ref layout) => {
                for pane in layout.iter() {
                    let p = Self::find_pane_by_id(pane, id);
                    if p.is_some() {
                        return p;
                    }
                }
            },
        }
        None
    }

    fn find_pane_by_id_mut(pane: &mut pane::Pane, id: PaneID) -> Option<&mut pane::Pane> {
        // *pane.kind();
        let pane_id = pane.id();
        match  pane.kind_mut() {
            PaneKind::Edit => {
                if pane_id == id {
                    return Some(pane);
                }
                else {
                    return None;
                    // panic!(format!("Unable to find pane of given id: {:?}", id));
                }
            },
            PaneKind::Vert(ref mut layout) => {
                for pane in layout.iter_mut() {
                    let p = Self::find_pane_by_id_mut(pane, id);
                    if p.is_some() {
                        return p;
                    }
                }
            },
            PaneKind::Hor(ref mut layout) => {
                for pane in layout.iter_mut() {
                    let p = Self::find_pane_by_id_mut(pane, id);
                    if p.is_some() {
                        return p;
                    }
                }
            },
        }
        None
    }
    
    pub fn find_active_pane_id(&self) -> Option<PaneID> {
        // attempting to be idiomatic
        let temp : Vec<(&PaneID, &PaneState)> = self.pane_states
                        .iter()
                        .filter(|(k, v)| v.active)
                        .collect();
        
        assert!(temp.len() == 1, format!("Unexpected number of active panes {}", temp.len()));

        Some(temp[0].0.clone())
    }

    pub fn get_pane_state(&self, id: PaneID) -> Option<&PaneState> {
        self.pane_states.get(&id)
    }

    pub fn get_pane_state_mut(&mut self, id: PaneID) -> Option<&mut PaneState> {
        self.pane_states.get_mut(&id)
    }

    pub fn set_pane_active(&mut self, id: PaneID) {
        if let Some(state) = self.get_pane_state_mut(id) {
            state.active = true;
        }
    }

    pub fn set_pane_deactive(&mut self, id: PaneID) {
        if let Some(state) = self.get_pane_state_mut(id) {
            state.active = false;
        }
    }
    */ 
}
