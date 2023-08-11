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
    sch::{Component, SchDocRecords},
    SchDoc,
    SchLib,
};
use log::info;

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
    pub title: Box<str>,
    pub path: PathBuf,
    pub inner: TabDataInner,
}

/// Tab variable data
#[derive(Debug)]
pub enum TabDataInner {
    SchLib(SchLibTab),
    SchDoc(SchDocTab),
}

/// Data for a single tab
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
    pub records: SchDocRecords,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FileTy {
    SchLib,
    PcbLib,
    SchDoc,
    PcbDoc,
}

/// Open a file, load its contents to the global static. If the file type is not
/// recognized, return an error immediately.
pub fn open_file(path: PathBuf) {
    info!("opening {}", path.display());

    let mut ty_res = match &path.extension().map(OsStr::to_ascii_lowercase) {
        Some(v) if v == "schlib" => Ok(FileTy::SchLib),
        Some(v) if v == "pcblib" => Ok(FileTy::PcbLib),
        Some(v) if v == "schdoc" => Ok(FileTy::SchDoc),
        Some(v) if v == "pcbdoc" => Ok(FileTy::PcbDoc),
        Some(v) => Err(format!("unrecognized extension {}", v.to_string_lossy())),
        None => Err("No file type given".into()),
    };

    match ty_res {
        Ok(ty) => {
            thread::spawn(move || open_file_worker(path, ty));
        }
        Err(emsg) => GlobalQueue::push_err(emsg),
    }
}

/// Open a file and add it to the global context. Push an error if something fails
fn open_file_worker(path: PathBuf, file_ty: FileTy) {
    let optional_new_tab = match file_ty {
        FileTy::SchLib => schlib_to_tab(path),
        FileTy::SchDoc => schdoc_to_tab(path),
        FileTy::PcbLib => return GlobalQueue::push_err("PcbLib not yet supported"),
        FileTy::PcbDoc => return GlobalQueue::push_err("PcbDoc not yet supported"),
    };

    if let Some(new_tab) = optional_new_tab {
        GlobalQueue::push_tab(new_tab);
    }

    info!("queue: {:?}", GLOBAL_QUEUE.lock().unwrap());

    HAS_FRESH_DATA.store(true, SeqCst);
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
        records: doc.into_records(),
    };

    let new_tab = TabData {
        title: make_title(&path),
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
