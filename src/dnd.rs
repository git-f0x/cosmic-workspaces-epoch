//! Types related to drag-and-drop

use cosmic::{
    cctk::{
        cosmic_protocols::workspace::v1::client::zcosmic_workspace_handle_v1,
        wayland_client::{protocol::wl_output, Proxy},
    },
    iced::clipboard::mime::AsMimeTypes,
};
use std::{borrow::Cow, sync::LazyLock};

use crate::backend::{ZcosmicToplevelHandleV1, ZcosmicWorkspaceHandleV1};

// Include `pid` in mime. Want to drag between our surfaces, but not another
// process, if we use Wayland object ids.
#[allow(dead_code)]
static WORKSPACE_MIME: LazyLock<String> =
    LazyLock::new(|| format!("text/x.cosmic-workspace-id-{}", std::process::id()));

static TOPLEVEL_MIME: LazyLock<String> =
    LazyLock::new(|| format!("text/x.cosmic-toplevel-id-{}", std::process::id()));

#[derive(Clone, Debug)]
pub enum DragSurface {
    #[allow(dead_code)]
    Workspace {
        handle: ZcosmicWorkspaceHandleV1,
        output: wl_output::WlOutput,
    },
    Toplevel {
        handle: ZcosmicToplevelHandleV1,
        output: wl_output::WlOutput,
    },
}

// TODO store protocol object id?
#[derive(Clone, Debug)]
pub struct DragToplevel {}

impl AsMimeTypes for DragToplevel {
    fn available(&self) -> Cow<'static, [String]> {
        vec![TOPLEVEL_MIME.clone()].into()
    }

    fn as_bytes(&self, mime_type: &str) -> Option<Cow<'static, [u8]>> {
        if mime_type == *TOPLEVEL_MIME {
            Some(Vec::new().into())
        } else {
            None
        }
    }
}

impl cosmic::iced::clipboard::mime::AllowedMimeTypes for DragToplevel {
    fn allowed() -> Cow<'static, [String]> {
        vec![TOPLEVEL_MIME.clone()].into()
    }
}

impl TryFrom<(Vec<u8>, std::string::String)> for DragToplevel {
    type Error = ();
    fn try_from((_bytes, mime_type): (Vec<u8>, String)) -> Result<Self, ()> {
        if mime_type == *TOPLEVEL_MIME {
            Ok(Self {})
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum DropTarget {
    WorkspaceSidebarEntry(
        zcosmic_workspace_handle_v1::ZcosmicWorkspaceHandleV1,
        wl_output::WlOutput,
    ),
}

impl DropTarget {
    /// Encode as a u64 for iced/smithay_sctk to associate drag destination area with widget.
    pub fn drag_id(&self) -> u64 {
        // https://doc.rust-lang.org/std/mem/fn.discriminant.html#accessing-the-numeric-value-of-the-discriminant
        let discriminant = unsafe { *<*const _>::from(self).cast::<u8>() };
        match self {
            Self::WorkspaceSidebarEntry(workspace, _output) => {
                // TODO consider workspace that span multiple outputs?
                let id = workspace.id().protocol_id();
                (u64::from(discriminant) << 32) | u64::from(id)
            }
        }
    }
}
