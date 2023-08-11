use std::{
    collections::VecDeque,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Mutex,
    },
    thread,
};

use altium::{
    sch::{Component, SchRecord},
    SchDoc,
    SchLib,
};
use egui::Vec2;
use log::{info, trace};

/// One entry per tab
static GLOBAL_QUEUE: Mutex<GlobalQueue> = Mutex::new(GlobalQueue::new());
/// Indicating that our GUI thread should pick this up
pub static HAS_FRESH_DATA: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct GlobalQueue {
    pub tabs: VecDeque<TabData>,
    pub errors: VecDeque<String>,
}

impl GlobalQueue {
    const fn new() -> Self {
        Self {
            tabs: VecDeque::new(),
            errors: VecDeque::new(),
        }
    }

    fn push_tab(tab: TabData) {
        GLOBAL_QUEUE.lock().unwrap().tabs.push_back(tab);
    }

    fn push_err<S: Into<String>>(err: S) {
        let err = err.into();
        log::debug!("pushing error {err}");
        GLOBAL_QUEUE.lock().unwrap().errors.push_back(err);
    }
    /// Empty the global context into vectors (our local/stored context)
    pub fn drain(
        tabs: &mut Vec<TabData>,
        recent_files: &mut Vec<PathBuf>,
        errors: &mut Vec<String>,
    ) {
        debug_assert!(HAS_FRESH_DATA.load(SeqCst));

        let gctx = &mut *GLOBAL_QUEUE.lock().unwrap();
        gctx.tabs
            .iter()
            .rev()
            .for_each(|tabdata| insert_recent_path(recent_files, &tabdata.path));
        tabs.extend(gctx.tabs.drain(..).rev());
        errors.extend(gctx.errors.drain(..).rev());

        HAS_FRESH_DATA.store(false, SeqCst);
    }
}

/// Insert a recent path if it doesn't exist, otherwise bubble it to the top
fn insert_recent_path(recent_files: &mut Vec<PathBuf>, path: &Path) {
    let existing_idx = recent_files.iter().position(|p| p == path);

    if let Some(idx) = existing_idx {
        if idx != 0 {
            recent_files.remove(idx);
            recent_files.insert(0, path.to_owned());
        }
    } else {
        recent_files.insert(0, path.to_owned());
    }
}

/// Top level tab representation
#[derive(Debug)]
pub struct TabData {
    /// The tab's title
    pub title: Box<str>,
    pub view_state: ViewState,
    /// Path to the file
    pub path: PathBuf,
    /// Contents of the tab
    pub inner: TabDataInner,
}

/// Scale and position of a view
#[derive(Clone, Copy, Debug)]
pub struct ViewState {
    /// Zoom if applicable, in m/px
    pub scale: f32,
    /// Center position
    pub center: Vec2,
}

impl ViewState {
    /// Apply a drag to this view state. Only do for a secondary (right) click.
    pub fn update_dragged_by(&mut self, drag_delta: Vec2) {
        self.center += drag_delta;
    }

    /// Update with zoom or multitouch (trackpad). Requires a zoom delta separately
    pub fn update_with_input_state(&mut self, istate: &egui::InputState) {
        const SCALE_MIN: f32 = 1e-6; // 1 um per px
        const SCALE_MAX: f32 = 10e-3; // 10 mm per px

        self.scale = f32::clamp(self.scale / istate.zoom_delta(), SCALE_MIN, SCALE_MAX);
        self.center += istate.scroll_delta;
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            scale: 1e-4, // m/px
            center: Vec2::default(),
        }
    }
}

/// Per-content-type variable tab data
#[derive(Debug)]
pub enum TabDataInner {
    SchLib(SchLibTab),
    SchDoc(SchDocTab),
}

/// Data for a single schematic library tab. This needs to hold a list of components
/// and track the selection
#[derive(Debug, Default)]
pub struct SchLibTab {
    pub components: Vec<Component>,
    /// Index in `components` to display in a scrollable list
    pub active_component: usize,
    pub search_query: String,
    /// A filter indicates to hide these items. We store hidden rather than
    /// visible so if there are no filters, we don't allocate
    pub hide_items: Vec<usize>,
}

#[derive(Debug, Default)]
pub struct SchDocTab {
    #[allow(dead_code)]
    pub records: Vec<SchRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FileTy {
    SchLib,
    PcbLib,
    SchDoc,
    PcbDoc,
}

/// Open a file, load its contents to the global static. Errors get pushed.
pub fn open_file_async(path: PathBuf) {
    info!("opening {}", path.display());

    thread::spawn(move || match open_file_sync_err(path) {
        Ok(()) => (),
        Err(e) => GlobalQueue::push_err(e),
    });
}

/// Open a file and add it to the global context. Returns an error on failure, rather than
/// pushing it.
pub fn open_file_sync_err(path: PathBuf) -> Result<(), String> {
    let file_ty = match &path.extension().map(OsStr::to_ascii_lowercase) {
        Some(v) if v == "schlib" => Ok(FileTy::SchLib),
        Some(v) if v == "pcblib" => Ok(FileTy::PcbLib),
        Some(v) if v == "schdoc" => Ok(FileTy::SchDoc),
        Some(v) if v == "pcbdoc" => Ok(FileTy::PcbDoc),
        Some(v) => Err(format!("unrecognized extension {}", v.to_string_lossy())),
        None => Err("No file type given".into()),
    }?;

    let optional_new_tab = match file_ty {
        FileTy::SchLib => schlib_to_tab(path),
        FileTy::SchDoc => schdoc_to_tab(path),
        FileTy::PcbLib => return Err("PcbLib not yet supported".into()),
        FileTy::PcbDoc => return Err("PcbDoc not yet supported".into()),
    };

    if let Some(new_tab) = optional_new_tab {
        GlobalQueue::push_tab(new_tab);
    }

    trace!("queue: {:?}", GLOBAL_QUEUE.lock().unwrap());

    HAS_FRESH_DATA.store(true, SeqCst);
    Ok(())
}

/// Create a tab if everything is OK, push an error if not
fn schlib_to_tab(path: PathBuf) -> Option<TabData> {
    let lib = match SchLib::open(&path) {
        Ok(lib) => lib,
        Err(e) => {
            GlobalQueue::push_err(e.to_string());
            return None;
        }
    };

    let mut inner = SchLibTab {
        components: lib.components().collect(),
        ..Default::default()
    };

    inner
        .components
        .sort_unstable_by(|a, b| a.name_cmp(b).unwrap());

    let new_tab = TabData {
        title: make_title(&path),
        view_state: ViewState::default(),
        path,
        inner: TabDataInner::SchLib(inner),
    };

    Some(new_tab)
}

/// Create a tab if everything is OK, push an error if not
fn schdoc_to_tab(path: PathBuf) -> Option<TabData> {
    let doc = match SchDoc::open(&path) {
        Ok(lib) => lib,
        Err(e) => {
            GlobalQueue::push_err(e.to_string());
            return None;
        }
    };

    let inner = SchDocTab {
        records: doc.records().cloned().collect(),
    };

    let new_tab = TabData {
        title: make_title(&path),
        view_state: ViewState::default(),
        path,
        inner: TabDataInner::SchDoc(inner),
    };

    Some(new_tab)
}

fn make_title(path: &Path) -> Box<str> {
    path.file_name()
        .unwrap_or("[Unnamed]".as_ref())
        .to_string_lossy()
        .into()
}
